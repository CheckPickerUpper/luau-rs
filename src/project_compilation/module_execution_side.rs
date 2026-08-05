/// States which Roblox runtime owns a generated module.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum ModuleExecutionSide {
    /// Runs only in the authoritative server runtime.
    Server,
    /// Runs only in each player's client runtime.
    Client,
    /// Is available to both runtimes and initialized independently by each one.
    Shared,
}
