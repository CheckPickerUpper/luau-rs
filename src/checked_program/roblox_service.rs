use crate::ModuleExecutionSide;

/// Keeps the V1 Roblox service surface closed so source spellings cannot become runtime strings.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RobloxService {
    Players,
    ReplicatedStorage,
    UserInputService,
    DataStoreService,
    ServerScriptService,
}

/// Keeps service lookup, type spelling, and execution-side policy in one versioned compiler catalog.
impl RobloxService {
    pub(crate) fn from_type_name(type_name: &str) -> Option<Self> {
        match type_name {
            "Players" => Some(Self::Players),
            "ReplicatedStorage" => Some(Self::ReplicatedStorage),
            "UserInputService" => Some(Self::UserInputService),
            "DataStoreService" => Some(Self::DataStoreService),
            "ServerScriptService" => Some(Self::ServerScriptService),
            _ => None,
        }
    }

    pub(crate) const fn canonical_name(self) -> &'static str {
        match self {
            Self::Players => "Players",
            Self::ReplicatedStorage => "ReplicatedStorage",
            Self::UserInputService => "UserInputService",
            Self::DataStoreService => "DataStoreService",
            Self::ServerScriptService => "ServerScriptService",
        }
    }

    pub(crate) const fn is_available_on(self, execution_side: ModuleExecutionSide) -> bool {
        match (self, execution_side) {
            (Self::Players | Self::ReplicatedStorage, _)
            | (Self::UserInputService, ModuleExecutionSide::Client)
            | (Self::DataStoreService | Self::ServerScriptService, ModuleExecutionSide::Server) => {
                true
            }
            (Self::UserInputService, ModuleExecutionSide::Server | ModuleExecutionSide::Shared)
            | (
                Self::DataStoreService | Self::ServerScriptService,
                ModuleExecutionSide::Client | ModuleExecutionSide::Shared,
            ) => false,
        }
    }
}
