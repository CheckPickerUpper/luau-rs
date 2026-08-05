use crate::generated_luau::LuauExpression;

/// Captures whether a generated source unit starts itself or waits for an importer.
#[derive(Debug, PartialEq, Eq)]
pub enum LuauProgramEnding {
    /// Calls the source entrypoint after declarations have been defined.
    EntrypointCall(LuauExpression),
    /// Leaves declarations inert until a future module surface imports them.
    NoEntrypointCall,
}
