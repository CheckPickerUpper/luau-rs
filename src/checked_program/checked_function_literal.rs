use crate::checked_program::{CheckedFunctionBody, CheckedParameter, CheckedValueType};

/// Owns one checked anonymous function and the lexical body it closes over.
pub struct CheckedFunctionLiteral {
    function_parameters: Vec<CheckedParameter>,
    returned_value_type: CheckedValueType,
    function_body: CheckedFunctionBody,
}

/// Keeps closure construction behind the function-literal checker.
impl CheckedFunctionLiteral {
    /// Builds a closure after parameters, body control flow, and return values validate.
    pub(crate) fn from_parts(
        literal_parts: (Vec<CheckedParameter>, CheckedValueType, CheckedFunctionBody),
    ) -> Self {
        let (function_parameters, returned_value_type, function_body) = literal_parts;
        Self {
            function_parameters,
            returned_value_type,
            function_body,
        }
    }

    /// Gives lowering the checked closure parameters.
    pub(crate) fn function_parameters(&self) -> &[CheckedParameter] {
        &self.function_parameters
    }

    /// Gives type checking and lowering the closure return contract.
    pub(crate) fn returned_value_type(&self) -> CheckedValueType {
        self.returned_value_type.clone()
    }

    /// Gives lowering the checked lexical closure body.
    pub(crate) const fn function_body(&self) -> &CheckedFunctionBody {
        &self.function_body
    }
}
