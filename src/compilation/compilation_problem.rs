use crate::{CompilationProblemReason, SourceRange};

/// Couples a typed compiler failure with its original source location.
#[derive(Debug, PartialEq, Eq)]
pub struct CompilationProblem {
    source_range: SourceRange,
    reason: CompilationProblemReason,
}

/// Restricts diagnostics to typed compiler phase failures.
impl CompilationProblem {
    /// Keeps source location and rejection classification together across phases.
    pub(crate) const fn from_problem_at_range(
        problem_at_range: (SourceRange, CompilationProblemReason),
    ) -> Self {
        let (source_range, reason) = problem_at_range;
        Self {
            source_range,
            reason,
        }
    }

    /// @why Returns the typed failure reason so callers can classify a rejection without parsing text.
    #[must_use]
    pub const fn reason(&self) -> &CompilationProblemReason {
        &self.reason
    }

    /// @why Preserves exact source attribution so editors can highlight the construct responsible for a rejection.
    #[must_use]
    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
}
