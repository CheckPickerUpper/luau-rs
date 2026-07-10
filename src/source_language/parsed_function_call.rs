use crate::{source_language::ParsedExpression, SourceRange};

/// Owns a parsed function invocation shared by value and effect-only grammar positions.
pub(crate) struct ParsedFunctionCall {
    function_name: String,
    function_name_range: SourceRange,
    function_arguments: Vec<ParsedExpression>,
    call_range: SourceRange,
}

/// Keeps call-only statements structurally separate from arbitrary expressions.
impl ParsedFunctionCall {
    /// Preserves the complete parsed call and both its target and expression locations.
    pub(crate) fn from_call(
        parsed_call: (String, SourceRange, Vec<ParsedExpression>, SourceRange),
    ) -> Self {
        let (function_name, function_name_range, function_arguments, call_range) = parsed_call;
        Self {
            function_name,
            function_name_range,
            function_arguments,
            call_range,
        }
    }

    /// Gives semantic checking the invoked source name.
    pub(crate) fn function_name(&self) -> &str {
        &self.function_name
    }

    /// Gives semantic checking the range used for call diagnostics.
    pub(crate) fn function_name_range(&self) -> SourceRange {
        self.function_name_range
    }

    /// Gives semantic checking the ordered source arguments.
    pub(crate) fn function_arguments(&self) -> &[ParsedExpression] {
        &self.function_arguments
    }

    /// Gives type checking the complete call-expression range.
    pub(crate) fn source_range(&self) -> SourceRange {
        self.call_range
    }
}
