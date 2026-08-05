use crate::{source_language::ParsedExpression, SourceRange};

/// Retains one source-language zero-based array access.
pub struct ParsedArrayRead {
    base_expression: Box<ParsedExpression>,
    index_expression: Box<ParsedExpression>,
    expression_range: SourceRange,
}

/// Keeps both access operands and their complete range for precise diagnostics.
impl ParsedArrayRead {
    /// Builds an array read while postfix syntax is being parsed.
    pub(crate) fn from_read(
        read_parts: (Box<ParsedExpression>, Box<ParsedExpression>, SourceRange),
    ) -> Self {
        let (base_expression, index_expression, expression_range) = read_parts;
        Self {
            base_expression,
            index_expression,
            expression_range,
        }
    }
    /// Provides the indexed value.
    pub(crate) const fn base_expression(&self) -> &ParsedExpression {
        &self.base_expression
    }
    /// Provides the required numeric source index.
    pub(crate) const fn index_expression(&self) -> &ParsedExpression {
        &self.index_expression
    }
    /// Provides the full read range.
    pub(crate) const fn expression_range(&self) -> SourceRange {
        self.expression_range
    }
    /// Decomposes the read while flattening an assignment target.
    pub(crate) fn into_read(self) -> (Box<ParsedExpression>, ParsedExpression, SourceRange) {
        (
            self.base_expression,
            *self.index_expression,
            self.expression_range,
        )
    }
}
