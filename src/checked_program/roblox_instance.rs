use crate::checked_program::CheckedValueType;

/// Keeps constructible Roblox classes and their typed members in one closed compiler catalog.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RobloxInstance {
    /// Represents a physical Roblox part.
    Part,
    /// Represents a hierarchy folder.
    Folder,
    /// Represents a model container.
    Model,
    /// Represents an asynchronous remote event.
    RemoteEvent,
    /// Represents a callable remote function.
    RemoteFunction,
    /// Represents a player supplied by Roblox at a remote boundary.
    Player,
}

impl RobloxInstance {
    pub(crate) fn from_type_name(type_name: &str) -> Option<Self> {
        match type_name {
            "Part" => Some(Self::Part),
            "Folder" => Some(Self::Folder),
            "Model" => Some(Self::Model),
            "RemoteEvent" => Some(Self::RemoteEvent),
            "RemoteFunction" => Some(Self::RemoteFunction),
            "Player" => Some(Self::Player),
            _ => None,
        }
    }

    pub(crate) const fn canonical_name(self) -> &'static str {
        match self {
            Self::Part => "Part",
            Self::Folder => "Folder",
            Self::Model => "Model",
            Self::RemoteEvent => "RemoteEvent",
            Self::RemoteFunction => "RemoteFunction",
            Self::Player => "Player",
        }
    }

    /// Distinguishes source-created Instances from engine-supplied boundary values.
    pub(crate) const fn can_construct(self) -> bool {
        !matches!(self, Self::Player)
    }

    pub(crate) fn property_type(self, property_name: &str) -> Option<CheckedValueType> {
        match property_name {
            "Name" => Some(CheckedValueType::String),
            "Anchored" | "CanCollide" if matches!(self, Self::Part) => {
                Some(CheckedValueType::Boolean)
            }
            "Transparency" if matches!(self, Self::Part) => Some(CheckedValueType::Number),
            _ => None,
        }
    }
}
