/// Names the value categories that survive lowering into typed Luau.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LuauValueType {
    /// Uses Luau's sole numeric representation.
    Number,
    /// Marks a function that produces no values.
    NoReturnedValues,
}
