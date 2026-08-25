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
}

impl WasmDecodeRejection {
    /// @why Lets every rejection problem travel together through one outcome.
    #[must_use]
    pub const fn from_problems(problems: Vec<WasmDecodeProblemReason>) -> Self {
        Self { problems }
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
