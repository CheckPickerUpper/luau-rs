use crate::{CompilationProblemReason, MacroExpansionFrame, SourceRange};

/// Couples a typed compiler failure with its original source location.
#[derive(Debug, PartialEq, Eq)]
pub struct CompilationProblem {
    source_range: SourceRange,
    reason: CompilationProblemReason,
    macro_backtrace: Option<Box<[MacroExpansionFrame]>>,
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
            macro_backtrace: None,
        }
    }

    pub(crate) fn from_problem_at_origin(
        problem_at_origin: (
            SourceRange,
            CompilationProblemReason,
            Vec<MacroExpansionFrame>,
        ),
    ) -> Self {
        let (source_range, reason, macro_backtrace) = problem_at_origin;
        Self {
            source_range,
            reason,
            macro_backtrace: if macro_backtrace.is_empty() {
                None
            } else {
                Some(macro_backtrace.into_boxed_slice())
            },
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

    /// @why Keeps expansion context attached to the structured diagnostic so an editor can show both definition and invocation sites.
    #[must_use]
    pub fn macro_backtrace(&self) -> &[MacroExpansionFrame] {
        match &self.macro_backtrace {
            Some(macro_backtrace) => macro_backtrace,
            None => &[],
        }
    }

    pub(crate) fn with_macro_backtrace(mut self, macro_backtrace: &[MacroExpansionFrame]) -> Self {
        if self.macro_backtrace().is_empty() {
            self.macro_backtrace = Some(macro_backtrace.to_vec().into_boxed_slice());
        }
        self
    }
}
