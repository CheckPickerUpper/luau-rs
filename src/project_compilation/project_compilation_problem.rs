use crate::{CompilationRejection, ProjectModuleIdentity, SourceRange};

/// Names a project-level rejection without forcing callers to parse diagnostic strings.
#[derive(Debug)]
pub enum ProjectCompilationProblem {
    /// The project supplies no executable source module for the Roblox runtime to start.
    MissingEntrypointModule,
    /// A shared module attempted to start itself even though client and server must initialize it independently through imports.
    SharedModuleCannotBeEntrypoint {
        /// Identifies the shared source module that declared an invalid role.
        module_identity: ProjectModuleIdentity,
    },
    /// A source module path cannot map safely and deterministically into a Roblox instance tree.
    InvalidModuleIdentity {
        /// Identifies the source module whose path contains an unsupported segment.
        module_identity: ProjectModuleIdentity,
    },
    /// Two source modules would claim the same generated Roblox location.
    DuplicateModuleIdentity {
        /// Identifies the repeated source module identity.
        module_identity: ProjectModuleIdentity,
    },
    /// An import names no module with the exact requested execution side and path.
    ImportedModuleNotFound {
        /// Identifies the source module that declared the unresolved import.
        importing_module_identity: ProjectModuleIdentity,
        /// Preserves the exact identity required by the source import.
        target_module_identity: ProjectModuleIdentity,
        /// Highlights the whole import declaration that could not resolve.
        source_range: SourceRange,
    },
    /// An import attempts to require executable code instead of a library surface.
    ImportedModuleIsEntrypoint {
        /// Identifies the source module that declared the invalid import.
        importing_module_identity: ProjectModuleIdentity,
        /// Identifies the entrypoint module that cannot be required.
        target_module_identity: ProjectModuleIdentity,
        /// Highlights the whole import declaration that selected an entrypoint.
        source_range: SourceRange,
    },
    /// An import crosses a Roblox execution boundary unavailable to its importing module.
    ImportExecutionSideNotAllowed {
        /// Identifies the source module making the unavailable cross-side request.
        importing_module_identity: ProjectModuleIdentity,
        /// Identifies the module located on the forbidden execution side.
        target_module_identity: ProjectModuleIdentity,
        /// Highlights the whole import declaration that crosses the boundary.
        source_range: SourceRange,
    },
    /// An import names no declaration in the resolved target library.
    ImportedFunctionNotFound {
        /// Identifies the source module that requested the unavailable function.
        importing_module_identity: ProjectModuleIdentity,
        /// Identifies the library searched for the function.
        target_module_identity: ProjectModuleIdentity,
        /// Preserves the source function name for a diagnostic without string parsing.
        function_name: String,
        /// Highlights only the imported function segment.
        source_range: SourceRange,
    },
    /// An import reaches a library function that its module did not export publicly.
    ImportedFunctionIsPrivate {
        /// Identifies the source module that requested the private declaration.
        importing_module_identity: ProjectModuleIdentity,
        /// Identifies the library that owns the private declaration.
        target_module_identity: ProjectModuleIdentity,
        /// Preserves the source function name for a diagnostic without string parsing.
        function_name: String,
        /// Highlights only the imported function segment.
        source_range: SourceRange,
    },
    /// An import would create a local callable name that is already owned by this module.
    ImportNameCollidesWithLocalDeclaration {
        /// Identifies the module whose local namespace would become ambiguous.
        importing_module_identity: ProjectModuleIdentity,
        /// Preserves the local callable name that would collide.
        function_name: String,
        /// Highlights only the imported function segment.
        source_range: SourceRange,
    },
    /// Resolved library dependencies form a deterministic closed module path.
    ImportCycle {
        /// Lists the repeated starting identity again at the end so the cycle is closed.
        cycle_path: Vec<ProjectModuleIdentity>,
    },
    /// A source-level rejection occurred within the named project module.
    SourceModuleRejected {
        /// Identifies the source module that failed language compilation.
        module_identity: ProjectModuleIdentity,
        /// Preserves the typed source diagnostic rather than replacing it with a project string.
        compilation_rejection: CompilationRejection,
    },
}
