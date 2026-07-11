use crate::checked_program::{CheckedFunctionBody, CheckedParameter, CheckedValueType};

/// Owns one validated function signature and body.
pub(crate) struct CheckedFunction {
    function_name: String,
    function_parameters: Vec<CheckedParameter>,
    returned_value_type: CheckedValueType,
    function_body: CheckedFunctionBody,
}

/// Exposes only the validated structure needed by Luau generation.
impl CheckedFunction {
    /// Builds a function after every statement validates against its signature.
    pub(crate) fn from_checked_declaration(
        checked_declaration: (
            String,
            Vec<CheckedParameter>,
            CheckedValueType,
            CheckedFunctionBody,
        ),
    ) -> Self {
        let (function_name, function_parameters, returned_value_type, function_body) =
            checked_declaration;
        Self {
            function_name,
            function_parameters,
            returned_value_type,
            function_body,
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

    /// Gives Luau generation the whole checked lexical scope of the function.
    pub(crate) fn function_body(&self) -> &CheckedFunctionBody {
        &self.function_body
    }
}
