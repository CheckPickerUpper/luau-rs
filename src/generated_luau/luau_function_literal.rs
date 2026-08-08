use crate::generated_luau::{LuauFunctionBody, LuauParameter, LuauValueType};

/// Owns one anonymous Luau function expression and its typed lexical body.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauFunctionLiteral {
    function_parameters: Vec<LuauParameter>,
    returned_value_type: LuauValueType,
    function_body: LuauFunctionBody,
}

/// Keeps closure expression construction inside the lowering model.
impl LuauFunctionLiteral {
    /// Builds one anonymous function from checked declaration parts.
    pub(crate) fn from_parts(
        literal_parts: (Vec<LuauParameter>, LuauValueType, LuauFunctionBody),
    ) -> Self {
        let (function_parameters, returned_value_type, function_body) = literal_parts;
        Self {
            function_parameters,
            returned_value_type,
            function_body,
        }
    }

    /// Gives the writer the ordered generated closure parameters.
    pub(crate) fn function_parameters(&self) -> &[LuauParameter] {
        &self.function_parameters
    }

    /// Gives the writer the generated closure return type.
    pub(crate) fn returned_value_type(&self) -> LuauValueType {
        self.returned_value_type.clone()
    }

    /// Gives the writer the complete generated closure body.
    pub(crate) const fn function_body(&self) -> &LuauFunctionBody {
        &self.function_body
    }
}
