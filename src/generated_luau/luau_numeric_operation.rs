use crate::generated_luau::{LuauExpression, LuauNumericOperator};

/// Retains one target numeric operation for precedence-aware writing.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LuauNumericOperation {
    left_operand: Box<LuauExpression>,
    right_operand: Box<LuauExpression>,
    operator: LuauNumericOperator,
}

/// Provides construction and stage-boundary access for target operations.
impl LuauNumericOperation {
    /// Builds a target operation from generated operands and an operator.
    pub(crate) fn from_parts(
        parts: (
            Box<LuauExpression>,
            Box<LuauExpression>,
            LuauNumericOperator,
        ),
    ) -> Self {
        let (left_operand, right_operand, operator) = parts;
        Self {
            left_operand,
            right_operand,
            operator,
        }
    }

    /// Gives the generated left operand to the writer.
    pub(crate) fn left_operand(&self) -> &LuauExpression {
        &self.left_operand
    }

    /// Gives the generated right operand to the writer.
    pub(crate) fn right_operand(&self) -> &LuauExpression {
        &self.right_operand
    }

    /// Gives the generated operator to the writer.
    pub(crate) fn operator(&self) -> &LuauNumericOperator {
        &self.operator
    }
}
