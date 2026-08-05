use crate::{source_language::ParsedExpression, SourceRange};

/// Represents one postfix field read from a parsed base expression.
pub struct ParsedFieldRead {
    base_expression: Box<ParsedExpression>,
    field_name: String,
    field_name_range: SourceRange,
    expression_range: SourceRange,
}

/// Separates postfix field reads from primary parsing while retaining every diagnostic range.
impl ParsedFieldRead {
    /// Preserves the base, field spelling, and whole read range.
    pub(crate) fn from_read(
        read: (Box<ParsedExpression>, String, SourceRange, SourceRange),
    ) -> Self {
        let (base_expression, field_name, field_name_range, expression_range) = read;
        Self {
            base_expression,
            field_name,
            field_name_range,
            expression_range,
        }
    }

    /// Gives semantic checking the read base expression.
    pub(crate) fn base_expression(&self) -> &ParsedExpression {
        &self.base_expression
    }

    /// Gives semantic checking the accessed field spelling.
    pub(crate) fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Gives unknown-field diagnostics the field name range.
    pub(crate) const fn field_name_range(&self) -> SourceRange {
        self.field_name_range
    }

    /// Gives enclosing expressions the complete postfix read range.
    pub(crate) const fn expression_range(&self) -> SourceRange {
        self.expression_range
    }

    /// Moves postfix parts into the assignment parser without admitting general expressions as targets.
    pub(crate) fn into_read(self) -> (Box<ParsedExpression>, String, SourceRange, SourceRange) {
        (
            self.base_expression,
            self.field_name,
            self.field_name_range,
            self.expression_range,
        )
    }
}
