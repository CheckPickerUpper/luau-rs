use crate::generated_luau::LuauExpression;

/// Retains one target boolean negation for precedence-aware writing.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauLogicalNegation {
    negated_expression: Box<LuauExpression>,
}

/// Provides construction and stage-boundary access for target negation.
impl LuauLogicalNegation {
    /// Builds a target negation from a generated boolean operand.
    pub(crate) const fn from_expression(negated_expression: Box<LuauExpression>) -> Self {
        Self { negated_expression }
    }

    /// Gives the generated operand to the writer.
    pub(crate) fn negated_expression(&self) -> &LuauExpression {
        &self.negated_expression
    }
}
