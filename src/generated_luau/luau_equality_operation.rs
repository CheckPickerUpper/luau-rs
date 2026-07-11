use crate::generated_luau::{LuauEqualityOperator, LuauExpression};

/// Retains one target equality operation for precedence-aware writing.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LuauEqualityOperation {
    left_operand: Box<LuauExpression>,
    right_operand: Box<LuauExpression>,
    operator: LuauEqualityOperator,
}

/// Provides construction and stage-boundary access for target equality operations.
impl LuauEqualityOperation {
    /// Builds a target equality operation from generated matching typed operands and an operator.
    pub(crate) fn from_parts(
        parts: (
            Box<LuauExpression>,
            Box<LuauExpression>,
            LuauEqualityOperator,
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

    /// Gives the generated equality operator to the writer.
    pub(crate) fn operator(&self) -> &LuauEqualityOperator {
        &self.operator
    }
}
