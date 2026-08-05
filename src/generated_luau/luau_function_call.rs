use crate::generated_luau::LuauExpression;

/// Owns a generated Luau function invocation with ordered arguments.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauFunctionCall {
    function_name: String,
    function_arguments: Vec<LuauExpression>,
}

/// Keeps callable target syntax available to expressions and call-only statements.
impl LuauFunctionCall {
    /// Builds a generated call from its target name and generated arguments.
    pub(crate) fn from_call(generated_call: (String, Vec<LuauExpression>)) -> Self {
        let (function_name, function_arguments) = generated_call;
        Self {
            function_name,
            function_arguments,
        }
    }

    /// Gives the text writer the generated target name.
    pub(crate) fn function_name(&self) -> &str {
        &self.function_name
    }

    /// Gives the text writer the ordered generated arguments.
    pub(crate) fn function_arguments(&self) -> &[LuauExpression] {
        &self.function_arguments
    }
}
