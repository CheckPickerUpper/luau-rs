use crate::checked_program::CheckedExpression;

/// Owns one validated record field initializer.
pub struct CheckedRecordFieldInitializer {
    field_name: String,
    initialized_value: CheckedExpression,
}

/// Keeps a field spelling with its checked expression for direct Luau table lowering.
impl CheckedRecordFieldInitializer {
    /// Builds an initializer only after it matches the field's declared type.
    pub(crate) fn from_initializer(initializer: (String, CheckedExpression)) -> Self {
        let (field_name, initialized_value) = initializer;
        Self {
            field_name,
            initialized_value,
        }
    }

    /// Gives Luau table lowering the declared field name.
    pub(crate) fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Gives Luau table lowering the checked field value.
    pub(crate) const fn initialized_value(&self) -> &CheckedExpression {
        &self.initialized_value
    }
}
