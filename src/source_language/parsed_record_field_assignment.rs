use crate::{source_language::ParsedExpression, SourceRange};

/// Represents an identifier-rooted record update before its binding and field types validate.
pub struct ParsedRecordFieldAssignment {
    root_binding_name: String,
    root_binding_range: SourceRange,
    first_field_name: String,
    first_field_range: SourceRange,
    remaining_field_names: Vec<(String, SourceRange)>,
    assigned_value: ParsedExpression,
}

/// Preserves a non-empty field path and every diagnostic location needed during checking.
impl ParsedRecordFieldAssignment {
    /// Builds an assignment after parsing an identifier root and at least one postfix field.
    pub(crate) fn from_assignment(
        assignment_parts: (
            String,
            SourceRange,
            String,
            SourceRange,
            Vec<(String, SourceRange)>,
            ParsedExpression,
        ),
    ) -> Self {
        let (
            root_binding_name,
            root_binding_range,
            first_field_name,
            first_field_range,
            remaining_field_names,
            assigned_value,
        ) = assignment_parts;
        Self {
            root_binding_name,
            root_binding_range,
            first_field_name,
            first_field_range,
            remaining_field_names,
            assigned_value,
        }
    }

    /// Gives binding resolution the exact root name spelling.
    pub(crate) fn root_binding_name(&self) -> &str {
        &self.root_binding_name
    }

    /// Gives root resolution and immutable-binding diagnostics the root name range.
    pub(crate) const fn root_binding_range(&self) -> SourceRange {
        self.root_binding_range
    }

    /// Gives path checking the first required field segment and its source range.
    pub(crate) fn first_field(&self) -> (&str, SourceRange) {
        (&self.first_field_name, self.first_field_range)
    }

    /// Gives path checking every segment after the required first field.
    pub(crate) fn remaining_fields(&self) -> &[(String, SourceRange)] {
        &self.remaining_field_names
    }

    /// Gives type checking the value that will replace the final field.
    pub(crate) const fn assigned_value(&self) -> &ParsedExpression {
        &self.assigned_value
    }

    /// Keeps reachability diagnostics tied to the assignment root.
    pub(crate) const fn source_range(&self) -> SourceRange {
        self.root_binding_range
    }
}
