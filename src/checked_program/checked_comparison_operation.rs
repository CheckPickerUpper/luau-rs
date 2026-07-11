use crate::checked_program::{CheckedComparisonOperator, CheckedExpression};

/// Retains a checked numeric comparison for target generation.
pub(crate) struct CheckedComparisonOperation {
    left_operand: Box<CheckedExpression>,
    right_operand: Box<CheckedExpression>,
    operator: CheckedComparisonOperator,
}

/// Provides construction and stage-boundary access for checked comparisons.
impl CheckedComparisonOperation {
    /// Builds a checked comparison from validated numeric operands and an operator.
    pub(crate) fn from_parts(
        parts: (
            Box<CheckedExpression>,
            Box<CheckedExpression>,
            CheckedComparisonOperator,
        ),
    ) -> Self {
        let (left_operand, right_operand, operator) = parts;
        Self {
            left_operand,
            right_operand,
            operator,
        }
    }

    /// Gives the checked left operand to target generation.
    pub(crate) fn left_operand(&self) -> &CheckedExpression {
        &self.left_operand
    }

    /// Gives the checked right operand to target generation.
    pub(crate) fn right_operand(&self) -> &CheckedExpression {
        &self.right_operand
    }

    /// Gives the checked comparison operator to target generation.
    pub(crate) fn operator(&self) -> &CheckedComparisonOperator {
        &self.operator
    }
}
