use crate::checked_program::CheckedExpression;

/// Retains a checked boolean negation for target generation.
pub(crate) struct CheckedLogicalNegation {
    negated_expression: Box<CheckedExpression>,
}

/// Provides construction and stage-boundary access for checked negation.
impl CheckedLogicalNegation {
    /// Builds a checked negation from a validated boolean operand.
    pub(crate) fn from_expression(negated_expression: Box<CheckedExpression>) -> Self {
        Self { negated_expression }
    }

    /// Gives the checked operand to target generation.
    pub(crate) fn negated_expression(&self) -> &CheckedExpression {
        &self.negated_expression
    }
}
