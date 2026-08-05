use crate::{source_language::ParsedExpression, SourceRange};

/// Retains one parsed logical negation and its diagnostic locations.
pub struct ParsedLogicalNegation {
    negated_expression: Box<ParsedExpression>,
    operator_range: SourceRange,
    expression_range: SourceRange,
}

/// Provides construction and stage-boundary access for parsed negation.
impl ParsedLogicalNegation {
    /// Builds a parsed negation from its operand and source locations.
    pub(crate) fn from_parts(parts: (Box<ParsedExpression>, SourceRange, SourceRange)) -> Self {
        let (negated_expression, operator_range, expression_range) = parts;
        Self {
            negated_expression,
            operator_range,
            expression_range,
        }
    }

    /// Gives the negated expression to later compiler stages.
    pub(crate) fn negated_expression(&self) -> &ParsedExpression {
        &self.negated_expression
    }

    /// Gives the operator location used for negation type diagnostics.
    pub(crate) const fn operator_range(&self) -> SourceRange {
        self.operator_range
    }

    /// Gives the complete negation range.
    pub(crate) const fn expression_range(&self) -> SourceRange {
        self.expression_range
    }
}
