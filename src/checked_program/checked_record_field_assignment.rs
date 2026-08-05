use crate::checked_program::CheckedExpression;

/// Represents a record update whose mutable root and complete field path have been validated.
pub struct CheckedRecordFieldAssignment {
    root_binding_name: String,
    first_field_name: String,
    remaining_field_names: Vec<String>,
    assigned_value: CheckedExpression,
}

/// Retains only the target names required by Luau lowering after source-range diagnostics finish.
impl CheckedRecordFieldAssignment {
    /// Builds a checked update after every field resolves from a mutable record root.
    pub(crate) fn from_assignment(
        assignment_parts: (String, String, Vec<String>, CheckedExpression),
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

    /// Gives Luau lowering the validated root binding spelling.
    pub(crate) fn root_binding_name(&self) -> &str {
        &self.root_binding_name
    }

    /// Gives Luau lowering the first segment that keeps the path non-empty.
    pub(crate) fn first_field_name(&self) -> &str {
        &self.first_field_name
    }

    /// Gives Luau lowering the remaining validated field segments.
    pub(crate) fn remaining_field_names(&self) -> &[String] {
        &self.remaining_field_names
    }

    /// Gives Luau lowering the expression that replaces the final field value.
    pub(crate) const fn assigned_value(&self) -> &CheckedExpression {
        &self.assigned_value
    }
}
