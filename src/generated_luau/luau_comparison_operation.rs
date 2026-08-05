use crate::generated_luau::{LuauComparisonOperator, LuauExpression};

/// Retains one target numeric comparison for precedence-aware writing.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauComparisonOperation {
    left_operand: Box<LuauExpression>,
    right_operand: Box<LuauExpression>,
    operator: LuauComparisonOperator,
}

/// Provides construction and stage-boundary access for target comparisons.
impl LuauComparisonOperation {
    /// Builds a target comparison from generated numeric operands and an operator.
    pub(crate) fn from_parts(
        parts: (
            Box<LuauExpression>,
            Box<LuauExpression>,
            LuauComparisonOperator,
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

    /// Gives the generated comparison operator to the writer.
    pub(crate) const fn operator(&self) -> &LuauComparisonOperator {
        &self.operator
    }
}
