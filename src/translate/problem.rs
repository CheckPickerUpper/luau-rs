use thiserror::Error;

/// Names one reason translation stopped before an artifact was produced.
///
/// Every payload field is documented by its `#[error]` message text.
#[allow(
    missing_docs,
    reason = "thiserror messages document every payload field"
)]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TranslationProblemReason {
    /// A wasm instruction the backend does not translate yet appeared inside
    /// a function body (SIMD, atomics, bulk memory, reference types, ...).
    #[error("instruction \"{instruction}\" is not yet translated")]
    UnsupportedInstruction { instruction: String },

    /// `global.set` targets a global the module declares immutable.
    #[error("global.set targets immutable global {global_index}")]
    ImmutableGlobalSet { global_index: usize },

    /// A `call_indirect` instruction references a table that does not exist.
    #[error("call_indirect references a table the module does not declare")]
    MissingIndirectCallTable,

    /// A memory instruction appears in a module that declares no memory.
    #[error("memory instruction appears in a module with no memory")]
    MissingMemory,

    /// A function body failed to translate for an internal reason.
    #[error("internal translation failure: {0}")]
    Internal(String),
}

/// Carries every translation problem discovered while emitting one module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationRejection {
    problems: Vec<TranslationProblemReason>,
}

impl TranslationRejection {
    /// @why Lets every rejection problem travel together through one outcome.
    #[must_use]
    pub const fn from_problems(problems: Vec<TranslationProblemReason>) -> Self {
        Self { problems }
    }

    /// @why Lets callers report every problem at once instead of stopping at the first.
    #[must_use]
    #[allow(
        clippy::missing_const_for_fn,
        reason = "Vec-to-slice coercion is not const-stable"
    )]
    pub fn problems(&self) -> &[TranslationProblemReason] {
        &self.problems
    }

    /// @why Gives diagnostics a stable count without exposing the problem vector.
    #[must_use]
    pub const fn problem_count(&self) -> usize {
        self.problems.len()
    }
}

impl From<TranslationProblemReason> for TranslationRejection {
    fn from(reason: TranslationProblemReason) -> Self {
        Self::from_problems(vec![reason])
    }
}
