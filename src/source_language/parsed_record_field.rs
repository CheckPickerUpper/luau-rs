use crate::{source_language::ParsedValueType, SourceRange};

/// Owns one declared record field and its declared value type.
pub struct ParsedRecordField {
    field_name: String,
    field_name_range: SourceRange,
    value_type: ParsedValueType,
}

/// Keeps record field names and type annotations together for declaration validation.
impl ParsedRecordField {
    /// Preserves the parsed field declaration and its precise source locations.
    pub(crate) fn from_declaration(declaration: (String, SourceRange, ParsedValueType)) -> Self {
        let (field_name, field_name_range, value_type) = declaration;
        Self {
            field_name,
            field_name_range,
            value_type,
        }
    }

    /// Gives semantic registration the unique field spelling.
    pub(crate) fn field_name(&self) -> &str {
        &self.field_name
    }

    /// Gives duplicate-field diagnostics the field's exact name range.
    pub(crate) const fn field_name_range(&self) -> SourceRange {
        self.field_name_range
    }

    /// Gives semantic registration the field's declared value type.
    pub(crate) const fn value_type(&self) -> &ParsedValueType {
        &self.value_type
    }
}
