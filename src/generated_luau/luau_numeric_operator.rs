/// Names numeric operators emitted into Luau.
#[derive(Debug, PartialEq, Eq)]
pub enum LuauNumericOperator {
    /// Emits addition.
    Addition,
    /// Emits subtraction.
    Subtraction,
    /// Emits multiplication.
    Multiplication,
    /// Emits division.
    Division,
}
