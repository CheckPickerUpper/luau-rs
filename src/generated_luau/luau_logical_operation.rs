use crate::generated_luau::{LuauExpression, LuauLogicalOperator};

/// Retains one target short-circuit logical operation for precedence-aware writing.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LuauLogicalOperation {
    left_operand: Box<LuauExpression>,
    right_operand: Box<LuauExpression>,
    operator: LuauLogicalOperator,
}

/// Provides construction and stage-boundary access for target logical operations.
impl LuauLogicalOperation {
    /// Builds a target logical operation from generated boolean operands and an operator.
    pub(crate) fn from_parts(
        parts: (
            Box<LuauExpression>,
            Box<LuauExpression>,
            LuauLogicalOperator,
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

    /// Gives the generated logical operator to the writer.
    pub(crate) fn operator(&self) -> &LuauLogicalOperator {
        &self.operator
    }
}
