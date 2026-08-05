/// Names every value type accepted by the first source-language slice.
#[derive(Clone)]
pub enum ParsedValueType {
    /// Represents Luau's numeric value domain.
    Number,
    /// Represents immutable UTF-8 text values accepted by the source language.
    String,
    /// Represents the source language's two truth values.
    Boolean,
    /// Represents a homogeneous, zero-based source array.
    Array(Box<Self>),
    /// Names a record declared in the current source file.
    NamedRecord {
        record_name: String,
        record_name_range: crate::SourceRange,
    },
    /// Represents a function that returns no value.
    NoReturnedValues,
}

impl ParsedValueType {
    pub(crate) fn named_record_parts(&self) -> Option<(&str, crate::SourceRange)> {
        match self {
            Self::NamedRecord {
                record_name,
                record_name_range,
            } => Some((record_name, *record_name_range)),
            Self::Number
            | Self::String
            | Self::Boolean
            | Self::Array(_)
            | Self::NoReturnedValues => None,
        }
    }
}
