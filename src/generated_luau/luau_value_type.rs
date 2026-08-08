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
    /// Uses Luau's typed callback notation.
    Function {
        parameter_types: Vec<Self>,
        returned_value_type: Box<Self>,
    },
    /// Uses a file-local Luau table type alias.
    NamedRecord(String),
    /// Uses the Roblox engine's built-in service type spelling.
    RobloxService(String),
    /// Uses the Roblox engine's built-in Instance class type spelling.
    RobloxInstance(String),
    /// Uses Roblox's connection object type spelling.
    RobloxConnection,
    /// Marks a function that produces no values.
    NoReturnedValues,
}
