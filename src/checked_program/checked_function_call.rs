use crate::checked_program::CheckedExpression;

/// Owns a resolved function invocation with type-checked arguments.
pub(crate) struct CheckedFunctionCall {
    function_name: String,
    function_arguments: Vec<CheckedExpression>,
}

/// Preserves call-only statement validity beyond the parser boundary.
impl CheckedFunctionCall {
    /// Builds a call only after function and argument validation succeeds.
    pub(crate) fn from_checked_call(checked_call: (String, Vec<CheckedExpression>)) -> Self {
        let (function_name, function_arguments) = checked_call;
        Self {
            function_name,
            function_arguments,
        }
    }

    /// Gives Luau generation the resolved function name.
    pub(crate) fn function_name(&self) -> &str {
        &self.function_name
    }

    /// Gives Luau generation the validated ordered arguments.
    pub(crate) fn function_arguments(&self) -> &[CheckedExpression] {
        &self.function_arguments
    }
}
