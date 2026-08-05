/// Carries an observed number of function arguments without exposing mutable count state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArgumentCount {
    argument_count: usize,
}

/// Keeps argument-count construction inside compiler-observed collection boundaries.
impl ArgumentCount {
    /// Preserves the argument total observed at a checked call boundary.
    pub(crate) const fn from_number_of_arguments(number_of_arguments: usize) -> Self {
        Self {
            argument_count: number_of_arguments,
        }
    }

    /// @why Returns the observed argument total so diagnostic presenters can explain the rejected call precisely.
    #[must_use]
    pub const fn number_of_arguments(&self) -> usize {
        self.argument_count
    }
}
