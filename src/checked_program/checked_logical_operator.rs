/// Names short-circuit logical operations after source names and types are checked.
pub enum CheckedLogicalOperator {
    /// Requires both boolean operands to be true.
    Conjunction,
    /// Requires either boolean operand to be true.
    Disjunction,
}
