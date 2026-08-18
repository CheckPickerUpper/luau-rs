//! Per-function wasm-to-Luau translation.
//!
//! One generated Luau function keeps a shared value stack (`stack` plus an
//! explicit `sp`) and lowers wasm structured control flow (blocks, loops, ifs)
//! to nested `while true` loops with per-construct exit/restart flags, so `br`
//! can break out of any enclosing construct with correct Luau semantics.

use crate::wasm::{DecodedFunction, WasmDecodeProblemReason, WasmValueType};
use std::collections::HashSet;
use walrus::ir::{ExtendedLoad, Instr, InstrSeqId, LoadKind, MemArg, StoreKind};
use walrus::{LocalFunction, LocalId, Module};

use super::ops::{binop_expression, luau_constant, unop_expression};
use super::problem::TranslationProblemReason;
use super::writer::{TextWriter, LUAU_INDEX_OFFSET, SP_NAME, STACK_NAME, WASM_PAGE_SIZE_BYTES};

/// A stable identifier for one structured construct within a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ConstructId(usize);

/// A stable identifier for one temporary value within a function.
#[derive(Debug, Clone, Copy)]
struct TempId(usize);

/// The kind of a structured wasm construct, for branch-target handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConstructKind {
    Block,
    Loop,
    If,
}

/// One open structured construct with its flag names and branch seq ids.
struct ConstructContext {
    id: ConstructId,
    kind: ConstructKind,
    seq_ids: HashSet<InstrSeqId>,
}

/// The Luau statements that realize one branch.
enum BranchAction {
    /// Break the innermost loop.
    Break,
    /// Continue the innermost loop.
    Continue,
    /// Return from the function.
    Return,
    /// Set flags for the crossed constructs, then break the innermost loop.
    Cascade { set_flags: Vec<String> },
}

/// Every input needed to emit one defined function body.
pub struct FunctionBodyInput<'a> {
    pub function: &'a DecodedFunction,
    pub local_function: &'a LocalFunction,
    pub module: &'a Module,
    pub entry_sequence: InstrSeqId,
}

/// Emits one defined wasm function as a strict Luau function.
pub fn emit_function_body(
    input: &FunctionBodyInput<'_>,
    writer: &mut TextWriter,
) -> Result<(), TranslationProblemReason> {
    let (parameters, return_annotation) = function_signature(input.function);
    let comment_name = input
        .function
        .name()
        .map_or_else(String::new, |name| format!(" -- {name}"));
    if return_annotation.is_empty() {
        writer.line(&format!(
            "local function FUNC_{}({}){}",
            input.function.index(),
            parameters.join(", "),
            comment_name
        ));
    } else {
        writer.line(&format!(
            "local function FUNC_{}({}): {}{}",
            input.function.index(),
            parameters.join(", "),
            return_annotation,
            comment_name
        ));
    }
    writer.push_indent();
    let mut emitter = FunctionEmitter::new(input, writer);
    emitter.emit_prologue()?;
    emitter.translate_sequence(input.entry_sequence)?;
    emitter.emit_final_return();
    writer.pop_indent();
    writer.line("end");
    writer.line("");
    Ok(())
}

/// Returns the (parameter names, return annotation) pair for a generated function.
/// Returns the (parameter names, return annotation) pair for a generated function.
#[must_use]
pub fn function_signature(function: &DecodedFunction) -> (Vec<String>, String) {
    let parameters = function
        .params()
        .iter()
        .enumerate()
        .map(|(index, value_type)| format!("p{index}: {}", value_type.luau_type_name()))
        .collect();
    let return_annotation = match function.results() {
        [] => String::new(),
        [single] => (*single).luau_type_name().into(),
        multiple => multiple
            .iter()
            .map(|value_type| (*value_type).luau_type_name())
            .collect::<Vec<_>>()
            .join(", "),
    };
    (parameters, return_annotation)
}

const fn zero_initializer(value_type: WasmValueType) -> &'static str {
    match value_type {
        WasmValueType::I32 | WasmValueType::I64 | WasmValueType::F32 | WasmValueType::F64 => "0",
        WasmValueType::ExternRef | WasmValueType::FuncRef => "nil",
    }
}

/// Collects every local of a function in deterministic order: parameters
/// first (in wasm order), then declared locals sorted by arena id.
///
/// Walrus stores declared locals in a module-wide arena rather than on the
/// function, so the translator derives the per-function set by walking the
/// body for referenced `local.get`/`local.set`/`local.tee` ids.
fn collect_local_order(input: &FunctionBodyInput<'_>) -> Vec<LocalId> {
    let mut referenced = Vec::new();
    collect_referenced_locals(input, input.entry_sequence, &mut referenced);
    let mut local_order = input.local_function.args.clone();
    let mut declared = referenced
        .into_iter()
        .filter(|local| !input.local_function.args.contains(local))
        .collect::<Vec<_>>();
    declared.sort_unstable();
    declared.dedup();
    local_order.extend(declared);
    local_order
}

fn collect_referenced_locals(
    input: &FunctionBodyInput<'_>,
    sequence_id: InstrSeqId,
    referenced: &mut Vec<LocalId>,
) {
    let sequence = input.local_function.block(sequence_id);
    for (instruction, _) in &sequence.instrs {
        match instruction {
            Instr::LocalGet(local_get) => referenced.push(local_get.local),
            Instr::LocalSet(local_set) => referenced.push(local_set.local),
            Instr::LocalTee(local_tee) => referenced.push(local_tee.local),
            Instr::Block(block) => {
                collect_referenced_locals(input, block.seq, referenced);
            }
            Instr::Loop(loop_) => {
                collect_referenced_locals(input, loop_.seq, referenced);
            }
            Instr::IfElse(if_else) => {
                collect_referenced_locals(input, if_else.consequent, referenced);
                collect_referenced_locals(input, if_else.alternative, referenced);
            }
            _ => {}
        }
    }
}

/// Translates one function body into Luau statements inside a `TextWriter`.
struct FunctionEmitter<'a> {
    input: &'a FunctionBodyInput<'a>,
    writer: &'a mut TextWriter,
    local_positions: Vec<walrus::LocalId>,
    next_construct_id: ConstructId,
    next_temp_id: TempId,
    contexts: Vec<ConstructContext>,
    /// Whether the most recent statement terminates the block (a `return` or
    /// unconditional branch). Dead statements are skipped because the pinned
    /// Luau enforces Lua 5.1's rule that `return` is the last statement of a
    /// block.
    unreachable: bool,
}

impl<'a> FunctionEmitter<'a> {
    fn new(input: &'a FunctionBodyInput<'a>, writer: &'a mut TextWriter) -> Self {
        Self {
            input,
            writer,
            local_positions: collect_local_order(input),
            next_construct_id: ConstructId(0),
            next_temp_id: TempId(0),
            contexts: Vec::new(),
            unreachable: false,
        }
    }

    fn emit_prologue(&mut self) -> Result<(), TranslationProblemReason> {
        self.writer
            .line(&format!("local {STACK_NAME}: {{any}} = {{}}"));
        self.writer.line(&format!("local {SP_NAME}: number = 0"));
        for local_index in self.input.function.params().len()..self.local_positions.len() {
            let local_type = self.local_type_at(local_index)?;
            self.writer.line(&format!(
                "local {}: {} = {}",
                self.local_name_at(local_index),
                local_type.luau_type_name(),
                zero_initializer(local_type)
            ));
        }
        Ok(())
    }

    fn translate_sequence(
        &mut self,
        sequence_id: InstrSeqId,
    ) -> Result<(), TranslationProblemReason> {
        let sequence = self.input.local_function.block(sequence_id);
        let instructions = sequence.instrs.clone();
        for (instruction, _) in &instructions {
            if self.unreachable {
                break;
            }
            self.translate_instruction(instruction)?;
        }
        Ok(())
    }

    fn translate_instruction(
        &mut self,
        instruction: &Instr,
    ) -> Result<(), TranslationProblemReason> {
        match instruction {
            Instr::Const(constant) => {
                self.emit_push(&luau_constant(constant.value)?);
                Ok(())
            }
            Instr::LocalGet(local_get) => {
                let local_name = self.local_name(local_get.local)?;
                self.emit_push(&local_name);
                Ok(())
            }
            Instr::LocalSet(local_set) => {
                let local_name = self.local_name(local_set.local)?;
                self.emit_pop_into(&local_name);
                Ok(())
            }
            Instr::LocalTee(local_tee) => {
                let local_name = self.local_name(local_tee.local)?;
                self.emit_tee_into(&local_name);
                Ok(())
            }
            Instr::GlobalGet(global_get) => {
                self.emit_push(&format!(
                    "GLOBALS[{}]",
                    global_get.global.index() + LUAU_INDEX_OFFSET
                ));
                Ok(())
            }
            Instr::GlobalSet(global_set) => {
                self.emit_pop_into(&format!(
                    "GLOBALS[{}]",
                    global_set.global.index() + LUAU_INDEX_OFFSET
                ));
                Ok(())
            }
            Instr::Unop(unop) => self.emit_unop(unop.op),
            Instr::Binop(binop) => self.emit_binop(binop.op),
            Instr::TernOp(ternop) => Err(TranslationProblemReason::UnsupportedInstruction {
                instruction: format!("ternary op {ternop:?}"),
            }),
            Instr::Select(_) => {
                self.emit_select();
                Ok(())
            }
            Instr::Drop(_) => {
                self.emit_drop();
                Ok(())
            }
            Instr::Call(call) => {
                self.emit_call(call.func.index());
                Ok(())
            }
            Instr::CallIndirect(call_indirect) => {
                self.emit_call_indirect(call_indirect.ty, call_indirect.table);
                Ok(())
            }
            Instr::Block(block) => self.emit_construct(ConstructKind::Block, block.seq),
            Instr::Loop(loop_) => self.emit_construct(ConstructKind::Loop, loop_.seq),
            Instr::IfElse(if_else) => self.emit_if(if_else.consequent, if_else.alternative),
            Instr::Br(br) => {
                self.emit_branch(br.block);
                self.unreachable = true;
                Ok(())
            }
            Instr::BrIf(br_if) => {
                self.emit_branch_if(br_if.block);
                Ok(())
            }
            Instr::BrTable(br_table) => {
                self.emit_branch_table(&br_table.blocks, br_table.default);
                self.unreachable = true;
                Ok(())
            }
            Instr::Return(_) => {
                self.emit_return();
                self.unreachable = true;
                Ok(())
            }
            Instr::Unreachable(_) => {
                self.writer.line("error(\"wasm trap: unreachable\")");
                self.unreachable = true;
                Ok(())
            }
            Instr::MemorySize(_) => {
                self.emit_push("MEMORY_PAGES");
                Ok(())
            }
            Instr::MemoryGrow(_) => {
                self.emit_memory_grow();
                Ok(())
            }
            Instr::Load(load) => self.emit_load(load.kind, load.arg),
            Instr::Store(store) => self.emit_store(store.kind, store.arg),
            unsupported => Err(TranslationProblemReason::UnsupportedInstruction {
                instruction: format!("{unsupported:?}"),
            }),
        }
    }

    fn emit_construct(
        &mut self,
        kind: ConstructKind,
        sequence_id: InstrSeqId,
    ) -> Result<(), TranslationProblemReason> {
        let construct_id = self.open_construct(kind, sequence_id);
        self.writer.line("while true do");
        self.writer.push_indent();
        self.translate_sequence(sequence_id)?;
        if (kind == ConstructKind::Block || kind == ConstructKind::If) && !self.unreachable {
            self.writer.line("break");
        }
        self.writer.pop_indent();
        self.writer.line("end");
        self.unreachable = false;
        self.close_construct(construct_id);
        self.emit_after_construct_checks();
        Ok(())
    }

    fn emit_if(
        &mut self,
        consequent: InstrSeqId,
        alternative: InstrSeqId,
    ) -> Result<(), TranslationProblemReason> {
        let construct_id = self.open_construct(ConstructKind::If, consequent);
        self.writer.line("while true do");
        self.writer.push_indent();
        let condition = self.pop_value();
        self.writer.line(&format!("if {condition} ~= 0 then"));
        self.writer.push_indent();
        self.translate_sequence(consequent)?;
        self.writer.pop_indent();
        self.writer.line("else");
        self.writer.push_indent();
        self.translate_sequence(alternative)?;
        self.writer.pop_indent();
        self.writer.line("end");
        if !self.unreachable {
            self.writer.line("break");
        }
        self.writer.pop_indent();
        self.writer.line("end");
        self.unreachable = false;
        self.close_construct(construct_id);
        self.emit_after_construct_checks();
        Ok(())
    }

    fn emit_branch(&mut self, target: InstrSeqId) {
        let branch = self.branch_action(target);
        self.emit_branch_action(&branch);
    }

    fn emit_branch_if(&mut self, target: InstrSeqId) {
        let condition = self.pop_value();
        let branch = self.branch_action(target);
        self.writer.line(&format!("if {condition} ~= 0 then"));
        self.writer.push_indent();
        self.emit_branch_action(&branch);
        self.writer.pop_indent();
        self.writer.line("end");
    }

    fn emit_branch_table(&mut self, blocks: &[InstrSeqId], default: InstrSeqId) {
        let index = self.pop_value();
        let mut first_arm = true;
        for (target_index, target) in blocks.iter().enumerate() {
            let branch = self.branch_action(*target);
            let connector = if first_arm { "if" } else { "elseif" };
            self.writer
                .line(&format!("{connector} {index} == {target_index} then"));
            self.writer.push_indent();
            self.emit_branch_action(&branch);
            self.writer.pop_indent();
            first_arm = false;
        }
        let default_branch = self.branch_action(default);
        self.writer.line("else");
        self.writer.push_indent();
        self.emit_branch_action(&default_branch);
        self.writer.pop_indent();
        self.writer.line("end");
    }

    /// Computes the Luau statements that realize a branch to a target seq.
    fn branch_action(&self, target: InstrSeqId) -> BranchAction {
        let Some(target_position) = self
            .contexts
            .iter()
            .rposition(|context| context.seq_ids.contains(&target))
        else {
            // The branch targets the function entry: a full return.
            return BranchAction::Return;
        };
        let Some(target_construct) = self.contexts.get(target_position) else {
            return BranchAction::Return;
        };
        let depth = self.contexts.len() - target_position;
        if depth == 1 {
            return match target_construct.kind {
                ConstructKind::Loop => BranchAction::Continue,
                ConstructKind::Block | ConstructKind::If => BranchAction::Break,
            };
        }
        // Crossing constructs: set flags from the target construct through the
        // innermost construct, then break the innermost loop.
        let mut set_flags = Vec::new();
        for construct in &self.contexts[target_position..] {
            if construct.id == target_construct.id && target_construct.kind == ConstructKind::Loop {
                set_flags.push(construct_restart_name(construct.id));
            } else {
                set_flags.push(construct_flag_name(construct.id));
            }
        }
        BranchAction::Cascade { set_flags }
    }

    fn emit_branch_action(&mut self, branch: &BranchAction) {
        match branch {
            BranchAction::Break => self.writer.line("break"),
            BranchAction::Continue => self.writer.line("continue"),
            BranchAction::Return => self.emit_return(),
            BranchAction::Cascade { set_flags } => {
                for flag in set_flags {
                    self.writer.line(&format!("{flag} = true"));
                }
                self.writer.line("break");
            }
        }
    }

    /// Emits the flag checks that follow every nested construct inside a body.
    fn emit_after_construct_checks(&mut self) {
        let Some(parent) = self.contexts.last() else {
            return;
        };
        let mut exit_conditions = Vec::new();
        for construct in &self.contexts {
            exit_conditions.push(construct_flag_name(construct.id));
        }
        self.writer
            .line(&format!("if {} then", exit_conditions.join(" or ")));
        self.writer.push_indent();
        self.writer.line("break");
        self.writer.pop_indent();
        self.writer.line("end");
        if parent.kind == ConstructKind::Loop {
            let restart_name = construct_restart_name(parent.id);
            self.writer.line(&format!("if {restart_name} then"));
            self.writer.push_indent();
            self.writer.line(&format!("{restart_name} = false"));
            self.writer.line("continue");
            self.writer.pop_indent();
            self.writer.line("end");
        }
    }

    fn open_construct(&mut self, kind: ConstructKind, sequence_id: InstrSeqId) -> ConstructId {
        let construct = ConstructContext {
            id: self.next_construct_id,
            kind,
            seq_ids: HashSet::from([sequence_id]),
        };
        self.next_construct_id = ConstructId(self.next_construct_id.0 + 1);
        let exit_flag = construct_flag_name(construct.id);
        self.writer
            .line(&format!("local {exit_flag}: boolean = false"));
        if kind == ConstructKind::Loop {
            let restart_flag = construct_restart_name(construct.id);
            self.writer
                .line(&format!("local {restart_flag}: boolean = false"));
        }
        let construct_id = construct.id;
        self.unreachable = false;
        self.contexts.push(construct);
        construct_id
    }

    fn close_construct(&mut self, construct_id: ConstructId) {
        self.contexts.retain(|context| context.id != construct_id);
    }

    fn emit_final_return(&mut self) {
        let result_count = self.input.function.results().len();
        if result_count == 0 {
            return;
        }
        let values = (0..result_count)
            .map(|offset| {
                let position = result_count - offset;
                format!("stack[sp - {}]", position - 1)
            })
            .collect::<Vec<_>>()
            .join(", ");
        self.writer.line(&format!("return {values}"));
    }

    fn emit_return(&mut self) {
        self.emit_final_return();
    }

    fn local_name(&self, local: walrus::LocalId) -> Result<String, TranslationProblemReason> {
        let Some(position) = self
            .local_positions
            .iter()
            .position(|candidate| *candidate == local)
        else {
            return Err(TranslationProblemReason::Internal(format!(
                "unknown local {local:?}"
            )));
        };
        Ok(self.local_name_at(position))
    }

    fn local_name_at(&self, position: usize) -> String {
        if position < self.input.function.params().len() {
            format!("p{position}")
        } else {
            format!("l{}", position - self.input.function.params().len())
        }
    }

    fn local_type_at(&self, position: usize) -> Result<WasmValueType, TranslationProblemReason> {
        let Some(local_id) = self.local_positions.get(position) else {
            return Err(TranslationProblemReason::Internal(format!(
                "local position {position} is out of range"
            )));
        };
        let local = self.input.module.locals.get(*local_id);
        match WasmValueType::try_from(local.ty()) {
            Ok(value_type) => Ok(value_type),
            Err(WasmDecodeProblemReason::UnsupportedVectorType) => {
                Err(TranslationProblemReason::UnsupportedInstruction {
                    instruction: format!("v128 local at position {position}"),
                })
            }
            Err(other_reason) => Err(TranslationProblemReason::Internal(other_reason.to_string())),
        }
    }

    fn emit_push(&mut self, value_expression: &str) {
        self.writer.line(&format!("{SP_NAME} += 1"));
        self.writer
            .line(&format!("{STACK_NAME}[{SP_NAME}] = {value_expression}"));
    }

    /// Pops one value into a fresh temporary and returns its name.
    fn pop_value(&mut self) -> String {
        let temp = self.next_temp();
        self.writer
            .line(&format!("local {temp} = {STACK_NAME}[{SP_NAME}]"));
        self.writer.line(&format!("{SP_NAME} -= 1"));
        temp
    }

    fn emit_pop_into(&mut self, target: &str) {
        let value = self.pop_value();
        self.writer.line(&format!("{target} = {value}"));
    }

    fn emit_tee_into(&mut self, target: &str) {
        let temp = self.next_temp();
        self.writer
            .line(&format!("local {temp} = {STACK_NAME}[{SP_NAME}]"));
        self.writer.line(&format!("{target} = {temp}"));
    }

    fn emit_drop(&mut self) {
        self.writer.line(&format!("{SP_NAME} -= 1"));
    }

    fn emit_select(&mut self) {
        let condition = self.pop_value();
        let when_false = self.pop_value();
        let when_true = self.pop_value();
        self.writer.line(&format!("if {condition} ~= 0 then"));
        self.writer.push_indent();
        self.emit_push(&when_true);
        self.writer.pop_indent();
        self.writer.line("else");
        self.writer.push_indent();
        self.emit_push(&when_false);
        self.writer.pop_indent();
        self.writer.line("end");
    }

    fn emit_unop(&mut self, op: walrus::ir::UnaryOp) -> Result<(), TranslationProblemReason> {
        let operand = self.pop_value();
        self.emit_push(&unop_expression(op, &operand)?);
        Ok(())
    }

    fn emit_binop(&mut self, op: walrus::ir::BinaryOp) -> Result<(), TranslationProblemReason> {
        let right = self.pop_value();
        let left = self.pop_value();
        self.emit_push(&binop_expression(op, &left, &right)?);
        Ok(())
    }

    fn emit_call(&mut self, function_index: usize) {
        let arity = self.input.function.params().len();
        let arguments = self.pop_arguments(arity);
        self.emit_push(&format!("FUNC_{function_index}({})", arguments.join(", ")));
    }

    fn emit_call_indirect(&mut self, function_type: walrus::TypeId, _table: walrus::TableId) {
        let table_index = self.pop_value();
        let function_type = self.input.module.types.get(function_type);
        let arguments = self.pop_arguments(function_type.params().len());
        self.emit_push(&format!(
            "FUNCTIONS[{table_index} + {LUAU_INDEX_OFFSET}]({})",
            arguments.join(", ")
        ));
    }

    /// Pops call arguments (last argument on top) and returns them in order.
    fn pop_arguments(&mut self, arity: usize) -> Vec<String> {
        let mut arguments = Vec::with_capacity(arity);
        for _ in 0..arity {
            arguments.push(self.pop_value());
        }
        arguments.reverse();
        arguments
    }

    fn emit_memory_grow(&mut self) {
        let delta = self.pop_value();
        let old = self.next_temp();
        self.writer.line(&format!("local {old} = MEMORY_PAGES"));
        self.writer
            .line(&format!("if MEMORY_PAGES + {delta} <= MAXIMUM_PAGES then"));
        self.writer.push_indent();
        self.writer.line(&format!(
            "local new_memory = buffer.create((MEMORY_PAGES + {delta}) * {WASM_PAGE_SIZE_BYTES})"
        ));
        self.writer.line(&format!(
            "buffer.copy(new_memory, 0, MEMORY, 0, MEMORY_PAGES * {WASM_PAGE_SIZE_BYTES})"
        ));
        self.writer.line("MEMORY = new_memory");
        self.writer
            .line(&format!("MEMORY_PAGES = MEMORY_PAGES + {delta}"));
        self.emit_push(&old);
        self.writer.pop_indent();
        self.writer.line("else");
        self.writer.push_indent();
        self.emit_push("-1");
        self.writer.pop_indent();
        self.writer.line("end");
    }

    fn emit_load(&mut self, kind: LoadKind, arg: MemArg) -> Result<(), TranslationProblemReason> {
        let address = self.pop_value();
        let read_expression = match kind {
            LoadKind::I32 { atomic: false }
            | LoadKind::I64_32 {
                kind: ExtendedLoad::SignExtend,
            } => "readi32",
            LoadKind::I64 { atomic: false } | LoadKind::F64 => "readf64",
            LoadKind::F32 => "readf32",
            LoadKind::I32_8 {
                kind: ExtendedLoad::ZeroExtend,
            }
            | LoadKind::I64_8 {
                kind: ExtendedLoad::ZeroExtend,
            } => "readu8",
            LoadKind::I32_8 {
                kind: ExtendedLoad::SignExtend,
            }
            | LoadKind::I64_8 {
                kind: ExtendedLoad::SignExtend,
            } => "readi8",
            LoadKind::I32_16 {
                kind: ExtendedLoad::ZeroExtend,
            }
            | LoadKind::I64_16 {
                kind: ExtendedLoad::ZeroExtend,
            } => "readu16",
            LoadKind::I32_16 {
                kind: ExtendedLoad::SignExtend,
            }
            | LoadKind::I64_16 {
                kind: ExtendedLoad::SignExtend,
            } => "readi16",
            LoadKind::I64_32 {
                kind: ExtendedLoad::ZeroExtend,
            } => "readu32",
            unsupported => {
                return Err(TranslationProblemReason::UnsupportedInstruction {
                    instruction: format!("load {unsupported:?}"),
                });
            }
        };
        self.emit_push(&format!(
            "buffer.{read_expression}(MEMORY, {address} + {})",
            arg.offset
        ));
        Ok(())
    }

    fn emit_store(&mut self, kind: StoreKind, arg: MemArg) -> Result<(), TranslationProblemReason> {
        let value = self.pop_value();
        let address = self.pop_value();
        let write_expression = match kind {
            StoreKind::I32 { atomic: false } | StoreKind::I64_32 { atomic: false } => "writei32",
            StoreKind::I64 { atomic: false } | StoreKind::F64 => "writef64",
            StoreKind::F32 => "writef32",
            StoreKind::I32_8 { atomic: false } | StoreKind::I64_8 { atomic: false } => "writei8",
            StoreKind::I32_16 { atomic: false } | StoreKind::I64_16 { atomic: false } => "writei16",
            unsupported => {
                return Err(TranslationProblemReason::UnsupportedInstruction {
                    instruction: format!("store {unsupported:?}"),
                });
            }
        };
        self.writer.line(&format!(
            "buffer.{write_expression}(MEMORY, {address} + {}, {value})",
            arg.offset
        ));
        Ok(())
    }

    fn next_temp(&mut self) -> String {
        let temp = format!("t{}", self.next_temp_id.0);
        self.next_temp_id = TempId(self.next_temp_id.0 + 1);
        temp
    }
}

fn construct_flag_name(construct_id: ConstructId) -> String {
    format!("exit_{}", construct_id.0)
}

fn construct_restart_name(construct_id: ConstructId) -> String {
    format!("restart_{}", construct_id.0)
}
