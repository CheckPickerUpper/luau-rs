/// Names equality operations after source names and types are checked.
pub enum CheckedEqualityOperator {
    /// Tests whether matching typed values are equal.
    Equal,
    /// Tests whether matching typed values are different.
    NotEqual,
}
