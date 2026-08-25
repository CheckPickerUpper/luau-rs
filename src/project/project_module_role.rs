//! Initialization role derived from a Roblox service.

/// Distinguishes eager Roblox scripts from lazily initialized `ModuleScripts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum ProjectModuleRole {
    /// Instantiates and invokes the module's `main` export eagerly.
    Entrypoint,
    /// Emits declarations only; a later import surface owns initialization.
    Library,
}
