/// Names numeric comparisons accepted by the source language.
pub enum ParsedComparisonOperator {
    /// Tests whether the left number is smaller than the right number.
    LessThan,
    /// Tests whether the left number is no greater than the right number.
    LessThanOrEqual,
    /// Tests whether the left number is greater than the right number.
    GreaterThan,
    /// Tests whether the left number is at least the right number.
    GreaterThanOrEqual,
}
