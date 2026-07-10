use crate::generated_luau::{LuauFunctionReturn, LuauParameter, LuauStatement, LuauValueType};

/// Owns one complete function after checked source constructs have been lowered.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LuauFunction {
    function_name: String,
    function_parameters: Vec<LuauParameter>,
    returned_value_type: LuauValueType,
    function_statements: Vec<LuauStatement>,
    function_return: LuauFunctionReturn,
}

/// Restricts generated-function assembly to the lowering phase.
impl LuauFunction {
    /// Collects the ordered function parts needed for deterministic output.
    pub(crate) fn from_function_parts(
        function_parts: (
            String,
            Vec<LuauParameter>,
            LuauValueType,
            Vec<LuauStatement>,
            LuauFunctionReturn,
        ),
    ) -> Self {
        let (
            function_name,
            function_parameters,
            returned_value_type,
            function_statements,
            function_return,
        ) = function_parts;
        Self {
            function_name,
            function_parameters,
            returned_value_type,
            function_statements,
            function_return,
        }
    }

    /// Exposes the generated function name without transferring ownership.
    pub(crate) fn function_name(&self) -> &str {
        &self.function_name
    }

    /// Preserves parameter order for textual emission.
    pub(crate) fn function_parameters(&self) -> &[LuauParameter] {
        &self.function_parameters
    }

    /// Supplies the explicit Luau return annotation.
    pub(crate) fn returned_value_type(&self) -> LuauValueType {
        self.returned_value_type
    }

    /// Preserves checked statement order for textual emission.
    pub(crate) fn function_statements(&self) -> &[LuauStatement] {
        &self.function_statements
    }

    /// Gives the text writer the structurally final function return.
    pub(crate) fn function_return(&self) -> &LuauFunctionReturn {
        &self.function_return
    }
}
