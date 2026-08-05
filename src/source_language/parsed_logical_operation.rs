use crate::{
    source_language::{ParsedExpression, ParsedLogicalOperator},
    SourceRange,
};

/// Retains one parsed short-circuit logical operation and its diagnostic locations.
pub struct ParsedLogicalOperation {
    left_operand: Box<ParsedExpression>,
    right_operand: Box<ParsedExpression>,
    operator: ParsedLogicalOperator,
    operator_range: SourceRange,
    expression_range: SourceRange,
}

/// Provides construction and stage-boundary access for parsed logical operations.
impl ParsedLogicalOperation {
    /// Builds a parsed logical operation from its operands, operator, and source locations.
    pub(crate) fn from_parts(
        parts: (
            Box<ParsedExpression>,
            Box<ParsedExpression>,
            ParsedLogicalOperator,
            SourceRange,
            SourceRange,
        ),
    ) -> Self {
        let (left_operand, right_operand, operator, operator_range, expression_range) = parts;
        Self {
            left_operand,
            right_operand,
            operator,
            operator_range,
            expression_range,
        }
    }

    /// Gives the left operand to later compiler stages.
    pub(crate) fn left_operand(&self) -> &ParsedExpression {
        &self.left_operand
    }

    /// Gives the right operand to later compiler stages.
    pub(crate) fn right_operand(&self) -> &ParsedExpression {
        &self.right_operand
    }

    /// Gives the parsed logical operator to later compiler stages.
    pub(crate) const fn operator(&self) -> &ParsedLogicalOperator {
        &self.operator
    }

    /// Gives the operator location used for logical type diagnostics.
    pub(crate) const fn operator_range(&self) -> SourceRange {
        self.operator_range
    }

    /// Gives the complete logical-operation range.
    pub(crate) const fn expression_range(&self) -> SourceRange {
        self.expression_range
    }
}
