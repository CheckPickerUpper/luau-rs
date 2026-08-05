/// Names equality operations emitted into Luau.
#[derive(Debug, PartialEq, Eq)]
pub enum LuauEqualityOperator {
    /// Emits equality.
    Equal,
    /// Emits inequality.
    NotEqual,
}
