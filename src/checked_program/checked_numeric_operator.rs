/// Names numeric operations after source names and types are checked.
pub(crate) enum CheckedNumericOperator {
    /// Adds the two numeric operands.
    Addition,
    /// Subtracts the right numeric operand from the left numeric operand.
    Subtraction,
    /// Multiplies the two numeric operands.
    Multiplication,
    /// Divides the left numeric operand by the right numeric operand.
    Division,
}
