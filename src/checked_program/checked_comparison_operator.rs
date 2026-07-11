/// Names numeric comparisons after source names and types are checked.
pub(crate) enum CheckedComparisonOperator {
    /// Tests whether the left number is smaller than the right number.
    LessThan,
    /// Tests whether the left number is no greater than the right number.
    LessThanOrEqual,
    /// Tests whether the left number is greater than the right number.
    GreaterThan,
    /// Tests whether the left number is at least the right number.
    GreaterThanOrEqual,
}
