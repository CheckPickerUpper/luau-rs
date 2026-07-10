/// Names the boolean literals preserved by semantic checking.
#[derive(Clone, Copy)]
pub(crate) enum CheckedBooleanLiteral {
    /// Represents a checked true literal.
    True,
    /// Represents a checked false literal.
    False,
}
