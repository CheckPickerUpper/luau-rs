/// Names every value type proven by semantic checking.
#[derive(Clone, PartialEq, Eq)]
pub enum CheckedValueType {
    /// Represents a validated numeric value.
    Number,
    /// Represents a validated string value.
    String,
    /// Represents a validated boolean value.
    Boolean,
    /// Represents a homogeneous, zero-based source array.
    Array(Box<Self>),
    /// Represents a checked reference to a file-private record alias.
    NamedRecord(String),
    /// Represents a service that can only be used as a directly acquired local value.
    RobloxService(super::roblox_service::RobloxService),
    /// Represents one catalogued Roblox Instance class.
    RobloxInstance(super::roblox_instance::RobloxInstance),
    /// Represents a validated no-value return.
    NoReturnedValues,
}
