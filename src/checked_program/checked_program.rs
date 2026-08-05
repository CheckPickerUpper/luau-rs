use crate::checked_program::{CheckedFunction, CheckedRecordDeclaration};

/// Owns every function after name and type validation succeeds.
pub struct CheckedProgram {
    checked_functions: Vec<CheckedFunction>,
    checked_records: Vec<CheckedRecordDeclaration>,
}

/// Keeps checked declarations separate from parsed and generated representations.
impl CheckedProgram {
    /// Builds a checked program only after every function body validates.
    pub(crate) fn from_declarations(
        checked_declarations: (Vec<CheckedRecordDeclaration>, Vec<CheckedFunction>),
    ) -> Self {
        let (checked_records, checked_functions) = checked_declarations;
        Self {
            checked_functions,
            checked_records,
        }
    }

    /// Gives Luau generation the complete validated function set.
    pub(crate) fn functions(&self) -> &[CheckedFunction] {
        &self.checked_functions
    }

    /// Gives Luau generation every file-local record alias before function declarations.
    pub(crate) fn records(&self) -> &[CheckedRecordDeclaration] {
        &self.checked_records
    }
}
