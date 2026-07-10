use crate::checked_program::{CheckedExpression, CheckedNumericOperator};

/// Retains a checked numeric operation for target generation.
pub(crate) struct CheckedNumericOperation {
    left_operand: Box<CheckedExpression>,
    right_operand: Box<CheckedExpression>,
    operator: CheckedNumericOperator,
}

/// Provides construction and stage-boundary access for checked operations.
impl CheckedNumericOperation {
    /// Builds a checked operation from validated operands and an operator.
    pub(crate) fn from_parts(
        parts: (
            Box<CheckedExpression>,
            Box<CheckedExpression>,
            CheckedNumericOperator,
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

    /// Gives the checked operator to target generation.
    pub(crate) fn operator(&self) -> &CheckedNumericOperator {
        &self.operator
    }
}
