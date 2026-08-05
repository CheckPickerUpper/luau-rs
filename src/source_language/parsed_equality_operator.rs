/// Names equality operations accepted by the source language.
pub enum ParsedEqualityOperator {
    /// Tests whether matching typed values are equal.
    Equal,
    /// Tests whether matching typed values are different.
    NotEqual,
}
