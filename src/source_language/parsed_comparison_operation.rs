use crate::{
    source_language::{ParsedComparisonOperator, ParsedExpression},
    SourceRange,
};

/// Retains one parsed numeric comparison and its diagnostic locations.
pub(crate) struct ParsedComparisonOperation {
    left_operand: Box<ParsedExpression>,
    right_operand: Box<ParsedExpression>,
    operator: ParsedComparisonOperator,
    operator_range: SourceRange,
    expression_range: SourceRange,
}

/// Provides construction and stage-boundary access for parsed comparisons.
impl ParsedComparisonOperation {
    /// Builds a parsed comparison from its operands, operator, and source locations.
    pub(crate) fn from_parts(
        parts: (
            Box<ParsedExpression>,
            Box<ParsedExpression>,
            ParsedComparisonOperator,
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

    /// Gives the parsed comparison operator to later compiler stages.
    pub(crate) fn operator(&self) -> &ParsedComparisonOperator {
        &self.operator
    }

    /// Gives the operator location used for comparison type diagnostics.
    pub(crate) fn operator_range(&self) -> SourceRange {
        self.operator_range
    }

    /// Gives the complete comparison range.
    pub(crate) fn expression_range(&self) -> SourceRange {
        self.expression_range
    }
}
