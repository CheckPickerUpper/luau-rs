use crate::{
    source_language::{ParsedFunctionBody, ParsedParameter, ParsedValueType},
    SourceRange,
};

/// Owns one parsed function declaration with its typed signature and body.
pub struct ParsedFunction {
    function_name: String,
    function_name_range: SourceRange,
    function_parameters: Vec<ParsedParameter>,
    returned_value_type: ParsedValueType,
    function_body: ParsedFunctionBody,
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
            ParsedFunctionBody,
        ),
    ) -> Self {
        let (
            function_name,
            function_name_range,
            function_parameters,
            returned_value_type,
            function_body,
        ) = declaration;
        Self {
            function_name,
            function_name_range,
            function_parameters,
            returned_value_type,
            function_body,
        }
    }

    /// Gives semantic checking the declared function identity.
    pub(crate) fn function_name(&self) -> &str {
        &self.function_name
    }

    /// Gives semantic checking the declaration-name range for function contract failures.
    pub(crate) const fn function_name_range(&self) -> SourceRange {
        self.function_name_range
    }

    /// Gives semantic checking the ordered declared parameter set.
    pub(crate) fn function_parameters(&self) -> &[ParsedParameter] {
        &self.function_parameters
    }

    /// Gives semantic checking the function's declared returned value type.
    pub(crate) const fn returned_value_type(&self) -> ParsedValueType {
        self.returned_value_type
    }

    /// Gives semantic checking the complete scope of the function body.
    pub(crate) const fn function_body(&self) -> &ParsedFunctionBody {
        &self.function_body
    }
}
