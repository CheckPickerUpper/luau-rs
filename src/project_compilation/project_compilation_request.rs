use crate::ProjectModuleSource;

/// Defines every source module that must be compiled together into one Roblox project layout.
#[derive(Debug)]
pub struct ProjectCompilationRequest {
    source_modules: Vec<ProjectModuleSource>,
}

/// Preserves the caller's complete source set so ordering cannot accidentally change the emitted layout.
impl ProjectCompilationRequest {
    /// @why Requires all source modules up front so identity conflicts and missing entrypoints reject the project before any artifact is accepted.
    #[must_use]
    pub const fn from_source_modules(source_modules: Vec<ProjectModuleSource>) -> Self {
        Self { source_modules }
    }

    pub(crate) fn into_source_modules(self) -> Vec<ProjectModuleSource> {
        self.source_modules
    }
}
