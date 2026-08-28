//! Runtime side derived from a Roblox service.

/// Distinguishes which Roblox runtime can execute a discovered project module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum ModuleExecutionSide {
    /// Runs only in the authoritative server runtime.
    Server,
    /// Runs only in each player's client runtime.
    Client,
    /// Is available to both runtimes and initialized independently by each one.
    Shared,
}
