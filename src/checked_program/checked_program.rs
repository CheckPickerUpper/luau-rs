use crate::checked_program::CheckedFunction;

/// Owns every function after name and type validation succeeds.
pub(crate) struct CheckedProgram {
    checked_functions: Vec<CheckedFunction>,
}

/// Keeps checked declarations separate from parsed and generated representations.
impl CheckedProgram {
    /// Builds a checked program only after every function body validates.
    pub(crate) fn from_functions(checked_functions: Vec<CheckedFunction>) -> Self {
        Self { checked_functions }
    }

    /// Gives Luau generation the complete validated function set.
    pub(crate) fn functions(&self) -> &[CheckedFunction] {
        &self.checked_functions
    }
}
