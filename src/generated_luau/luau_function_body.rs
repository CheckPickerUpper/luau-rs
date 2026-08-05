use crate::generated_luau::LuauStatement;

/// Owns generated statements that share one Luau lexical scope.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauFunctionBody {
    body_statements: Vec<LuauStatement>,
}

/// Keeps nested target-language bodies explicit until text serialization.
impl LuauFunctionBody {
    /// Collects generated statements in execution order.
    pub(crate) const fn from_statements(body_statements: Vec<LuauStatement>) -> Self {
        Self { body_statements }
    }

    /// Gives the writer the statements within this scope.
    pub(crate) fn body_statements(&self) -> &[LuauStatement] {
        &self.body_statements
    }
}
