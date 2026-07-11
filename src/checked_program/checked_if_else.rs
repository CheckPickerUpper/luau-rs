use crate::checked_program::{CheckedExpression, CheckedFunctionBody};

/// Retains a checked boolean decision and independently checked branch scopes.
pub(crate) struct CheckedIfElse {
    condition: CheckedExpression,
    then_body: CheckedFunctionBody,
    else_body: CheckedFunctionBody,
}

/// Ensures lowering never needs to reconsider condition type or scope boundaries.
impl CheckedIfElse {
    /// Joins a proven boolean condition with its checked branches.
    pub(crate) fn from_parts(
        if_else_parts: (CheckedExpression, CheckedFunctionBody, CheckedFunctionBody),
    ) -> Self {
        let (condition, then_body, else_body) = if_else_parts;
        Self {
            condition,
            then_body,
            else_body,
        }
    }

    /// Supplies the condition that is proven to evaluate to a boolean.
    pub(crate) fn condition(&self) -> &CheckedExpression {
        &self.condition
    }

    /// Supplies statements evaluated when the condition is true.
    pub(crate) fn then_body(&self) -> &CheckedFunctionBody {
        &self.then_body
    }

    /// Supplies statements evaluated when the condition is false.
    pub(crate) fn else_body(&self) -> &CheckedFunctionBody {
        &self.else_body
    }
}
