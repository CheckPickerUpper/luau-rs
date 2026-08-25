//! Per-function wasm-to-Luau translation.
//!
//! One generated Luau function keeps a shared value stack (`stack` plus an
//! explicit `sp`) and lowers wasm structured control flow (blocks, loops, ifs)
//! to nested `while true` loops with per-construct exit/restart flags, so `br`
//! can break out of any enclosing construct with correct Luau semantics.

use crate::wasm::{DecodedFunction, WasmDecodeProblemReason, WasmValueType};
use std::collections::HashSet;
use walrus::ir::{ExtendedLoad, Instr, InstrSeqId, LoadKind, MemArg, StoreKind, Value};
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

/// The two scalar slots used for one i64 value.
#[derive(Debug, Clone)]
struct ValueParts {
    low: String,
    high: String,
}

impl ValueParts {
    const fn scalar(value: String) -> Self {
        Self {
            low: value,
            high: String::new(),
        }
    }
}

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
            "FUNC_{} = function({}){}",
            input.function.index(),
            parameters.join(", "),
            comment_name
        ));
    } else {
        writer.line(&format!(
            "FUNC_{} = function({}): {}{}",
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
#[must_use]
pub fn function_signature(function: &DecodedFunction) -> (Vec<String>, String) {
    let parameters = expanded_parameters(function.params());
    let return_types = expanded_types(function.results())
        .into_iter()
        .map(WasmValueType::luau_type_name)
        .collect::<Vec<_>>();
    let return_annotation = match return_types.as_slice() {
        [] => String::new(),
        [single] => (*single).to_owned(),
        _ => format!("({})", return_types.join(", ")),
    };
    (parameters, return_annotation)
}

fn expanded_parameters(types: &[WasmValueType]) -> Vec<String> {
    let mut parameters = Vec::new();
    for (index, value_type) in types.iter().enumerate() {
        if *value_type == WasmValueType::I64 {
            parameters.push(format!("p{index}_lo: number"));
            parameters.push(format!("p{index}_hi: number"));
        } else {
            parameters.push(format!("p{index}: {}", value_type.luau_type_name()));
        }
    }
    parameters
}

fn expanded_types(types: &[WasmValueType]) -> Vec<WasmValueType> {
    types
        .iter()
        .flat_map(|value_type| {
            if *value_type == WasmValueType::I64 {
                vec![WasmValueType::I32, WasmValueType::I32]
            } else {
                vec![*value_type]
            }
        })
        .collect()
}

fn wasm_types(types: &[walrus::ValType]) -> Result<Vec<WasmValueType>, TranslationProblemReason> {
    types
        .iter()
        .copied()
        .map(|value_type| {
            WasmValueType::try_from(value_type)
                .map_err(|reason| TranslationProblemReason::Internal(reason.to_string()))
        })
        .collect()
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
            let local_name = self.local_base_name_at(local_index);
            if local_type == WasmValueType::I64 {
                self.writer
                    .line(&format!("local {local_name}_lo: number = 0"));
                self.writer
                    .line(&format!("local {local_name}_hi: number = 0"));
            } else {
                self.writer.line(&format!(
                    "local {local_name}: {} = {}",
                    local_type.luau_type_name(),
                    zero_initializer(local_type)
                ));
            }
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
                match constant.value {
                    Value::I64(value) => {
                        let (low, high) = super::ops::luau_i64_parts(value);
                        self.emit_push_i64(&low, &high);
                    }
                    value => self.emit_push(&luau_constant(value)?),
                }
                Ok(())
            }
            Instr::LocalGet(local_get) => {
                let parts = self.local_parts(local_get.local)?;
                self.emit_value_parts(&parts);
                Ok(())
            }
            Instr::LocalSet(local_set) => {
                let parts = self.local_parts(local_set.local)?;
                self.emit_pop_into_parts(&parts);
                Ok(())
            }
            Instr::LocalTee(local_tee) => {
                let parts = self.local_parts(local_tee.local)?;
                self.emit_tee_into_parts(&parts);
                Ok(())
            }
            Instr::GlobalGet(global_get) => {
                let parts = self.global_parts(global_get.global)?;
                self.emit_value_parts(&parts);
                Ok(())
            }
            Instr::GlobalSet(global_set) => {
                let parts = self.global_parts(global_set.global)?;
                self.emit_pop_into_parts(&parts);
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
                self.emit_call(call.func.index())?;
                Ok(())
            }
            Instr::CallIndirect(call_indirect) => {
                self.emit_call_indirect(call_indirect.ty, call_indirect.table)?;
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
            Instr::MemoryCopy(_) => {
                self.emit_memory_copy();
                Ok(())
            }
            Instr::MemoryFill(_) => {
                self.emit_memory_fill();
                Ok(())
            }
            Instr::MemoryInit(memory_init) => {
                self.emit_memory_init(memory_init.data.index());
                Ok(())
            }
            Instr::DataDrop(_) => Ok(()),
            Instr::RefFunc(ref_func) => {
                self.emit_push(&format!("FUNC_{}", ref_func.func.index()));
                Ok(())
            }
            Instr::RefNull(_) => {
                self.emit_push("nil");
                Ok(())
            }
            Instr::RefIsNull(_) => {
                let operand = self.pop_value();
                self.emit_push(&format!("({operand} == nil)"));
                Ok(())
            }
            Instr::TableGet(_) => {
                let index = self.pop_value();
                self.emit_push(&format!("FUNCTIONS[{index} + {LUAU_INDEX_OFFSET}]"));
                Ok(())
            }
            Instr::TableSet(_) => {
                let value = self.pop_value();
                let index = self.pop_value();
                self.writer.line(&format!(
                    "FUNCTIONS[{index} + {LUAU_INDEX_OFFSET}] = {value}"
                ));
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
        if self.unreachable {
            return;
        }
        let result_types = self.input.function.results();
        let slot_count = result_types
            .iter()
            .map(|value_type| {
                if *value_type == WasmValueType::I64 {
                    2
                } else {
                    1
                }
            })
            .sum::<usize>();
        if slot_count == 0 {
            return;
        }
        let values = (0..slot_count)
            .map(|offset| {
                let position = slot_count - offset;
                format!("{STACK_NAME}[{SP_NAME} - {}]", position - 1)
            })
            .collect::<Vec<_>>()
            .join(", ");
        self.writer.line(&format!("return {values}"));
    }

    fn emit_return(&mut self) {
        self.emit_final_return();
    }

    fn local_parts(&self, local: walrus::LocalId) -> Result<ValueParts, TranslationProblemReason> {
        let Some(position) = self
            .local_positions
            .iter()
            .position(|candidate| *candidate == local)
        else {
            return Err(TranslationProblemReason::Internal(format!(
                "unknown local {local:?}"
            )));
        };
        let local_type = self.local_type_at(position)?;
        let local_name = self.local_base_name_at(position);
        if local_type == WasmValueType::I64 {
            return Ok(ValueParts {
                low: format!("{local_name}_lo"),
                high: format!("{local_name}_hi"),
            });
        }
        Ok(ValueParts::scalar(local_name))
    }

    fn local_base_name_at(&self, position: usize) -> String {
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

    fn global_parts(
        &self,
        global: walrus::GlobalId,
    ) -> Result<ValueParts, TranslationProblemReason> {
        let global_data = self.input.module.globals.get(global);
        let value_type = WasmValueType::try_from(global_data.ty)
            .map_err(|reason| TranslationProblemReason::Internal(reason.to_string()))?;
        let mut slot = 0;
        for candidate in self.input.module.globals.iter() {
            if candidate.id() == global {
                break;
            }
            slot += if candidate.ty == walrus::ValType::I64 {
                2
            } else {
                1
            };
        }
        let index = slot + LUAU_INDEX_OFFSET;
        if value_type == WasmValueType::I64 {
            return Ok(ValueParts {
                low: format!("GLOBALS[{index}]"),
                high: format!("GLOBALS[{}]", index + 1),
            });
        }
        Ok(ValueParts::scalar(format!("GLOBALS[{index}]")))
    }

    fn emit_push(&mut self, value_expression: &str) {
        self.writer.line(&format!("{SP_NAME} += 1"));
        self.writer
            .line(&format!("{STACK_NAME}[{SP_NAME}] = {value_expression}"));
    }

    fn emit_push_i64(&mut self, low: &str, high: &str) {
        self.writer.line(&format!("{SP_NAME} += 2"));
        self.writer
            .line(&format!("{STACK_NAME}[{SP_NAME} - 1] = {low}"));
        self.writer
            .line(&format!("{STACK_NAME}[{SP_NAME}] = {high}"));
    }

    fn emit_value_parts(&mut self, parts: &ValueParts) {
        if parts.high.is_empty() {
            self.emit_push(&parts.low);
        } else {
            self.emit_push_i64(&parts.low, &parts.high);
        }
    }

    /// Pops one value into a fresh temporary and returns its name.
    fn pop_value(&mut self) -> String {
        let temp = self.next_temp();
        self.writer
            .line(&format!("local {temp} = {STACK_NAME}[{SP_NAME}]"));
        self.writer.line(&format!("{SP_NAME} -= 1"));
        temp
    }

    fn pop_i64(&mut self) -> (String, String) {
        let high = self.pop_value();
        let low = self.pop_value();
        (low, high)
    }

    fn emit_pop_into(&mut self, target: &str) {
        let value = self.pop_value();
        self.writer.line(&format!("{target} = {value}"));
    }

    fn emit_pop_into_parts(&mut self, target: &ValueParts) {
        if target.high.is_empty() {
            self.emit_pop_into(&target.low);
        } else {
            let (low, high) = self.pop_i64();
            self.writer.line(&format!("{} = {low}", target.low));
            self.writer.line(&format!("{} = {high}", target.high));
        }
    }

    fn emit_tee_into(&mut self, target: &str) {
        let temp = self.next_temp();
        self.writer
            .line(&format!("local {temp} = {STACK_NAME}[{SP_NAME}]"));
        self.writer.line(&format!("{target} = {temp}"));
    }

    fn emit_tee_into_parts(&mut self, target: &ValueParts) {
        if target.high.is_empty() {
            self.emit_tee_into(&target.low);
        } else {
            let high = self.next_temp();
            let low = self.next_temp();
            self.writer
                .line(&format!("local {high} = {STACK_NAME}[{SP_NAME}]"));
            self.writer
                .line(&format!("local {low} = {STACK_NAME}[{SP_NAME} - 1]"));
            self.writer.line(&format!("{} = {low}", target.low));
            self.writer.line(&format!("{} = {high}", target.high));
        }
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
        match op {
            walrus::ir::UnaryOp::I64Eqz => {
                let (low, high) = self.pop_i64();
                self.emit_push(&format!("wasm_i64_eqz({low}, {high})"));
            }
            walrus::ir::UnaryOp::I32WrapI64 => {
                let (low, _) = self.pop_i64();
                self.emit_push(&format!("wasm_i32_wrap({low})"));
            }
            walrus::ir::UnaryOp::I64ExtendSI32 => {
                let operand = self.pop_value();
                self.emit_i64_call(&format!("wasm_i64_from_i32s({operand})"));
            }
            walrus::ir::UnaryOp::I64ExtendUI32 => {
                let operand = self.pop_value();
                self.emit_i64_call(&format!("wasm_i64_from_i32u({operand})"));
            }
            walrus::ir::UnaryOp::I64Extend8S => {
                let (low, _) = self.pop_i64();
                self.emit_i64_call(&format!("wasm_i64_extend8s({low})"));
            }
            walrus::ir::UnaryOp::I64Extend16S => {
                let (low, _) = self.pop_i64();
                self.emit_i64_call(&format!("wasm_i64_extend16s({low})"));
            }
            walrus::ir::UnaryOp::I64Extend32S => {
                let (low, _) = self.pop_i64();
                self.emit_i64_call(&format!("wasm_i64_extend32s({low})"));
            }
            walrus::ir::UnaryOp::F32ConvertSI64 | walrus::ir::UnaryOp::F64ConvertSI64 => {
                let (low, high) = self.pop_i64();
                self.emit_push(&format!("wasm_i64_to_f64s({low}, {high})"));
            }
            walrus::ir::UnaryOp::F32ConvertUI64 | walrus::ir::UnaryOp::F64ConvertUI64 => {
                let (low, high) = self.pop_i64();
                self.emit_push(&format!("wasm_i64_to_f64u({low}, {high})"));
            }
            walrus::ir::UnaryOp::I64TruncSF32 | walrus::ir::UnaryOp::I64TruncSF64 => {
                let operand = self.pop_value();
                self.emit_i64_call(&format!("wasm_i64_truncs_pair({operand})"));
            }
            walrus::ir::UnaryOp::I64TruncUF32 | walrus::ir::UnaryOp::I64TruncUF64 => {
                let operand = self.pop_value();
                self.emit_i64_call(&format!("wasm_i64_truncu_pair({operand})"));
            }
            walrus::ir::UnaryOp::I64ReinterpretF64 => {
                let operand = self.pop_value();
                self.emit_i64_call(&format!("wasm_reinterpret_i64_f64_pair({operand})"));
            }
            walrus::ir::UnaryOp::F64ReinterpretI64 => {
                let (low, high) = self.pop_i64();
                self.emit_push(&format!("wasm_reinterpret_f64_i64_pair({low}, {high})"));
            }
            op => {
                let operand = self.pop_value();
                self.emit_push(&unop_expression(op, &operand)?);
            }
        }
        Ok(())
    }

    fn emit_binop(&mut self, op: walrus::ir::BinaryOp) -> Result<(), TranslationProblemReason> {
        let i64_helper = match op {
            walrus::ir::BinaryOp::I64Eq => Some(("wasm_i64_eq", false)),
            walrus::ir::BinaryOp::I64Ne => Some(("wasm_i64_ne", false)),
            walrus::ir::BinaryOp::I64LtS => Some(("wasm_i64_lt_s", false)),
            walrus::ir::BinaryOp::I64LtU => Some(("wasm_i64_lt_u", false)),
            walrus::ir::BinaryOp::I64GtS => Some(("wasm_i64_gt_s", false)),
            walrus::ir::BinaryOp::I64GtU => Some(("wasm_i64_gt_u", false)),
            walrus::ir::BinaryOp::I64LeS => Some(("wasm_i64_le_s", false)),
            walrus::ir::BinaryOp::I64LeU => Some(("wasm_i64_le_u", false)),
            walrus::ir::BinaryOp::I64GeS => Some(("wasm_i64_ge_s", false)),
            walrus::ir::BinaryOp::I64GeU => Some(("wasm_i64_ge_u", false)),
            walrus::ir::BinaryOp::I64Add => Some(("wasm_i64_add", true)),
            walrus::ir::BinaryOp::I64Sub => Some(("wasm_i64_sub", true)),
            walrus::ir::BinaryOp::I64Mul => Some(("wasm_i64_mul", true)),
            walrus::ir::BinaryOp::I64DivS => Some(("wasm_i64_div_s", true)),
            walrus::ir::BinaryOp::I64DivU => Some(("wasm_i64_div_u", true)),
            walrus::ir::BinaryOp::I64RemS => Some(("wasm_i64_rem_s", true)),
            walrus::ir::BinaryOp::I64RemU => Some(("wasm_i64_rem_u", true)),
            walrus::ir::BinaryOp::I64And => Some(("wasm_i64_and", true)),
            walrus::ir::BinaryOp::I64Or => Some(("wasm_i64_or", true)),
            walrus::ir::BinaryOp::I64Xor => Some(("wasm_i64_xor", true)),
            walrus::ir::BinaryOp::I64Shl => Some(("wasm_i64_shl", true)),
            walrus::ir::BinaryOp::I64ShrS => Some(("wasm_i64_shr_s", true)),
            walrus::ir::BinaryOp::I64ShrU => Some(("wasm_i64_shr_u", true)),
            walrus::ir::BinaryOp::I64Rotl => Some(("wasm_i64_rotl", true)),
            walrus::ir::BinaryOp::I64Rotr => Some(("wasm_i64_rotr", true)),
            _ => None,
        };
        if let Some((helper, returns_i64)) = i64_helper {
            let (right_low, right_high) = self.pop_i64();
            let (left_low, left_high) = self.pop_i64();
            let expression = if matches!(
                op,
                walrus::ir::BinaryOp::I64Shl
                    | walrus::ir::BinaryOp::I64ShrS
                    | walrus::ir::BinaryOp::I64ShrU
                    | walrus::ir::BinaryOp::I64Rotl
                    | walrus::ir::BinaryOp::I64Rotr
            ) {
                format!("{helper}({left_low}, {left_high}, {right_low})")
            } else {
                format!("{helper}({left_low}, {left_high}, {right_low}, {right_high})")
            };
            if returns_i64 {
                self.emit_i64_call(&expression);
            } else {
                self.emit_push(&expression);
            }
            return Ok(());
        }
        let right = self.pop_value();
        let left = self.pop_value();
        self.emit_push(&binop_expression(op, &left, &right)?);
        Ok(())
    }

    fn emit_i64_call(&mut self, expression: &str) {
        let low = self.next_temp();
        let high = self.next_temp();
        self.writer
            .line(&format!("local {low}, {high} = {expression}"));
        self.emit_push_i64(&low, &high);
    }

    fn emit_call(&mut self, function_index: usize) -> Result<(), TranslationProblemReason> {
        let (parameter_types, result_types) = self
            .input
            .module
            .funcs
            .iter()
            .find(|function| function.id().index() == function_index)
            .map(|function| {
                let function_type = self.input.module.types.get(function.ty());
                (
                    function_type.params().to_vec(),
                    function_type.results().to_vec(),
                )
            })
            .ok_or_else(|| {
                TranslationProblemReason::Internal(format!(
                    "call targets unknown function {function_index}"
                ))
            })?;
        let parameter_types = wasm_types(&parameter_types)?;
        let result_types = wasm_types(&result_types)?;
        let arguments = self.pop_arguments(&parameter_types);
        self.emit_call_results(
            &result_types,
            &format!("FUNC_{function_index}({})", arguments.join(", ")),
        );
        Ok(())
    }

    fn emit_call_indirect(
        &mut self,
        function_type: walrus::TypeId,
        _table: walrus::TableId,
    ) -> Result<(), TranslationProblemReason> {
        let table_index = self.pop_value();
        let function_type = self.input.module.types.get(function_type);
        let parameter_types = wasm_types(function_type.params())?;
        let result_types = wasm_types(function_type.results())?;
        let arguments = self.pop_arguments(&parameter_types);
        self.emit_call_results(
            &result_types,
            &format!(
                "FUNCTIONS[{table_index} + {LUAU_INDEX_OFFSET}]({})",
                arguments.join(", ")
            ),
        );
        Ok(())
    }

    fn emit_call_results(&mut self, result_types: &[WasmValueType], expression: &str) {
        if result_types.is_empty() {
            self.writer.line(expression);
            return;
        }
        let expanded_result_count = result_types
            .iter()
            .map(|value_type| {
                if *value_type == WasmValueType::I64 {
                    2
                } else {
                    1
                }
            })
            .sum::<usize>();
        let temporaries = (0..expanded_result_count)
            .map(|_| self.next_temp())
            .collect::<Vec<_>>();
        self.writer
            .line(&format!("local {} = {expression}", temporaries.join(", ")));
        let mut offset = 0;
        for value_type in result_types {
            if *value_type == WasmValueType::I64 {
                self.emit_push_i64(&temporaries[offset], &temporaries[offset + 1]);
                offset += 2;
            } else {
                self.emit_push(&temporaries[offset]);
                offset += 1;
            }
        }
    }

    /// Pops call arguments (last argument on top) and returns them in order.
    fn pop_arguments(&mut self, types: &[WasmValueType]) -> Vec<String> {
        let mut groups = Vec::with_capacity(types.len());
        for value_type in types.iter().rev() {
            if *value_type == WasmValueType::I64 {
                let (low, high) = self.pop_i64();
                groups.push(vec![low, high]);
            } else {
                groups.push(vec![self.pop_value()]);
            }
        }
        groups.reverse();
        groups.into_iter().flatten().collect()
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

    /// Lowers `memory.copy`: `buffer.copy(MEMORY, dest, MEMORY, src, len)`.
    fn emit_memory_copy(&mut self) {
        let length = self.pop_value();
        let source = self.pop_value();
        let destination = self.pop_value();
        self.writer.line(&format!(
            "buffer.copy(MEMORY, {destination}, MEMORY, {source}, {length})"
        ));
    }

    /// Lowers `memory.fill`: `buffer.fill(MEMORY, dest, value, len)`.
    fn emit_memory_fill(&mut self) {
        let length = self.pop_value();
        let value = self.pop_value();
        let destination = self.pop_value();
        self.writer.line(&format!(
            "buffer.fill(MEMORY, {destination}, {value}, {length})"
        ));
    }

    /// Lowers `memory.init`: copy from the segment's passive buffer.
    fn emit_memory_init(&mut self, data_index: usize) {
        let length = self.pop_value();
        let source = self.pop_value();
        let destination = self.pop_value();
        self.writer.line(&format!(
            "buffer.copy(MEMORY, {destination}, DATA_{data_index}, {source}, {length})"
        ));
    }

    fn emit_load(&mut self, kind: LoadKind, arg: MemArg) -> Result<(), TranslationProblemReason> {
        let address = self.pop_value();
        let offset = arg.offset;
        match kind {
            LoadKind::I64 { atomic: false } => {
                let low = self.next_temp();
                let high = self.next_temp();
                self.writer.line(&format!(
                    "local {low} = buffer.readu32(MEMORY, {address} + {offset})"
                ));
                self.writer.line(&format!(
                    "local {high} = buffer.readu32(MEMORY, {address} + {})",
                    offset + 4
                ));
                self.emit_push_i64(&low, &high);
            }
            LoadKind::I64_8 {
                kind: ExtendedLoad::SignExtend,
            } => {
                self.emit_i64_call(&format!(
                    "wasm_i64_from_i32s(buffer.readi8(MEMORY, {address} + {offset}))"
                ));
            }
            LoadKind::I64_8 {
                kind: ExtendedLoad::ZeroExtend,
            } => {
                self.emit_i64_call(&format!(
                    "wasm_i64_from_i32u(buffer.readu8(MEMORY, {address} + {offset}))"
                ));
            }
            LoadKind::I64_16 {
                kind: ExtendedLoad::SignExtend,
            } => {
                self.emit_i64_call(&format!(
                    "wasm_i64_from_i32s(buffer.readi16(MEMORY, {address} + {offset}))"
                ));
            }
            LoadKind::I64_16 {
                kind: ExtendedLoad::ZeroExtend,
            } => {
                self.emit_i64_call(&format!(
                    "wasm_i64_from_i32u(buffer.readu16(MEMORY, {address} + {offset}))"
                ));
            }
            LoadKind::I64_32 {
                kind: ExtendedLoad::SignExtend,
            } => {
                self.emit_i64_call(&format!(
                    "wasm_i64_from_i32s(buffer.readi32(MEMORY, {address} + {offset}))"
                ));
            }
            LoadKind::I64_32 {
                kind: ExtendedLoad::ZeroExtend,
            } => {
                self.emit_i64_call(&format!(
                    "wasm_i64_from_i32u(buffer.readu32(MEMORY, {address} + {offset}))"
                ));
            }
            LoadKind::I32 { atomic: false } => {
                self.emit_push(&format!("buffer.readi32(MEMORY, {address} + {offset})"));
            }
            LoadKind::F64 => {
                self.emit_push(&format!("buffer.readf64(MEMORY, {address} + {offset})"));
            }
            LoadKind::F32 => {
                self.emit_push(&format!("buffer.readf32(MEMORY, {address} + {offset})"));
            }
            LoadKind::I32_8 {
                kind: ExtendedLoad::ZeroExtend,
            } => self.emit_push(&format!("buffer.readu8(MEMORY, {address} + {offset})")),
            LoadKind::I32_8 {
                kind: ExtendedLoad::SignExtend,
            } => self.emit_push(&format!("buffer.readi8(MEMORY, {address} + {offset})")),
            LoadKind::I32_16 {
                kind: ExtendedLoad::ZeroExtend,
            } => self.emit_push(&format!("buffer.readu16(MEMORY, {address} + {offset})")),
            LoadKind::I32_16 {
                kind: ExtendedLoad::SignExtend,
            } => self.emit_push(&format!("buffer.readi16(MEMORY, {address} + {offset})")),
            unsupported => {
                return Err(TranslationProblemReason::UnsupportedInstruction {
                    instruction: format!("load {unsupported:?}"),
                });
            }
        }
        Ok(())
    }

    fn emit_store(&mut self, kind: StoreKind, arg: MemArg) -> Result<(), TranslationProblemReason> {
        let offset = arg.offset;
        match kind {
            StoreKind::I64 { atomic: false } => {
                let high = self.pop_value();
                let low = self.pop_value();
                let address = self.pop_value();
                self.writer.line(&format!(
                    "buffer.writeu32(MEMORY, {address} + {offset}, {low})"
                ));
                self.writer.line(&format!(
                    "buffer.writeu32(MEMORY, {address} + {}, {high})",
                    offset + 4
                ));
            }
            StoreKind::I64_8 { atomic: false } => {
                let high = self.pop_value();
                let low = self.pop_value();
                let address = self.pop_value();
                let _ = high;
                self.writer.line(&format!(
                    "buffer.writeu8(MEMORY, {address} + {offset}, {low})"
                ));
            }
            StoreKind::I64_16 { atomic: false } => {
                let high = self.pop_value();
                let low = self.pop_value();
                let address = self.pop_value();
                let _ = high;
                self.writer.line(&format!(
                    "buffer.writeu16(MEMORY, {address} + {offset}, {low})"
                ));
            }
            StoreKind::I64_32 { atomic: false } => {
                let high = self.pop_value();
                let low = self.pop_value();
                let address = self.pop_value();
                let _ = high;
                self.writer.line(&format!(
                    "buffer.writeu32(MEMORY, {address} + {offset}, {low})"
                ));
            }
            StoreKind::I32 { atomic: false } => {
                let value = self.pop_value();
                let address = self.pop_value();
                self.writer.line(&format!(
                    "buffer.writei32(MEMORY, {address} + {offset}, {value})"
                ));
            }
            StoreKind::F64 => {
                let value = self.pop_value();
                let address = self.pop_value();
                self.writer.line(&format!(
                    "buffer.writef64(MEMORY, {address} + {offset}, {value})"
                ));
            }
            StoreKind::F32 => {
                let value = self.pop_value();
                let address = self.pop_value();
                self.writer.line(&format!(
                    "buffer.writef32(MEMORY, {address} + {offset}, {value})"
                ));
            }
            StoreKind::I32_8 { atomic: false } => {
                let value = self.pop_value();
                let address = self.pop_value();
                self.writer.line(&format!(
                    "buffer.writei8(MEMORY, {address} + {offset}, {value})"
                ));
            }
            StoreKind::I32_16 { atomic: false } => {
                let value = self.pop_value();
                let address = self.pop_value();
                self.writer.line(&format!(
                    "buffer.writei16(MEMORY, {address} + {offset}, {value})"
                ));
            }
            unsupported => {
                return Err(TranslationProblemReason::UnsupportedInstruction {
                    instruction: format!("store {unsupported:?}"),
                });
            }
        }
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
