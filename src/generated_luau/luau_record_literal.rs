use crate::generated_luau::LuauRecordFieldInitializer;

/// Owns one Luau table literal whose source shape was validated as a record.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauRecordLiteral {
    field_initializers: Vec<LuauRecordFieldInitializer>,
}

/// Retains the validated table contents for direct expression emission.
impl LuauRecordLiteral {
    /// Builds a target table literal from checked field initializers.
    pub(crate) const fn from_initializers(
        field_initializers: Vec<LuauRecordFieldInitializer>,
    ) -> Self {
        Self { field_initializers }
    }

    /// Gives the writer the field initializers in source order.
    pub(crate) fn field_initializers(&self) -> &[LuauRecordFieldInitializer] {
        &self.field_initializers
    }
}
