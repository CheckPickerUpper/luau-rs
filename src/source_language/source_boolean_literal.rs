/// Names the two boolean literals recognized directly by source tokenization.
#[derive(Clone, Copy)]
pub(crate) enum SourceBooleanLiteral {
    /// Represents the source literal `true`.
    True,
    /// Represents the source literal `false`.
    False,
}
