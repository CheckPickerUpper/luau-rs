/// Names every value type proven by semantic checking.
#[derive(Clone, Copy)]
pub enum CheckedValueType {
    /// Represents a validated numeric value.
    Number,
    /// Represents a validated string value.
    String,
    /// Represents a validated boolean value.
    Boolean,
    /// Represents a validated no-value return.
    NoReturnedValues,
}
