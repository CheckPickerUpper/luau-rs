use crate::source_language::ParsedStatement;

/// Owns the ordered statements inside either a function or a conditional branch.
pub(crate) struct ParsedFunctionBody {
    body_statements: Vec<ParsedStatement>,
}

/// Preserves source nesting until semantic checking establishes each statement's meaning.
impl ParsedFunctionBody {
    /// Collects statements that share one lexical scope.
    pub(crate) fn from_statements(body_statements: Vec<ParsedStatement>) -> Self {
        Self { body_statements }
    }

    /// Supplies ordered statements for semantic checking.
    pub(crate) fn body_statements(&self) -> &[ParsedStatement] {
        &self.body_statements
    }
}
