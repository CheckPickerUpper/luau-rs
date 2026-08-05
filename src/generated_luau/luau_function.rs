use crate::generated_luau::{LuauFunctionBody, LuauParameter, LuauValueType};

/// Owns one complete function after checked source constructs have been lowered.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauFunction {
    function_name: String,
    function_parameters: Vec<LuauParameter>,
    returned_value_type: LuauValueType,
    function_body: LuauFunctionBody,
}

/// Restricts generated-function assembly to the lowering phase.
impl LuauFunction {
    /// Collects the ordered function parts needed for deterministic output.
    pub(crate) fn from_function_parts(
        function_parts: (String, Vec<LuauParameter>, LuauValueType, LuauFunctionBody),
    ) -> Self {
        let (function_name, function_parameters, returned_value_type, function_body) =
            function_parts;
        Self {
            function_name,
            function_parameters,
            returned_value_type,
            function_body,
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
    pub(crate) const fn returned_value_type(&self) -> LuauValueType {
        self.returned_value_type
    }

    /// Gives the text writer the complete generated function scope.
    pub(crate) const fn function_body(&self) -> &LuauFunctionBody {
        &self.function_body
    }
}
