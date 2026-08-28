//! Recognized Roblox service containers and their compiler-owned semantics.

use super::{ModuleExecutionSide, ProjectModuleRole};

/// Names the Roblox service containers accepted at the root of a project tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum RobloxService {
    /// Owns server entrypoint scripts.
    ServerScriptService,
    /// Owns server-only `ModuleScripts`.
    ServerStorage,
    /// Owns `ModuleScripts` replicated to both server and clients.
    ReplicatedStorage,
    /// Owns server scripts placed in the 3D world.
    Workspace,
    /// Owns client-first scripts.
    ReplicatedFirst,
    /// Owns client GUI scripts.
    StarterGui,
    /// Owns scripts that run for each player.
    StarterPlayerScripts,
    /// Owns scripts that run for each character.
    StarterCharacterScripts,
}

/// Adds the compiler's fixed path, role, and runtime mapping to each service.
impl RobloxService {
    /// Returns the `DataModel` path used to publish this service's children.
    pub(crate) const fn data_model_path(self) -> &'static str {
        match self {
            Self::ServerScriptService => "ServerScriptService",
            Self::ServerStorage => "ServerStorage",
            Self::ReplicatedStorage => "ReplicatedStorage",
            Self::Workspace => "Workspace",
            Self::ReplicatedFirst => "ReplicatedFirst",
            Self::StarterGui => "StarterGui",
            Self::StarterPlayerScripts => "StarterPlayer/StarterPlayerScripts",
            Self::StarterCharacterScripts => "StarterPlayer/StarterCharacterScripts",
        }
    }

    /// Returns the role Roblox gives a wasm module under this service.
    pub(crate) const fn module_role(self) -> ProjectModuleRole {
        match self {
            Self::ServerStorage | Self::ReplicatedStorage => ProjectModuleRole::Library,
            Self::ServerScriptService
            | Self::Workspace
            | Self::ReplicatedFirst
            | Self::StarterGui
            | Self::StarterPlayerScripts
            | Self::StarterCharacterScripts => ProjectModuleRole::Entrypoint,
        }
    }

    /// Returns the execution side implied by this service.
    pub(crate) const fn execution_side(self) -> ModuleExecutionSide {
        match self {
            Self::ReplicatedStorage => ModuleExecutionSide::Shared,
            Self::ReplicatedFirst
            | Self::StarterGui
            | Self::StarterPlayerScripts
            | Self::StarterCharacterScripts => ModuleExecutionSide::Client,
            Self::ServerScriptService | Self::ServerStorage | Self::Workspace => {
                ModuleExecutionSide::Server
            }
        }
    }
}
