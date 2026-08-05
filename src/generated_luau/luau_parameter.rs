use crate::generated_luau::LuauValueType;

/// Owns one generated function parameter and its checked Luau type.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauParameter {
    parameter_name: String,
    value_type: LuauValueType,
}

/// Keeps parameter construction and rendering access within the lowering phase.
impl LuauParameter {
    /// Builds a parameter from the checked name and lowered value type.
    pub(crate) fn from_name_and_type(name_and_type: (String, LuauValueType)) -> Self {
        let (parameter_name, value_type) = name_and_type;
        Self {
            parameter_name,
            value_type,
        }
    }

    /// Exposes the generated spelling to the Luau writer.
    pub(crate) fn parameter_name(&self) -> &str {
        &self.parameter_name
    }

    /// Exposes the lowered annotation to the Luau writer.
    pub(crate) fn value_type(&self) -> LuauValueType {
        self.value_type.clone()
    }
}
