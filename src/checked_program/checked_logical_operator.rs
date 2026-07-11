/// Names short-circuit logical operations after source names and types are checked.
pub(crate) enum CheckedLogicalOperator {
    /// Requires both boolean operands to be true.
    Conjunction,
    /// Requires either boolean operand to be true.
    Disjunction,
}
