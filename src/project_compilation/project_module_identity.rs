use crate::{ModuleExecutionSide, ProjectModuleRole};

/// Identifies one source module and its Roblox execution boundary.
#[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum ProjectModuleIdentity {
    /// Places the module beneath `ServerScriptService`.
    Server {
        /// Is a slash-separated path relative to the server source root.
        module_path: String,
    },
    /// Places the module beneath `StarterPlayerScripts`.
    Client {
        /// Is a slash-separated path relative to the client source root.
        module_path: String,
    },
    /// Places the module beneath `ReplicatedStorage` for independent client and server initialization.
    Shared {
        /// Is a slash-separated path relative to the shared source root.
        module_path: String,
    },
}

/// Keeps source identity decisions centralized before project layout derives target paths.
impl ProjectModuleIdentity {
    /// @why Lets compilation select Roblox placement without duplicating execution-side checks throughout project handling.
    #[must_use]
    pub const fn execution_side(&self) -> ModuleExecutionSide {
        match self {
            Self::Server { .. } => ModuleExecutionSide::Server,
            Self::Client { .. } => ModuleExecutionSide::Client,
            Self::Shared { .. } => ModuleExecutionSide::Shared,
        }
    }

    /// @why Lets diagnostics name the offending source module without exposing how each execution side stores it.
    #[must_use]
    pub fn module_path(&self) -> &str {
        match self {
            Self::Server { module_path }
            | Self::Client { module_path }
            | Self::Shared { module_path } => module_path,
        }
    }

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
        }
    }
}
