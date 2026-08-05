use crate::generated_luau::LuauExpression;

/// Owns one field initializer inside a Luau table literal.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauRecordFieldInitializer {
    field_name: String,
    initialized_value: LuauExpression,
}

/// Keeps a field spelling coupled to its lowered value expression.
impl LuauRecordFieldInitializer {
    /// Builds a table field after its source initializer has lowered.
    pub(crate) fn from_initializer(initializer: (String, LuauExpression)) -> Self {
        let (field_name, initialized_value) = initializer;
        Self {
            field_name,
            initialized_value,
        }
    }

    /// Gives the writer the target field name.
    pub(crate) fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Gives the writer the lowered field value.
    pub(crate) const fn initialized_value(&self) -> &LuauExpression {
        &self.initialized_value
    }
}
