/// Names numeric comparisons emitted into Luau.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LuauComparisonOperator {
    /// Emits a less-than comparison.
    LessThan,
    /// Emits a less-than-or-equal comparison.
    LessThanOrEqual,
    /// Emits a greater-than comparison.
    GreaterThan,
    /// Emits a greater-than-or-equal comparison.
    GreaterThanOrEqual,
}
