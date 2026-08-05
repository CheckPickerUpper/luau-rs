use crate::checked_program::CheckedRecordField;

/// Owns one record alias that has been registered for this complete source file.
pub struct CheckedRecordDeclaration {
    record_name: String,
    record_fields: Vec<CheckedRecordField>,
}

/// Preserves the resolved record shape for expression checks and Luau alias generation.
impl CheckedRecordDeclaration {
    /// Builds a record declaration after all field types and names validate.
    pub(crate) fn from_declaration(declaration: (String, Vec<CheckedRecordField>)) -> Self {
        let (record_name, record_fields) = declaration;
        Self {
            record_name,
            record_fields,
        }
    }

    /// Gives record resolution the declaration's unique type name.
    pub(crate) fn record_name(&self) -> &str {
        &self.record_name
    }

    /// Gives literal validation and lowering every checked field.
    pub(crate) fn record_fields(&self) -> &[CheckedRecordField] {
        &self.record_fields
    }
}
