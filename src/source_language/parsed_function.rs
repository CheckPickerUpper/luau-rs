use crate::{
    source_language::{ParsedFunctionReturn, ParsedParameter, ParsedStatement, ParsedValueType},
    SourceRange,
};

/// Owns one parsed function declaration with its typed signature and body.
pub(crate) struct ParsedFunction {
    function_name: String,
    function_name_range: SourceRange,
    function_parameters: Vec<ParsedParameter>,
    returned_value_type: ParsedValueType,
    function_statements: Vec<ParsedStatement>,
    function_return: ParsedFunctionReturn,
}

/// Prevents expression or statement nodes from appearing at the program level.
impl ParsedFunction {
    /// Builds a function only after its complete signature and body have parsed.
    pub(crate) fn from_declaration(
        declaration: (
            String,
            SourceRange,
            Vec<ParsedParameter>,
            ParsedValueType,
            Vec<ParsedStatement>,
            ParsedFunctionReturn,
        ),
    ) -> Self {
        let (
            function_name,
            function_name_range,
            function_parameters,
            returned_value_type,
            function_statements,
            function_return,
        ) = declaration;
        Self {
            function_name,
            function_name_range,
            function_parameters,
            returned_value_type,
            function_statements,
            function_return,
        }
    }

    /// Gives semantic checking the declared function identity.
    pub(crate) fn function_name(&self) -> &str {
        &self.function_name
    }

    /// Gives semantic checking the declaration-name range for function contract failures.
    pub(crate) fn function_name_range(&self) -> SourceRange {
        self.function_name_range
    }

    /// Gives semantic checking the ordered declared parameter set.
    pub(crate) fn function_parameters(&self) -> &[ParsedParameter] {
        &self.function_parameters
    }

    /// Gives semantic checking the function's declared returned value type.
    pub(crate) fn returned_value_type(&self) -> ParsedValueType {
        self.returned_value_type
    }

    /// Gives semantic checking the complete ordered function body.
    pub(crate) fn function_statements(&self) -> &[ParsedStatement] {
        &self.function_statements
    }

    /// Gives semantic checking the structurally final function return.
    pub(crate) fn function_return(&self) -> &ParsedFunctionReturn {
        &self.function_return
    }
}
