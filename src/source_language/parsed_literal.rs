use crate::SourceRange;

/// Keeps a validated literal spelling coupled to its diagnostic range.
pub(crate) struct ParsedLiteral {
    literal_spelling: String,
    literal_range: SourceRange,
}

/// Prevents literal variants from duplicating source-attribution fields.
impl ParsedLiteral {
    /// Preserves lexer-owned spelling and location without reinterpretation.
    pub(crate) fn from_spelling_at_range(spelling_at_range: (String, SourceRange)) -> Self {
        let (literal_spelling, literal_range) = spelling_at_range;
        Self {
            literal_spelling,
            literal_range,
        }
    }

    /// Gives semantic checking the validated source spelling.
    pub(crate) fn literal_spelling(&self) -> &str {
        &self.literal_spelling
    }

    /// Gives diagnostics the complete literal range.
    pub(crate) fn literal_range(&self) -> SourceRange {
        self.literal_range
    }
}
