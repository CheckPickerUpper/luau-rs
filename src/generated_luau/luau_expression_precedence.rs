/// Orders generated Luau expressions according to Luau's operator binding rules.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum LuauExpressionPrecedence {
    /// Identifies a short-circuit disjunction.
    Disjunction,
    /// Identifies a short-circuit conjunction.
    Conjunction,
    /// Identifies comparison and equality operations.
    Comparison,
    /// Identifies addition and subtraction.
    Additive,
    /// Identifies multiplication and division.
    Multiplicative,
    /// Identifies logical negation.
    Negation,
    /// Identifies literals, names, and calls.
    Primary,
}
