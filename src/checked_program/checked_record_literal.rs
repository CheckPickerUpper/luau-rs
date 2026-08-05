use crate::checked_program::CheckedRecordFieldInitializer;

/// Owns a validated record construction expression.
pub struct CheckedRecordLiteral {
    field_initializers: Vec<CheckedRecordFieldInitializer>,
}

/// Retains type-checked initializers for table literal lowering.
impl CheckedRecordLiteral {
    /// Builds a record value only after every declared field has exactly one checked initializer.
    pub(crate) const fn from_initializers(
        field_initializers: Vec<CheckedRecordFieldInitializer>,
    ) -> Self {
        Self { field_initializers }
    }

    /// Gives Luau lowering the validated field initializers.
    pub(crate) fn field_initializers(&self) -> &[CheckedRecordFieldInitializer] {
        &self.field_initializers
    }
}
