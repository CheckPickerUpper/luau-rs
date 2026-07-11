/// Names short-circuit logical operations accepted by the source language.
pub(crate) enum ParsedLogicalOperator {
    /// Requires both boolean operands to be true.
    Conjunction,
    /// Requires either boolean operand to be true.
    Disjunction,
}
