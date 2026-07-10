/// Names every value type accepted by the first source-language slice.
#[derive(Clone, Copy)]
pub(crate) enum ParsedValueType {
    /// Represents Luau's numeric value domain.
    Number,
    /// Represents immutable UTF-8 text values accepted by the source language.
    String,
    /// Represents the source language's two truth values.
    Boolean,
    /// Represents a function that returns no value.
    NoReturnedValues,
}
