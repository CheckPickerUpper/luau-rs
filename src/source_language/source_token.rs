use crate::{source_language::SourceTokenKind, MacroExpansionFrame, SourceRange};

/// Couples one lexical category with its byte range.
#[derive(Clone)]
pub struct SourceToken {
    token_kind: SourceTokenKind,
    source_range: SourceRange,
    macro_backtrace: Vec<MacroExpansionFrame>,
}

/// Prevents parsing from losing the tokenizer's source locations.
impl SourceToken {
    /// Restricts token construction to byte-aware tokenization.
    pub(crate) fn from_token_at_range(token_at_range: (SourceTokenKind, SourceRange)) -> Self {
        let (token_kind, source_range) = token_at_range;
        Self::from_token_at_origin((token_kind, source_range, Vec::new()))
    }

    pub(crate) fn from_token_at_origin(
        token_at_origin: (SourceTokenKind, SourceRange, Vec<MacroExpansionFrame>),
    ) -> Self {
        let (token_kind, source_range, macro_backtrace) = token_at_origin;
        Self {
            token_kind,
            source_range,
            macro_backtrace,
        }
    }
    /// Gives the parser a borrowed view of the lexical category.
    pub(crate) const fn token_kind(&self) -> &SourceTokenKind {
        &self.token_kind
    }
    /// Gives parse diagnostics the original token location.
    pub(crate) const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub(crate) fn macro_backtrace(&self) -> &[MacroExpansionFrame] {
        &self.macro_backtrace
    }

    /// Transfers the lexical category and location into the handwritten parser.
    pub(crate) fn into_token_at_range(self) -> (SourceTokenKind, SourceRange) {
        (self.token_kind, self.source_range)
    }
}
