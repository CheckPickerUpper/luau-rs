/// Identifies the half-open byte range associated with source syntax.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceRange {
    start_byte: usize,
    end_byte: usize,
}

/// Preserves source-location invariants at compiler phase boundaries.
impl SourceRange {
    /// Keeps tokenizer-controlled byte boundaries together for later diagnostics.
    pub(crate) const fn from_byte_range(byte_range: (usize, usize)) -> Self {
        let (start_byte, end_byte) = byte_range;
        Self {
            start_byte,
            end_byte,
        }
    }

    /// @why Exposes the inclusive byte boundary so editors can begin a diagnostic highlight at the compiler's exact location.
    #[must_use]
    pub const fn start_byte(&self) -> usize {
        self.start_byte
    }

    /// @why Exposes the exclusive byte boundary so editors can end a diagnostic highlight without guessing character width.
    #[must_use]
    pub const fn end_byte(&self) -> usize {
        self.end_byte
    }
}
