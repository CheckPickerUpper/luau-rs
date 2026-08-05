/// Names the value categories that survive lowering into typed Luau.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LuauValueType {
    /// Uses Luau's sole numeric representation.
    Number,
    /// Uses Luau's immutable string value representation.
    String,
    /// Uses Luau's two-valued boolean representation.
    Boolean,
    /// Marks a function that produces no values.
    NoReturnedValues,
}
