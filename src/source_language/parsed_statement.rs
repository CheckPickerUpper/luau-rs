use crate::{
    source_language::{ParsedExpression, ParsedFunctionCall, ParsedIfElse, ParsedValueType},
    SourceRange,
};

/// Represents only grammar forms valid inside a parsed function body.
pub(crate) enum ParsedStatement {
    /// Introduces an immutable typed local value.
    ImmutableLocal {
        local_name: String,
        local_name_range: crate::SourceRange,
        value_type: ParsedValueType,
        initial_value: ParsedExpression,
    },
    /// Invokes a function where the source discards any returned value.
    CallFunctionAndIgnoreResult(ParsedFunctionCall),
    /// Returns one expression from the enclosing function or branch.
    ReturnsValue(ParsedExpression),
    /// Chooses between two independently scoped bodies.
    IfElse(ParsedIfElse),
}

/// Keeps unreachable-statement diagnostics tied to the first unreachable source construct.
impl ParsedStatement {
    /// Gives control-flow checking the statement location that follows a guaranteed return.
    pub(crate) fn source_range(&self) -> SourceRange {
        match self {
            Self::ImmutableLocal {
                local_name_range, ..
            } => *local_name_range,
            Self::CallFunctionAndIgnoreResult(function_call) => function_call.source_range(),
            Self::ReturnsValue(returned_expression) => returned_expression.source_range(),
            Self::IfElse(parsed_if_else) => parsed_if_else.condition_range(),
        }
    }
}
