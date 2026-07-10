use crate::generated_luau::{LuauExpression, LuauFunction};

/// Owns the complete target-language program independently of checked source nodes.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LuauProgram {
    program_functions: Vec<LuauFunction>,
    entry_function_call: LuauExpression,
}

/// Keeps the target program immutable between lowering and text emission.
impl LuauProgram {
    /// Joins lowered declarations with the expression that starts program execution.
    pub(crate) fn from_program_parts(program_parts: (Vec<LuauFunction>, LuauExpression)) -> Self {
        let (program_functions, entry_function_call) = program_parts;
        Self {
            program_functions,
            entry_function_call,
        }
    }

    /// Preserves declaration order for deterministic output.
    pub(crate) fn program_functions(&self) -> &[LuauFunction] {
        &self.program_functions
    }

    /// Supplies the final expression that enters the compiled program.
    pub(crate) fn entry_function_call(&self) -> &LuauExpression {
        &self.entry_function_call
    }
}
