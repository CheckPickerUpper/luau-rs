use crate::{
    source_language::{ParsedExpression, ParsedNumericOperator},
    SourceRange,
};

/// Retains one parsed numeric operation and its source locations.
pub struct ParsedNumericOperation {
    left_operand: Box<ParsedExpression>,
    right_operand: Box<ParsedExpression>,
    operator: ParsedNumericOperator,
    operator_range: SourceRange,
    expression_range: SourceRange,
}

/// Provides construction and stage-boundary access for parsed operations.
impl ParsedNumericOperation {
    /// Builds a parsed operation from its operands and source ranges.
    pub(crate) fn from_parts(
        parts: (
            Box<ParsedExpression>,
            Box<ParsedExpression>,
            ParsedNumericOperator,
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

    /// Gives the parsed operator to later compiler stages.
    pub(crate) const fn operator(&self) -> &ParsedNumericOperator {
        &self.operator
    }

    /// Gives the operator range used for numeric type diagnostics.
    pub(crate) const fn operator_range(&self) -> SourceRange {
        self.operator_range
    }

    /// Gives the complete operation range.
    pub(crate) const fn expression_range(&self) -> SourceRange {
        self.expression_range
    }
}
