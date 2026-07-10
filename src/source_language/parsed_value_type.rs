/// Names every value type accepted by the first source-language slice.
#[derive(Clone, Copy)]
pub(crate) enum ParsedValueType {
    /// Represents Luau's numeric value domain.
    Number,
    /// Represents a function that returns no value.
    NoReturnedValues,
}
