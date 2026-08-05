use crate::generated_luau::LuauExpression;

/// Represents one target-language update of a non-empty record field path.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauRecordFieldAssignment {
    root_binding_name: String,
    first_field_name: String,
    remaining_field_names: Vec<String>,
    assigned_value: LuauExpression,
}

/// Keeps target emission from reconstructing an assignment target from general expressions.
impl LuauRecordFieldAssignment {
    /// Builds an update only after the source assignment has a mutable record root and valid path.
    pub(crate) fn from_assignment(
        assignment_parts: (String, String, Vec<String>, LuauExpression),
    ) -> Self {
        let (root_binding_name, first_field_name, remaining_field_names, assigned_value) =
            assignment_parts;
        Self {
            root_binding_name,
            first_field_name,
            remaining_field_names,
            assigned_value,
        }
    }

    /// Gives serialization the root local name.
    pub(crate) fn root_binding_name(&self) -> &str {
        &self.root_binding_name
    }

    /// Gives serialization the first field that makes this path non-empty.
    pub(crate) fn first_field_name(&self) -> &str {
        &self.first_field_name
    }

    /// Gives serialization the rest of the field path.
    pub(crate) fn remaining_field_names(&self) -> &[String] {
        &self.remaining_field_names
    }

    /// Gives serialization the checked value written to the final field.
    pub(crate) const fn assigned_value(&self) -> &LuauExpression {
        &self.assigned_value
    }
}
