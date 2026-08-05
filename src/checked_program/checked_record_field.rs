use crate::checked_program::CheckedValueType;

/// Owns one resolved record field declaration.
pub struct CheckedRecordField {
    field_name: String,
    value_type: CheckedValueType,
}

/// Keeps field spelling and type coupled for lookup and Luau alias emission.
impl CheckedRecordField {
    /// Builds a checked field after its declared type resolves in the file context.
    pub(crate) fn from_declaration(declaration: (String, CheckedValueType)) -> Self {
        let (field_name, value_type) = declaration;
        Self {
            field_name,
            value_type,
        }
    }

    /// Gives field lookup the validated field spelling.
    pub(crate) fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Gives literal checks and lowering the resolved field value type.
    pub(crate) const fn value_type(&self) -> &CheckedValueType {
        &self.value_type
    }
}
