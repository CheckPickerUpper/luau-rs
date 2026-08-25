//! Decoding and validation of wasm modules before Luau translation.

mod problem;

pub use problem::{WasmDecodeProblemReason, WasmDecodeRejection};

use walrus::ir::Value;
use walrus::{
    ConstExpr, DataKind, ElementItems, ElementKind, ExportItem, FunctionKind, GlobalKind,
    ImportKind, Module, ValType,
};

/// The maximum number of linear memories the backend can model (one).
const MAX_MEMORIES: usize = 1;

/// The outcome of decoding one wasm module.
#[derive(Debug)]
pub enum DecodeOutcome {
    /// The module decoded into a form the Luau backend can translate.
    Decoded(Box<DecodedModule>),
    /// The module was rejected with at least one typed reason.
    Rejected(WasmDecodeRejection),
}

/// A validated subset of a wasm module, owned and independent of the upstream
/// parser so the Luau backend never depends on `walrus` internals.
#[derive(Debug)]
pub struct DecodedModule {
    /// The upstream parsed module, kept so the backend can walk function bodies.
    walrus_module: Module,
    /// Every defined and imported function, ordered by wasm function index.
    functions: Vec<DecodedFunction>,
    /// Exported names mapped to their source (function index or memory).
    exports: Vec<DecodedExport>,
    /// Function imports: `(module, name, function index)`.
    imports: Vec<DecodedImport>,
    /// Globals in wasm index order, with their initial values.
    globals: Vec<DecodedGlobal>,
    /// Linear memory parameters, present only when the module declares memory.
    memory: Option<DecodedMemory>,
    /// Active data segments in memory order.
    data_segments: Vec<DecodedDataSegment>,
    /// Active function-index element segments.
    element_segments: Vec<DecodedElementSegment>,
    /// The start function, when the module declares one.
    start_function: StartFunctionPresence,
}

impl DecodedModule {
    /// @why Lets the backend walk function bodies through the upstream IR.
    #[must_use]
    pub const fn walrus_module(&self) -> &Module {
        &self.walrus_module
    }

    /// @why Lets the backend emit one Luau function per wasm function in index order.
    #[must_use]
    pub fn functions(&self) -> &[DecodedFunction] {
        &self.functions
    }

    /// @why Lets the backend assemble the exported surface and call the entrypoint.
    #[must_use]
    pub fn exports(&self) -> &[DecodedExport] {
        &self.exports
    }

    /// @why Lets the backend route imported calls through the instantiation seam.
    #[must_use]
    pub fn imports(&self) -> &[DecodedImport] {
        &self.imports
    }

    /// @why Lets the backend allocate and update the module's globals table.
    #[must_use]
    pub fn globals(&self) -> &[DecodedGlobal] {
        &self.globals
    }

    /// @why Lets the backend allocate the module's linear memory.
    #[must_use]
    pub const fn memory(&self) -> Option<&DecodedMemory> {
        self.memory.as_ref()
    }

    /// @why Lets the backend initialize data segments after allocating memory.
    #[must_use]
    pub fn data_segments(&self) -> &[DecodedDataSegment] {
        &self.data_segments
    }

    /// @why Lets the backend populate the indirect-call table before any function runs.
    #[must_use]
    pub fn element_segments(&self) -> &[DecodedElementSegment] {
        &self.element_segments
    }

    /// @why Lets the backend invoke the module start routine after initialization.
    #[must_use]
    pub const fn start_function(&self) -> &StartFunctionPresence {
        &self.start_function
    }
}

/// Whether the module declares a start function and which function it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartFunctionPresence {
    /// The module runs this function index during instantiation.
    Declared {
        /// The wasm function index invoked during instantiation.
        function_index: usize,
    },
    /// The module has no start function.
    Absent,
}

impl StartFunctionPresence {
    /// @why Lets the backend decide whether to emit the start invocation.
    #[must_use]
    pub const fn function_index(&self) -> Option<usize> {
        match self {
            Self::Declared { function_index } => Some(*function_index),
            Self::Absent => None,
        }
    }
}

/// One wasm function together with its validated body entry point.
#[derive(Debug)]
pub struct DecodedFunction {
    /// The wasm function index (includes imported functions).
    index: usize,
    /// The debug name, when the wasm module carries one.
    name: Option<String>,
    /// Parameter value types.
    params: Vec<WasmValueType>,
    /// Result value types.
    results: Vec<WasmValueType>,
    /// The number of declared locals beyond the parameters.
    local_count: usize,
    /// Whether the function is imported or defined with a translatable body.
    body: DecodedFunctionBody,
}

/// The two legal function forms: imported (a Luau callback) or defined (a
/// body to translate), so no combination of absent/present fields is possible.
#[derive(Debug, Clone, Copy)]
pub enum DecodedFunctionBody {
    /// An imported function with no wasm body.
    Imported,
    /// A defined function whose body starts at this instruction sequence.
    Defined {
        /// The instruction sequence that begins the translatable body.
        entry_sequence: walrus::ir::InstrSeqId,
    },
}

impl DecodedFunction {
    /// @why Gives the backend a stable, parser-independent function identity.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// @why Lets diagnostics and generated comments name functions when a debug name exists.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// @why Lets the backend type the generated Luau function parameters.
    #[must_use]
    pub fn params(&self) -> &[WasmValueType] {
        &self.params
    }

    /// @why Lets the backend type the generated Luau function result.
    #[must_use]
    pub fn results(&self) -> &[WasmValueType] {
        &self.results
    }

    /// @why Lets the backend know how many local slots must be reserved.
    #[must_use]
    pub const fn local_count(&self) -> usize {
        self.local_count
    }

    /// @why Lets the backend distinguish import proxies from translatable bodies.
    #[must_use]
    pub const fn body(&self) -> &DecodedFunctionBody {
        &self.body
    }
}

/// A wasm value type restricted to the subset the Luau backend models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmValueType {
    /// 32-bit integer, held in a Luau `number`.
    I32,
    /// 64-bit integer, held in a Luau `number` with documented precision limits.
    I64,
    /// 32-bit float, held in a Luau `number`.
    F32,
    /// 64-bit float, held in a Luau `number`.
    F64,
    /// An opaque host reference (a Roblox object), held as `any`.
    ExternRef,
    /// A function reference, held as `any`.
    FuncRef,
}

impl WasmValueType {
    /// @why Lets the backend and the generated module agree on a Luau type name.
    #[must_use]
    pub const fn luau_type_name(self) -> &'static str {
        match self {
            Self::I32 | Self::I64 | Self::F32 | Self::F64 => "number",
            Self::ExternRef | Self::FuncRef => "any",
        }
    }
}

impl TryFrom<ValType> for WasmValueType {
    type Error = WasmDecodeProblemReason;

    fn try_from(value_type: ValType) -> Result<Self, Self::Error> {
        match value_type {
            ValType::I32 => Ok(Self::I32),
            ValType::I64 => Ok(Self::I64),
            ValType::F32 => Ok(Self::F32),
            ValType::F64 => Ok(Self::F64),
            ValType::Ref(ref_type) if ref_type == walrus::RefType::EXTERNREF => Ok(Self::ExternRef),
            ValType::Ref(ref_type) if ref_type == walrus::RefType::FUNCREF => Ok(Self::FuncRef),
            ValType::Ref(_) => Err(WasmDecodeProblemReason::UnsupportedInstruction {
                instruction: "non-extern, non-func reference type".into(),
            }),
            // Vector values have no Luau representation; the surface pass
            // rejects them before translation, with this as the named failure.
            ValType::V128 => Err(WasmDecodeProblemReason::UnsupportedVectorType),
        }
    }
}

/// An exported name mapped to its module-local target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodedExport {
    /// Exported function.
    Function {
        /// The exported name callers use to reach the function.
        name: String,
        /// The wasm function index behind the export.
        function_index: usize,
    },
    /// Exported memory.
    Memory {
        /// The exported name callers use to reach the linear memory.
        name: String,
    },
}

impl DecodedExport {
    /// @why Lets the backend emit the exported function surface.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::Function { name, .. } | Self::Memory { name } => name,
        }
    }
}

/// A function import routed through the instantiation seam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedImport {
    /// The import module namespace (for example `"env"` or `"roblox"`).
    module: String,
    /// The import name within that namespace.
    name: String,
    /// The wasm function index assigned to this import.
    function_index: usize,
}

impl DecodedImport {
    /// @why Lets the backend route the call through `imports[module][name]`.
    #[must_use]
    pub fn module(&self) -> &str {
        &self.module
    }

    /// @why Lets the backend route the call through `imports[module][name]`.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// @why Lets the backend map an imported call back to its proxy.
    #[must_use]
    pub const fn function_index(&self) -> usize {
        self.function_index
    }
}

/// The initial value of a decoded global, kept exactly as decoded.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DecodedGlobalValue {
    /// A 32-bit integer initializer.
    I32(i32),
    /// A 64-bit integer initializer.
    I64(i64),
    /// A 32-bit float initializer.
    F32(f32),
    /// A 64-bit float initializer.
    F64(f64),
    /// A null reference initializer (externref or funcref), held as `nil`.
    NullReference,
}

/// A decoded global with its type, mutability, and constant initializer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecodedGlobal {
    /// The value type of this global.
    value_type: WasmValueType,
    /// Whether `global.set` is permitted on this global.
    mutable: bool,
    /// The constant initializer value.
    initial_value: DecodedGlobalValue,
}

impl DecodedGlobal {
    /// @why Lets the backend type the globals table entry.
    #[must_use]
    pub const fn value_type(&self) -> WasmValueType {
        self.value_type
    }

    /// @why Lets the backend allow or reject `global.set` against this global.
    #[must_use]
    pub const fn is_mutable(&self) -> bool {
        self.mutable
    }

    /// @why Lets the backend seed the globals table at instantiation.
    #[must_use]
    pub const fn initial_value(&self) -> DecodedGlobalValue {
        self.initial_value
    }
}

/// Parameters of the module's single linear memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodedMemory {
    /// Initial size in 64 KiB pages.
    initial_pages: u32,
    /// Maximum size in 64 KiB pages, when declared.
    maximum_pages: Option<u32>,
}

impl DecodedMemory {
    /// @why Lets the backend size the module `buffer` at instantiation.
    #[must_use]
    pub const fn initial_pages(&self) -> u32 {
        self.initial_pages
    }

    /// @why Lets the backend cap `memory.grow` at the declared maximum.
    #[must_use]
    pub const fn maximum_pages(&self) -> Option<u32> {
        self.maximum_pages
    }
}

/// How one data segment is materialized into the module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodedDataSegmentKind {
    /// The segment is written into linear memory at a fixed offset during
    /// instantiation.
    Active {
        /// Byte offset into linear memory.
        offset: u32,
    },
    /// The segment lives in its own buffer and is copied into memory by
    /// `memory.init` instructions.
    Passive,
}

/// A data segment decoded for instantiation or `memory.init`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedDataSegment {
    /// Whether the segment is active (offset in linear memory) or passive.
    kind: DecodedDataSegmentKind,
    /// The raw bytes carried by the segment.
    bytes: Vec<u8>,
}

impl DecodedDataSegment {
    /// @why Lets the backend choose the write target per segment kind.
    #[must_use]
    pub const fn kind(&self) -> DecodedDataSegmentKind {
        self.kind
    }

    /// @why Lets the backend emit the carried bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// An active element segment that populates the indirect-call table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedElementSegment {
    /// Table index at which the first function entry lands.
    table_offset: u32,
    /// Function indices in segment order.
    function_indices: Vec<usize>,
}

impl DecodedElementSegment {
    /// @why Lets the backend compute the table slot for each entry.
    #[must_use]
    pub const fn table_offset(&self) -> u32 {
        self.table_offset
    }

    /// @why Lets the backend bind each entry to its generated function.
    #[must_use]
    pub fn function_indices(&self) -> &[usize] {
        &self.function_indices
    }
}

/// Decodes and validates a wasm binary into the supported subset.
///
/// # Errors
///
/// Returns a typed rejection naming every unsupported feature instead of
/// translating a module the backend could only approximate.
#[must_use]
pub fn decode_module(wasm_bytes: &[u8]) -> DecodeOutcome {
    let module = match walrus::Module::from_buffer(wasm_bytes) {
        Ok(module) => module,
        Err(error) => {
            return DecodeOutcome::Rejected(WasmDecodeRejection::from(
                WasmDecodeProblemReason::MalformedModule(error.to_string().into_boxed_str()),
            ));
        }
    };
    let mut problems = Vec::new();

    reject_unsupported_surface(&module, &mut problems);

    let memory = match decode_memory(&module) {
        Ok(decoded_memory) => decoded_memory,
        Err(reason) => {
            problems.push(reason);
            None
        }
    };
    let functions = match decode_functions(&module) {
        Ok(decoded_functions) => decoded_functions,
        Err(reason) => {
            problems.push(reason);
            Vec::new()
        }
    };
    let exports = decode_exports(&module, &mut problems);
    let imports = decode_imports(&module, &mut problems);
    let globals = decode_globals(&module, &mut problems);
    let data_segments = decode_data_segments(&module, &mut problems);
    let element_segments = decode_element_segments(&module, &mut problems);
    let start_function = module
        .start
        .map_or(StartFunctionPresence::Absent, |function_id| {
            StartFunctionPresence::Declared {
                function_index: function_id.index(),
            }
        });

    if problems.is_empty() {
        DecodeOutcome::Decoded(Box::new(DecodedModule {
            walrus_module: module,
            functions,
            exports,
            imports,
            globals,
            data_segments,
            element_segments,
            memory,
            start_function,
        }))
    } else {
        DecodeOutcome::Rejected(WasmDecodeRejection::from_module(
            problems, &module, wasm_bytes,
        ))
    }
}

/// Rejects proposals the backend does not model before any translation begins.
fn reject_unsupported_surface(module: &Module, problems: &mut Vec<WasmDecodeProblemReason>) {
    if module.tags.iter().next().is_some() {
        problems.push(WasmDecodeProblemReason::UnsupportedExceptionHandling);
    }
    for memory in module.memories.iter() {
        if memory.shared || memory.memory64 {
            problems.push(WasmDecodeProblemReason::UnsupportedInstruction {
                instruction: "shared or 64-bit memory".into(),
            });
        }
    }
    for global in module.globals.iter() {
        if global.ty == ValType::V128 {
            problems.push(WasmDecodeProblemReason::UnsupportedVectorType);
        }
    }
    for function in module.funcs.iter() {
        let function_type = module.types.get(function.ty());
        for value_type in function_type.params().iter().chain(function_type.results()) {
            if *value_type == ValType::V128 {
                problems.push(WasmDecodeProblemReason::UnsupportedVectorType);
            }
        }
    }
}

fn decode_memory(module: &Module) -> Result<Option<DecodedMemory>, WasmDecodeProblemReason> {
    let memory_count = module.memories.iter().count();
    if memory_count > MAX_MEMORIES {
        return Err(WasmDecodeProblemReason::UnsupportedMemoryCount {
            count: memory_count,
        });
    }
    let mut decoded_memory = None;
    for memory in module.memories.iter() {
        let initial_pages = match u32::try_from(memory.initial) {
            Ok(pages) => pages,
            Err(conversion_error) => {
                return Err(WasmDecodeProblemReason::MemorySizeTooLarge {
                    pages: memory.initial,
                    detail: conversion_error.to_string(),
                });
            }
        };
        let maximum_pages = match memory.maximum {
            Some(maximum) => match u32::try_from(maximum) {
                Ok(pages) => Some(pages),
                Err(conversion_error) => {
                    return Err(WasmDecodeProblemReason::MemorySizeTooLarge {
                        pages: maximum,
                        detail: conversion_error.to_string(),
                    });
                }
            },
            None => None,
        };
        decoded_memory = Some(DecodedMemory {
            initial_pages,
            maximum_pages,
        });
    }
    Ok(decoded_memory)
}

fn decode_functions(module: &Module) -> Result<Vec<DecodedFunction>, WasmDecodeProblemReason> {
    let mut decoded_functions = Vec::new();
    for function in module.funcs.iter() {
        let function_type = module.types.get(function.ty());
        let params = function_type
            .params()
            .iter()
            .copied()
            .map(WasmValueType::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let results = function_type
            .results()
            .iter()
            .copied()
            .map(WasmValueType::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        decoded_functions.push(DecodedFunction {
            index: function.id().index(),
            name: function.name.clone(),
            params,
            results,
            local_count: match &function.kind {
                FunctionKind::Local(local_function) => local_function
                    .args
                    .len()
                    .saturating_sub(function_type.params().len()),
                FunctionKind::Import(_) | FunctionKind::Uninitialized(_) => 0,
            },
            body: match &function.kind {
                FunctionKind::Local(local_function) => DecodedFunctionBody::Defined {
                    entry_sequence: local_function.entry_block(),
                },
                FunctionKind::Import(_) | FunctionKind::Uninitialized(_) => {
                    DecodedFunctionBody::Imported
                }
            },
        });
    }
    Ok(decoded_functions)
}

fn decode_exports(
    module: &Module,
    problems: &mut Vec<WasmDecodeProblemReason>,
) -> Vec<DecodedExport> {
    let mut decoded_exports = Vec::new();
    for export in module.exports.iter() {
        match export.item {
            ExportItem::Function(function_id) => {
                decoded_exports.push(DecodedExport::Function {
                    name: export.name.clone(),
                    function_index: function_id.index(),
                });
            }
            ExportItem::Memory(_) => {
                decoded_exports.push(DecodedExport::Memory {
                    name: export.name.clone(),
                });
            }
            ExportItem::Global(_) => {
                problems.push(WasmDecodeProblemReason::UnsupportedExportKind {
                    kind: "global",
                    name: export.name.clone(),
                });
            }
            ExportItem::Table(_) => {
                problems.push(WasmDecodeProblemReason::UnsupportedExportKind {
                    kind: "table",
                    name: export.name.clone(),
                });
            }
            ExportItem::Tag(_) => {
                problems.push(WasmDecodeProblemReason::UnsupportedExportKind {
                    kind: "tag",
                    name: export.name.clone(),
                });
            }
        }
    }
    decoded_exports
}

fn decode_imports(
    module: &Module,
    problems: &mut Vec<WasmDecodeProblemReason>,
) -> Vec<DecodedImport> {
    let mut decoded_imports = Vec::new();
    for import in module.imports.iter() {
        match &import.kind {
            ImportKind::Function(function_id) => {
                decoded_imports.push(DecodedImport {
                    module: import.module.clone(),
                    name: import.name.clone(),
                    function_index: function_id.index(),
                });
            }
            ImportKind::Memory(_) => {
                problems.push(WasmDecodeProblemReason::UnsupportedImportKind {
                    kind: "memory",
                    module: import.module.clone(),
                    name: import.name.clone(),
                });
            }
            ImportKind::Table(_) => {
                problems.push(WasmDecodeProblemReason::UnsupportedImportKind {
                    kind: "table",
                    module: import.module.clone(),
                    name: import.name.clone(),
                });
            }
            ImportKind::Global(_) => {
                problems.push(WasmDecodeProblemReason::UnsupportedImportKind {
                    kind: "global",
                    module: import.module.clone(),
                    name: import.name.clone(),
                });
            }
            ImportKind::Tag(_) => {
                problems.push(WasmDecodeProblemReason::UnsupportedImportKind {
                    kind: "tag",
                    module: import.module.clone(),
                    name: import.name.clone(),
                });
            }
        }
    }
    decoded_imports
}

fn decode_globals(
    module: &Module,
    problems: &mut Vec<WasmDecodeProblemReason>,
) -> Vec<DecodedGlobal> {
    let mut decoded_globals = Vec::new();
    for global in module.globals.iter() {
        let value_type = match WasmValueType::try_from(global.ty) {
            Ok(value_type) => value_type,
            Err(reason) => {
                problems.push(reason);
                continue;
            }
        };
        let initial_value = match &global.kind {
            GlobalKind::Local(expression) => match fold_global_initializer(expression) {
                Ok(initial_value) => initial_value,
                Err(reason) => {
                    problems.push(reason);
                    continue;
                }
            },
            GlobalKind::Import(_) => {
                problems.push(WasmDecodeProblemReason::UnsupportedImportKind {
                    kind: "global",
                    module: String::new(),
                    name: String::new(),
                });
                continue;
            }
        };
        decoded_globals.push(DecodedGlobal {
            value_type,
            mutable: global.mutable,
            initial_value,
        });
    }
    decoded_globals
}

/// Folds a global initializer constant expression into a concrete value.
const fn fold_global_initializer(
    expression: &ConstExpr,
) -> Result<DecodedGlobalValue, WasmDecodeProblemReason> {
    match expression {
        ConstExpr::Value(Value::I32(constant)) => Ok(DecodedGlobalValue::I32(*constant)),
        ConstExpr::Value(Value::I64(constant)) => Ok(DecodedGlobalValue::I64(*constant)),
        ConstExpr::Value(Value::F32(constant)) => Ok(DecodedGlobalValue::F32(*constant)),
        ConstExpr::Value(Value::F64(constant)) => Ok(DecodedGlobalValue::F64(*constant)),
        ConstExpr::RefNull(_) => Ok(DecodedGlobalValue::NullReference),
        ConstExpr::RefFunc(_)
        | ConstExpr::Value(Value::V128(_))
        | ConstExpr::Global(_)
        | ConstExpr::Extended(_) => Err(WasmDecodeProblemReason::UnsupportedGlobalInitializer),
    }
}

fn decode_data_segments(
    module: &Module,
    problems: &mut Vec<WasmDecodeProblemReason>,
) -> Vec<DecodedDataSegment> {
    let mut decoded_segments = Vec::new();
    for data_segment in module.data.iter() {
        match &data_segment.kind {
            DataKind::Active { memory, offset } => {
                let memory_index = match u32::try_from(memory.index()) {
                    Ok(memory_index) => memory_index,
                    Err(conversion_error) => {
                        problems.push(WasmDecodeProblemReason::MemoryIndexTooLarge {
                            index: memory.index(),
                            detail: conversion_error.to_string(),
                        });
                        continue;
                    }
                };
                if memory_index != 0 {
                    problems
                        .push(WasmDecodeProblemReason::InvalidDataSegmentMemory { memory_index });
                    continue;
                }
                match fold_const_i32(offset) {
                    Ok(offset_value) => match u32::try_from(offset_value) {
                        Ok(offset_u32) => {
                            decoded_segments.push(DecodedDataSegment {
                                kind: DecodedDataSegmentKind::Active { offset: offset_u32 },
                                bytes: data_segment.value.clone(),
                            });
                        }
                        Err(_) => {
                            problems.push(WasmDecodeProblemReason::NegativeSegmentOffset {
                                offset: offset_value,
                            });
                        }
                    },
                    Err(reason) => problems.push(reason),
                }
            }
            DataKind::Passive => {
                decoded_segments.push(DecodedDataSegment {
                    kind: DecodedDataSegmentKind::Passive,
                    bytes: data_segment.value.clone(),
                });
            }
        }
    }
    decoded_segments
}

fn decode_element_segments(
    module: &Module,
    problems: &mut Vec<WasmDecodeProblemReason>,
) -> Vec<DecodedElementSegment> {
    let mut decoded_segments = Vec::new();
    for element_segment in module.elements.iter() {
        match &element_segment.kind {
            ElementKind::Active { table, offset } => {
                if table.index() != 0 {
                    problems.push(WasmDecodeProblemReason::UnsupportedElementSegment);
                    continue;
                }
                let ElementItems::Functions(function_indices) = &element_segment.items else {
                    problems.push(WasmDecodeProblemReason::UnsupportedElementSegment);
                    continue;
                };
                match fold_const_i32(offset) {
                    Ok(table_offset) => match u32::try_from(table_offset) {
                        Ok(table_offset_u32) => {
                            decoded_segments.push(DecodedElementSegment {
                                table_offset: table_offset_u32,
                                function_indices: function_indices
                                    .iter()
                                    .map(walrus::FunctionId::index)
                                    .collect(),
                            });
                        }
                        Err(_) => {
                            problems.push(WasmDecodeProblemReason::NegativeSegmentOffset {
                                offset: table_offset,
                            });
                        }
                    },
                    Err(_) => problems.push(WasmDecodeProblemReason::UnsupportedElementSegment),
                }
            }
            ElementKind::Passive | ElementKind::Declared => {
                problems.push(WasmDecodeProblemReason::UnsupportedElementSegment);
            }
        }
    }
    decoded_segments
}

/// Folds a constant expression to a single `i32` value.
///
/// The decoder accepts the constant-expression forms emitted by rustc and
/// standard linkers: an immediate constant.
const fn fold_const_i32(expression: &ConstExpr) -> Result<i32, WasmDecodeProblemReason> {
    match expression {
        ConstExpr::Value(Value::I32(constant)) => Ok(*constant),
        ConstExpr::Value(Value::I64(_) | Value::F32(_) | Value::F64(_) | Value::V128(_))
        | ConstExpr::Global(_)
        | ConstExpr::RefNull(_)
        | ConstExpr::RefFunc(_)
        | ConstExpr::Extended(_) => Err(WasmDecodeProblemReason::UnsupportedDataOffset),
    }
}
