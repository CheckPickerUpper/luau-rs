use crate::checked_program::CheckedValueType;

/// Owns one validated function parameter.
pub struct CheckedParameter {
    parameter_name: String,
    value_type: CheckedValueType,
}

/// Keeps checked parameter declarations distinct from local statements.
impl CheckedParameter {
    /// Builds a parameter after its type participates in signature validation.
    pub(crate) fn from_checked_declaration(
        checked_declaration: (String, CheckedValueType),
    ) -> Self {
        let (parameter_name, value_type) = checked_declaration;
        Self {
            parameter_name,
            value_type,
        }
    }

    /// Gives Luau generation the validated parameter name.
    pub(crate) fn parameter_name(&self) -> &str {
        &self.parameter_name
    }

    /// Gives Luau generation the validated parameter type.
    pub(crate) const fn value_type(&self) -> CheckedValueType {
        self.value_type
    }
}
