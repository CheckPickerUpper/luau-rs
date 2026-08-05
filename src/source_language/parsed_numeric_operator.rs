/// Names the numeric operators accepted by the source language.
pub enum ParsedNumericOperator {
    /// Adds the two operands.
    Addition,
    /// Subtracts the right operand from the left operand.
    Subtraction,
    /// Multiplies the two operands.
    Multiplication,
    /// Divides the left operand by the right operand.
    Division,
}
