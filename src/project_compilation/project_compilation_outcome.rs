use crate::{CompiledProject, ProjectCompilationRejection};

/// Separates a complete deterministic project artifact set from a typed rejection.
#[derive(Debug)]
pub enum ProjectCompilationOutcome {
    /// Every source module compiled into a strict Luau artifact with a unique Roblox destination.
    Compiled(CompiledProject),
    /// Compilation stopped before accepting any project artifact.
    Rejected(ProjectCompilationRejection),
}
