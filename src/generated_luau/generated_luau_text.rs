/// Owns complete Luau emitted only after syntax and semantic checks succeed.
#[derive(Debug, PartialEq, Eq)]
pub struct GeneratedLuauText {
    text: String,
}

/// Keeps emitted text immutable until a caller deliberately takes ownership.
impl GeneratedLuauText {
    /// Restricts construction to the renderer-backed compilation pipeline.
    pub(crate) fn from_text(text: String) -> Self {
        Self { text }
    }

    /// @why Transfers the validated artifact so callers can write or execute it without copying the generated program.
    pub fn into_text(self) -> String {
        self.text
    }
}
