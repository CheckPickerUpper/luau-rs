use crate::{source_language::ParsedRecordFieldInitializer, SourceRange};

/// Owns one record literal before its declaration and field values are validated.
pub struct ParsedRecordLiteral {
    record_name: String,
    record_name_range: SourceRange,
    field_initializers: Vec<ParsedRecordFieldInitializer>,
    literal_range: SourceRange,
}

/// Retains literal field locations so each record diagnostic can identify its responsible token.
impl ParsedRecordLiteral {
    /// Preserves the complete parsed record construction expression.
    pub(crate) fn from_literal(
        literal: (
            String,
            SourceRange,
            Vec<ParsedRecordFieldInitializer>,
            SourceRange,
        ),
    ) -> Self {
        let (record_name, record_name_range, field_initializers, literal_range) = literal;
        Self {
            record_name,
            record_name_range,
            field_initializers,
            literal_range,
        }
    }

    /// Gives type checking the declared record name.
    pub(crate) fn record_name(&self) -> &str {
        &self.record_name
    }

    /// Gives unknown-record and missing-field diagnostics the literal's type name location.
    pub(crate) const fn record_name_range(&self) -> SourceRange {
        self.record_name_range
    }

    /// Gives type checking the source-ordered initializer set.
    pub(crate) fn field_initializers(&self) -> &[ParsedRecordFieldInitializer] {
        &self.field_initializers
    }

    /// Gives enclosing expressions the literal's complete range.
    pub(crate) const fn literal_range(&self) -> SourceRange {
        self.literal_range
    }
}
