/// Names the value categories that survive lowering into typed Luau.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LuauValueType {
    /// Uses Luau's sole numeric representation.
    Number,
    /// Uses Luau's immutable string value representation.
    String,
    /// Uses Luau's two-valued boolean representation.
    Boolean,
    /// Uses Luau's homogeneous table notation.
    Array(Box<Self>),
    /// Uses a file-local Luau table type alias.
    NamedRecord(String),
    /// Uses the Roblox engine's built-in service type spelling.
    RobloxService(String),
    /// Uses the Roblox engine's built-in Instance class type spelling.
    RobloxInstance(String),
    /// Marks a function that produces no values.
    NoReturnedValues,
}
