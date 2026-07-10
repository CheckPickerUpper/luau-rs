use crate::{CompilationRejection, GeneratedLuauText};

/// Forces callers to handle generated Luau and typed rejection diagnostics separately.
#[derive(Debug, PartialEq, Eq)]
pub enum CompilationOutcome {
    /// Compilation completed with a whole validated Luau program.
    Compiled(GeneratedLuauText),
    /// Compilation stopped before emission because validation failed.
    Rejected(CompilationRejection),
}
