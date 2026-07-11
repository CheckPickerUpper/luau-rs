use crate::checked_program::{CheckedEqualityOperator, CheckedExpression};

/// Retains a checked equality operation for target generation.
pub(crate) struct CheckedEqualityOperation {
    left_operand: Box<CheckedExpression>,
    right_operand: Box<CheckedExpression>,
    operator: CheckedEqualityOperator,
}

/// Provides construction and stage-boundary access for checked equality operations.
impl CheckedEqualityOperation {
    /// Builds a checked equality operation from matching typed operands and an operator.
    pub(crate) fn from_parts(
        parts: (
            Box<CheckedExpression>,
            Box<CheckedExpression>,
            CheckedEqualityOperator,
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

    /// Gives the checked equality operator to target generation.
    pub(crate) fn operator(&self) -> &CheckedEqualityOperator {
        &self.operator
    }
}
