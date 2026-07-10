use crate::source_language::{ParsedExpression, ParsedFunctionCall, ParsedValueType};

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
}
