use crate::generated_luau::LuauRecordField;

/// Owns one strict Luau table type alias.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauRecordAlias {
    record_name: String,
    record_fields: Vec<LuauRecordField>,
}

/// Keeps a record's aliases and fields together for declaration emission.
impl LuauRecordAlias {
    /// Builds a target alias from a checked source record declaration.
    pub(crate) fn from_alias(alias: (String, Vec<LuauRecordField>)) -> Self {
        let (record_name, record_fields) = alias;
        Self {
            record_name,
            record_fields,
        }
    }

    /// Gives the writer the emitted alias name.
    pub(crate) fn record_name(&self) -> &str {
        &self.record_name
    }

    /// Gives the writer the ordered field declarations.
    pub(crate) fn record_fields(&self) -> &[LuauRecordField] {
        &self.record_fields
    }
}
