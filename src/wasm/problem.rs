use crate::diagnostics::{function_context, source_location, Diagnostic, DiagnosticReport};
use thiserror::Error;

/// Names one reason a wasm module cannot be accepted by the luau-rs pipeline.
///
/// Every payload field is documented by its `#[error]` message text.
#[allow(
    missing_docs,
    reason = "thiserror messages document every payload field"
)]
///
/// Every variant is a deliberate scope boundary: the decoder accepts the core
/// wasm instruction set and rejects proposals that the Luau backend does not
/// (yet) model, so a rejected module fails loudly instead of translating into
/// silently wrong Luau.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WasmDecodeProblemReason {
    /// The wasm binary failed to parse or validate in the upstream parser.
    #[error("wasm module failed to parse: {0}")]
    MalformedModule(Box<str>),

    /// The module declares more than one linear memory, which the Luau
    /// backend models as a single module-scoped `buffer`.
    #[error("module declares {count} memories; luau-rs supports exactly one")]
    UnsupportedMemoryCount { count: usize },

    /// A memory size in wasm pages does not fit the backend's `u32` model.
    #[error("memory size {pages} pages is larger than the supported range: {detail}")]
    MemorySizeTooLarge { pages: u64, detail: String },

    /// A data or element segment offset is negative, which is invalid wasm.
    #[error("segment offset {offset} is negative")]
    NegativeSegmentOffset { offset: i32 },

    /// A memory index does not fit the backend's `u32` model.
    #[error("memory index {index} is larger than the supported range: {detail}")]
    MemoryIndexTooLarge { index: usize, detail: String },

    /// The module imports a memory, table, global, or tag. Only function
    /// imports are supported because they map to Luau callbacks.
    #[error("imported {kind} \"{module}.{name}\" is not supported; only function imports are")]
    UnsupportedImportKind {
        kind: &'static str,
        module: String,
        name: String,
    },

    /// The module uses a wasm proposal or instruction the backend does not
    /// translate yet (SIMD, atomics, bulk memory, reference types, ...).
    #[error("instruction \"{instruction}\" is not yet translated")]
    UnsupportedInstruction { instruction: String },

    /// A `v128` vector value appears in a signature or constant.
    #[error("v128 vector values are not supported")]
    UnsupportedVectorType,

    /// Exception-handling tags are not supported.
    #[error("exception-handling tags are not supported")]
    UnsupportedExceptionHandling,

    /// An active data segment references a memory other than memory 0.
    #[error("data segment references memory {memory_index}, but only memory 0 exists")]
    InvalidDataSegmentMemory { memory_index: u32 },

    /// An element segment is passive, declarative, or uses expressions.
    #[error("element segment form is not supported; only active function-index segments are")]
    UnsupportedElementSegment,

    /// An exported kind other than function or memory is not supported yet.
    #[error("exported {kind} \"{name}\" is not supported; only functions and memory are")]
    UnsupportedExportKind { kind: &'static str, name: String },

    /// A global initializer is not a constant expression the decoder can fold.
    #[error("global initializer is not a constant expression")]
    UnsupportedGlobalInitializer,

    /// A data segment offset is not a constant expression the decoder can fold.
    #[error("data segment offset is not a constant expression")]
    UnsupportedDataOffset,
}

/// Carries every rejection reason discovered while decoding one wasm module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmDecodeRejection {
    problems: Vec<WasmDecodeProblemReason>,
    diagnostics: DiagnosticReport,
}

impl WasmDecodeRejection {
    /// @why Lets every rejection problem travel together through one outcome.
    #[must_use]
    pub fn from_problems(problems: Vec<WasmDecodeProblemReason>) -> Self {
        let diagnostics = DiagnosticReport::without_locations(
            problems
                .iter()
                .map(|problem| (problem_code(problem).into(), problem.to_string())),
        );
        Self {
            problems,
            diagnostics,
        }
    }

    /// Builds diagnostics while the parsed wasm metadata is still available.
    pub(crate) fn from_module(
        problems: Vec<WasmDecodeProblemReason>,
        module: &walrus::Module,
        raw_wasm: &[u8],
    ) -> Self {
        let mut diagnostics = problems
            .iter()
            .map(|problem| {
                let preferred = |function: &walrus::Function| match problem {
                    WasmDecodeProblemReason::UnsupportedVectorType => {
                        let ty = module.types.get(function.ty());
                        ty.params()
                            .iter()
                            .chain(ty.results())
                            .any(|value_type| *value_type == walrus::ValType::V128)
                    }
                    _ => true,
                };
                let (function, offset) =
                    function_context(module, preferred).unwrap_or((None, None));
                Diagnostic {
                    code: problem_code(problem).into(),
                    message: problem.to_string(),
                    location: source_location(raw_wasm, function, offset),
                }
            })
            .collect::<Vec<_>>();
        diagnostics.dedup();
        Self {
            problems,
            diagnostics: DiagnosticReport::new(diagnostics),
        }
    }

    /// @why Lets callers report every problem at once instead of stopping at the first.
    #[must_use]
    #[allow(
        clippy::missing_const_for_fn,
        reason = "Vec-to-slice coercion is not const-stable"
    )]
    pub fn problems(&self) -> &[WasmDecodeProblemReason] {
        &self.problems
    }

    /// Returns the stable structured diagnostics for these rejection reasons.
    #[must_use]
    pub const fn diagnostics(&self) -> &DiagnosticReport {
        &self.diagnostics
    }

    /// @why Gives diagnostics a stable count without exposing the problem vector.
    #[must_use]
    pub const fn problem_count(&self) -> usize {
        self.problems.len()
    }
}

impl From<WasmDecodeProblemReason> for WasmDecodeRejection {
    fn from(reason: WasmDecodeProblemReason) -> Self {
        Self::from_problems(vec![reason])
    }
}

const fn problem_code(problem: &WasmDecodeProblemReason) -> &'static str {
    match problem {
        WasmDecodeProblemReason::MalformedModule(_) => "malformed_module",
        WasmDecodeProblemReason::UnsupportedMemoryCount { .. } => "unsupported_memory_count",
        WasmDecodeProblemReason::MemorySizeTooLarge { .. } => "memory_size_too_large",
        WasmDecodeProblemReason::NegativeSegmentOffset { .. } => "negative_segment_offset",
        WasmDecodeProblemReason::MemoryIndexTooLarge { .. } => "memory_index_too_large",
        WasmDecodeProblemReason::UnsupportedImportKind { .. } => "unsupported_import_kind",
        WasmDecodeProblemReason::UnsupportedInstruction { .. } => "unsupported_instruction",
        WasmDecodeProblemReason::UnsupportedVectorType => "unsupported_vector_type",
        WasmDecodeProblemReason::UnsupportedExceptionHandling => "unsupported_exception_handling",
        WasmDecodeProblemReason::InvalidDataSegmentMemory { .. } => "invalid_data_segment_memory",
        WasmDecodeProblemReason::UnsupportedElementSegment => "unsupported_element_segment",
        WasmDecodeProblemReason::UnsupportedExportKind { .. } => "unsupported_export_kind",
        WasmDecodeProblemReason::UnsupportedGlobalInitializer => "unsupported_global_initializer",
        WasmDecodeProblemReason::UnsupportedDataOffset => "unsupported_data_offset",
    }
}
