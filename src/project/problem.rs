use thiserror::Error;

/// Names one reason a project cannot be compiled into a Roblox layout.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProjectCompilationProblem {
    /// No module in the project carries the entrypoint role.
    #[error("project has no entrypoint module")]
    MissingEntrypointModule,

    /// A module identity repeats within one project.
    #[error("module identity {0} appears more than once")]
    DuplicateModuleIdentity(crate::project::ProjectModuleIdentity),

    /// A shared module cannot be an entrypoint.
    #[error("shared module {0} cannot be an entrypoint")]
    SharedEntrypoint(crate::project::ProjectModuleIdentity),

    /// A service-derived module was assigned a role that its service cannot host.
    #[error("module {identity} cannot be placed in its Roblox service as a {role:?}")]
    IllegalServicePlacement {
        /// The service-derived module identity that failed validation.
        identity: crate::project::ProjectModuleIdentity,
        /// The role that was requested for the service.
        role: crate::project::ProjectModuleRole,
    },

    /// Two source modules would publish the same `DataModel` path.
    #[error("multiple modules would publish to {0}")]
    DuplicateOutputPath(String),

    /// A generated module would overwrite a source asset at the same `DataModel` path.
    #[error("generated module overlaps source asset at {0}")]
    SourceOutputOverlap(String),

    /// The wasm bytes for a module failed to decode.
    #[error("module {0} failed to decode")]
    DecodeFailed(String),

    /// Translation rejected the decoded module.
    #[error("module {0} failed to translate")]
    TranslateFailed(String),
}

/// Carries the first project compilation problem encountered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCompilationRejection {
    problem: ProjectCompilationProblem,
}

/// Exposes the typed reason that stopped project compilation.
impl ProjectCompilationRejection {
    /// @why Lets callers wrap a single problem into a rejection outcome.
    #[must_use]
    pub const fn from_problem(problem: ProjectCompilationProblem) -> Self {
        Self { problem }
    }

    /// @why Lets diagnostics name the rejection without exposing internal state.
    #[must_use]
    pub const fn problem(&self) -> &ProjectCompilationProblem {
        &self.problem
    }
}

impl From<ProjectCompilationProblem> for ProjectCompilationRejection {
    fn from(problem: ProjectCompilationProblem) -> Self {
        Self::from_problem(problem)
    }
}
