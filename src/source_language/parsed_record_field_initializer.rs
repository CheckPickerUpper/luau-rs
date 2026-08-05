use crate::{source_language::ParsedExpression, SourceRange};

/// Owns one source field initializer inside a record literal.
pub struct ParsedRecordFieldInitializer {
    field_name: String,
    field_name_range: SourceRange,
    initialized_value: ParsedExpression,
}

/// Retains each literal field's name and value for field-specific validation.
impl ParsedRecordFieldInitializer {
    /// Preserves the field spelling, its location, and its value expression.
    pub(crate) fn from_initializer(initializer: (String, SourceRange, ParsedExpression)) -> Self {
        let (field_name, field_name_range, initialized_value) = initializer;
        Self {
            field_name,
            field_name_range,
            initialized_value,
        }
    }

    /// Gives record validation the initialized field name.
    pub(crate) fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Gives unknown and duplicate field diagnostics the name's exact range.
    pub(crate) const fn field_name_range(&self) -> SourceRange {
        self.field_name_range
    }

    /// Gives expression checking the initializer to type-check.
    pub(crate) const fn initialized_value(&self) -> &ParsedExpression {
        &self.initialized_value
    }
}
