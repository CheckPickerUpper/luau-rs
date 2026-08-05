/// States whether a source module begins a Roblox execution path or waits to be imported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum ProjectModuleRole {
    /// Requires and invokes a zero-argument `main` function after lowering declarations.
    Entrypoint,
    /// Emits declarations without an eager `main` call so a later import surface owns initialization.
    Library,
}
