use crate::checked_program::{
    CheckedFunctionReturn, CheckedParameter, CheckedStatement, CheckedValueType,
};

/// Owns one validated function signature and body.
pub(crate) struct CheckedFunction {
    function_name: String,
    function_parameters: Vec<CheckedParameter>,
    returned_value_type: CheckedValueType,
    function_statements: Vec<CheckedStatement>,
    function_return: CheckedFunctionReturn,
}

/// Exposes only the validated structure needed by Luau generation.
impl CheckedFunction {
    /// Builds a function after every statement validates against its signature.
    pub(crate) fn from_checked_declaration(
        checked_declaration: (
            String,
            Vec<CheckedParameter>,
            CheckedValueType,
            Vec<CheckedStatement>,
            CheckedFunctionReturn,
        ),
    ) -> Self {
        let (
            function_name,
            function_parameters,
            returned_value_type,
            function_statements,
            function_return,
        ) = checked_declaration;
        Self {
            function_name,
            function_parameters,
            returned_value_type,
            function_statements,
            function_return,
        }
    }

    /// Gives Luau generation the validated declaration name.
    pub(crate) fn function_name(&self) -> &str {
        &self.function_name
    }

    /// Gives Luau generation the ordered validated parameter set.
    pub(crate) fn function_parameters(&self) -> &[CheckedParameter] {
        &self.function_parameters
    }

    /// Gives Luau generation the validated returned value type.
    pub(crate) fn returned_value_type(&self) -> CheckedValueType {
        self.returned_value_type
    }

    /// Gives Luau generation the ordered validated function body.
    pub(crate) fn statements(&self) -> &[CheckedStatement] {
        &self.function_statements
    }

    /// Gives Luau generation the structurally final checked function return.
    pub(crate) fn function_return(&self) -> &CheckedFunctionReturn {
        &self.function_return
    }
}
