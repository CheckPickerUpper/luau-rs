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
    /// Represents a typed callback value with ordered parameters and a return contract.
    Function {
        parameter_types: Vec<Self>,
        returned_value_type: Box<Self>,
    },
    /// Names a record declared in the current source file.
    NamedRecord {
        record_name: String,
        record_name_range: crate::SourceRange,
    },
    /// Represents a function that returns no value.
    NoReturnedValues,
}
