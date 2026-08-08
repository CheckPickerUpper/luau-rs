use crate::{
    source_language::{ParsedFunctionBody, ParsedParameter, ParsedValueType},
    SourceRange,
};

/// Owns one anonymous function expression with its typed signature and lexical body.
pub struct ParsedFunctionLiteral {
    function_parameters: Vec<ParsedParameter>,
    returned_value_type: ParsedValueType,
    function_body: ParsedFunctionBody,
    expression_range: SourceRange,
}

/// Keeps closure syntax and its source extent together until semantic checking establishes captures.
impl ParsedFunctionLiteral {
    /// Builds a function literal after its signature and body have parsed.
    pub(crate) fn from_parts(
        literal_parts: (
            Vec<ParsedParameter>,
            ParsedValueType,
            ParsedFunctionBody,
            SourceRange,
        ),
    ) -> Self {
        let (function_parameters, returned_value_type, function_body, expression_range) =
            literal_parts;
        Self {
            function_parameters,
            returned_value_type,
            function_body,
            expression_range,
        }
    }

    /// Gives semantic checking the ordered closure parameter set.
    pub(crate) fn function_parameters(&self) -> &[ParsedParameter] {
        &self.function_parameters
    }

    /// Gives semantic checking the closure's declared returned value type.
    pub(crate) fn returned_value_type(&self) -> ParsedValueType {
        self.returned_value_type.clone()
    }

    /// Gives semantic checking the complete lexical closure body.
    pub(crate) const fn function_body(&self) -> &ParsedFunctionBody {
        &self.function_body
    }

    /// Gives diagnostics the complete anonymous function expression range.
    pub(crate) const fn expression_range(&self) -> SourceRange {
        self.expression_range
    }
}
