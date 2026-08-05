use crate::generated_luau::{LuauExpression, LuauFunction, LuauProgramEnding, LuauRecordAlias};

/// Owns the complete target-language program independently of checked source nodes.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauProgram {
    program_functions: Vec<LuauFunction>,
    record_aliases: Vec<LuauRecordAlias>,
    program_ending: LuauProgramEnding,
}

/// Keeps the target program immutable between lowering and text emission.
impl LuauProgram {
    /// Joins lowered declarations with the expression that starts program execution.
    pub(crate) fn from_program_parts(
        program_parts: (Vec<LuauRecordAlias>, Vec<LuauFunction>, LuauExpression),
    ) -> Self {
        let (record_aliases, program_functions, entry_function_call) = program_parts;
        Self {
            program_functions,
            record_aliases,
            program_ending: LuauProgramEnding::EntrypointCall(entry_function_call),
        }
    }

    /// Represents a module whose declarations must not run merely because the module loaded.
    pub(crate) fn from_library_declarations(
        library_declarations: (Vec<LuauRecordAlias>, Vec<LuauFunction>),
    ) -> Self {
        let (record_aliases, program_functions) = library_declarations;
        Self {
            program_functions,
            record_aliases,
            program_ending: LuauProgramEnding::NoEntrypointCall,
        }
    }

    /// Preserves declaration order for deterministic output.
    pub(crate) fn program_functions(&self) -> &[LuauFunction] {
        &self.program_functions
    }

    /// Supplies the target-level ending selected for this source unit.
    pub(crate) const fn program_ending(&self) -> &LuauProgramEnding {
        &self.program_ending
    }

    /// Supplies strict table aliases before target function declarations.
    pub(crate) fn record_aliases(&self) -> &[LuauRecordAlias] {
        &self.record_aliases
    }
}
