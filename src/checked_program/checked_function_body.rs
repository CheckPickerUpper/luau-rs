use crate::checked_program::CheckedStatement;

/// Owns statements that have passed type and reachability checking within one lexical scope.
pub(crate) struct CheckedFunctionBody {
    body_statements: Vec<CheckedStatement>,
}

/// Keeps branch and function bodies structurally identical after semantic validation.
impl CheckedFunctionBody {
    /// Collects checked statements that share one scope.
    pub(crate) fn from_statements(body_statements: Vec<CheckedStatement>) -> Self {
        Self { body_statements }
    }

    /// Supplies ordered statements for Luau lowering.
    pub(crate) fn body_statements(&self) -> &[CheckedStatement] {
        &self.body_statements
    }
}
