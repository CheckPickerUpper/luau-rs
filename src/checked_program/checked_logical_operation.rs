use crate::checked_program::{CheckedExpression, CheckedLogicalOperator};

/// Retains a checked short-circuit logical operation for target generation.
pub struct CheckedLogicalOperation {
    left_operand: Box<CheckedExpression>,
    right_operand: Box<CheckedExpression>,
    operator: CheckedLogicalOperator,
}

/// Provides construction and stage-boundary access for checked logical operations.
impl CheckedLogicalOperation {
    /// Builds a checked logical operation from validated boolean operands and an operator.
    pub(crate) fn from_parts(
        parts: (
            Box<CheckedExpression>,
            Box<CheckedExpression>,
            CheckedLogicalOperator,
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

    /// Gives the checked logical operator to target generation.
    pub(crate) const fn operator(&self) -> &CheckedLogicalOperator {
        &self.operator
    }
}
