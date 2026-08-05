use crate::{source_language::ParsedExpression, SourceRange};

/// Retains the non-empty expressions that establish one homogeneous array type.
pub struct ParsedArrayLiteral {
    element_expressions: Vec<ParsedExpression>,
    literal_range: SourceRange,
}

/// Keeps literal elements and their full source span together for semantic checking.
impl ParsedArrayLiteral {
    /// Builds an array only after syntax has rejected the ambiguous empty form.
    pub(crate) fn from_elements(array_parts: (Vec<ParsedExpression>, SourceRange)) -> Self {
        let (element_expressions, literal_range) = array_parts;
        Self {
            element_expressions,
            literal_range,
        }
    }
    /// Provides the ordered element expressions.
    pub(crate) fn element_expressions(&self) -> &[ParsedExpression] {
        &self.element_expressions
    }
    /// Provides the complete source span for this literal.
    pub(crate) const fn literal_range(&self) -> SourceRange {
        self.literal_range
    }
}
