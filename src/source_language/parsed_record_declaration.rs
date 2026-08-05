use crate::{source_language::ParsedRecordField, SourceRange};

/// Owns one file-private named record declaration.
pub struct ParsedRecordDeclaration {
    name: String,
    name_range: SourceRange,
    fields: Vec<ParsedRecordField>,
}

/// Keeps record declaration metadata available until semantic registration resolves its fields.
impl ParsedRecordDeclaration {
    /// Preserves the parsed declaration together with all of its source-located fields.
    pub(crate) fn from_declaration(
        declaration: (String, SourceRange, Vec<ParsedRecordField>),
    ) -> Self {
        let (name, name_range, fields) = declaration;
        Self {
            name,
            name_range,
            fields,
        }
    }

    /// Gives semantic registration the record type's source spelling.
    pub(crate) fn record_name(&self) -> &str {
        &self.name
    }

    /// Gives diagnostic construction the declared record name location.
    pub(crate) const fn record_name_range(&self) -> SourceRange {
        self.name_range
    }

    /// Gives semantic registration every declared field in source order.
    pub(crate) fn record_fields(&self) -> &[ParsedRecordField] {
        &self.fields
    }
}
