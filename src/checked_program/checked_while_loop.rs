use crate::checked_program::{CheckedExpression, CheckedFunctionBody};

/// Retains a checked boolean condition and the scoped statements it repeatedly evaluates.
pub struct CheckedWhileLoop {
    condition: CheckedExpression,
    body: CheckedFunctionBody,
}

/// Ensures Luau lowering never needs to reconsider the loop's condition type or local scope.
impl CheckedWhileLoop {
    /// Joins a proven boolean condition with its independently checked body.
    pub(crate) fn from_parts(while_loop_parts: (CheckedExpression, CheckedFunctionBody)) -> Self {
        let (condition, body) = while_loop_parts;
        Self { condition, body }
    }

    /// Supplies the condition that is proven to evaluate to a boolean.
    pub(crate) const fn condition(&self) -> &CheckedExpression {
        &self.condition
    }

    /// Supplies statements evaluated for each iteration.
    pub(crate) const fn body(&self) -> &CheckedFunctionBody {
        &self.body
    }
}
