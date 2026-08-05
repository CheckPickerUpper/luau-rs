use crate::generated_luau::LuauValueType;

/// Owns one field inside a strict Luau table alias.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauRecordField {
    field_name: String,
    value_type: LuauValueType,
}

/// Couples each target field spelling to its target type annotation.
impl LuauRecordField {
    /// Builds one target record field from its checked declaration.
    pub(crate) fn from_field(field: (String, LuauValueType)) -> Self {
        let (field_name, value_type) = field;
        Self {
            field_name,
            value_type,
        }
    }

    /// Gives the writer the field spelling.
    pub(crate) fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Gives the writer the field type annotation.
    pub(crate) const fn value_type(&self) -> &LuauValueType {
        &self.value_type
    }
}
