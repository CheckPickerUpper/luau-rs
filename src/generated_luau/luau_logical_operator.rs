/// Names short-circuit logical operations emitted into Luau.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LuauLogicalOperator {
    /// Emits a short-circuit conjunction.
    Conjunction,
    /// Emits a short-circuit disjunction.
    Disjunction,
}
