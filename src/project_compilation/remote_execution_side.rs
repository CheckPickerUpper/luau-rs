/// Identifies the only concrete execution sides that can own a direction-specific remote.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteExecutionSide {
    /// Runs in the authoritative server runtime.
    Server,
    /// Runs in a player's client runtime.
    Client,
}
