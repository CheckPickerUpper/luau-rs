//! Source identities and generated Roblox destinations.

use super::{ModuleExecutionSide, ProjectModuleRole, RobloxService};
use std::fmt;

/// Identifies one source module and its Roblox execution boundary.
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum ProjectModuleIdentity {
    /// Places a lower-level CLI module beneath `ServerScriptService`.
    Server {
        /// Is a slash-separated path relative to the server source root.
        module_path: String,
    },
    /// Places a lower-level CLI module beneath `StarterPlayerScripts`.
    Client {
        /// Is a slash-separated path relative to the client source root.
        module_path: String,
    },
    /// Places a lower-level CLI module beneath `ReplicatedStorage`.
    Shared {
        /// Is a slash-separated path relative to the shared source root.
        module_path: String,
    },
    /// Places a discovered module under its owning Roblox service.
    RobloxService {
        /// The recognized service that owns the module.
        service: RobloxService,
        /// Is a slash-separated path beneath that service.
        module_path: String,
    },
}

/// Keeps source identity decisions centralized before project layout derives target paths.
impl ProjectModuleIdentity {
    /// Lets compilation select Roblox placement without duplicating execution-side checks.
    #[must_use]
    pub const fn execution_side(&self) -> ModuleExecutionSide {
        match self {
            Self::Server { .. } => ModuleExecutionSide::Server,
            Self::Client { .. } => ModuleExecutionSide::Client,
            Self::Shared { .. } => ModuleExecutionSide::Shared,
            Self::RobloxService { service, .. } => service.execution_side(),
        }
    }

    /// Lets diagnostics name the offending source module without exposing storage.
    #[must_use]
    pub fn module_path(&self) -> &str {
        match self {
            Self::Server { module_path }
            | Self::Client { module_path }
            | Self::Shared { module_path }
            | Self::RobloxService { module_path, .. } => module_path,
        }
    }

    /// Derives the Roblox destination path for a module identity and role.
    pub(crate) fn output_path_text(&self, module_role: ProjectModuleRole) -> Option<String> {
        let module_path = self.module_path();
        match (self, module_role) {
            (Self::Server { .. }, ProjectModuleRole::Entrypoint) => {
                Some(format!("ServerScriptService/{module_path}.server.luau"))
            }
            (Self::Client { .. }, ProjectModuleRole::Entrypoint) => Some(format!(
                "StarterPlayer/StarterPlayerScripts/{module_path}.client.luau"
            )),
            (Self::Server { .. }, ProjectModuleRole::Library) => {
                Some(format!("ServerScriptService/{module_path}.luau"))
            }
            (Self::Client { .. }, ProjectModuleRole::Library) => Some(format!(
                "StarterPlayer/StarterPlayerScripts/{module_path}.luau"
            )),
            (Self::Shared { .. }, ProjectModuleRole::Library) => {
                Some(format!("ReplicatedStorage/{module_path}.luau"))
            }
            (Self::Shared { .. }, ProjectModuleRole::Entrypoint) => None,
            (Self::RobloxService { service, .. }, role) => {
                service_output_path(*service, module_path, role)
            }
        }
    }
}

fn service_output_path(
    service: RobloxService,
    module_path: &str,
    module_role: ProjectModuleRole,
) -> Option<String> {
    match service.module_role() {
        expected_role if expected_role == module_role => {
            let suffix = match module_role {
                ProjectModuleRole::Entrypoint => match service.execution_side() {
                    ModuleExecutionSide::Server => ".server.luau",
                    ModuleExecutionSide::Client => ".client.luau",
                    ModuleExecutionSide::Shared => ".luau",
                },
                ProjectModuleRole::Library => ".luau",
            };
            Some(format!(
                "{}/{module_path}{suffix}",
                service.data_model_path()
            ))
        }
        _ => None,
    }
}

impl fmt::Display for ProjectModuleIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let execution_side = match self {
            Self::Server { .. } => "server",
            Self::Client { .. } => "client",
            Self::Shared { .. } => "shared",
            Self::RobloxService { service, .. } => {
                return write!(formatter, "{service:?}/{}", self.module_path())
            }
        };
        write!(formatter, "{execution_side}:{}", self.module_path())
    }
}
