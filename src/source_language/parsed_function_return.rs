use crate::source_language::ParsedExpression;

/// Describes how a parsed function body ends after all non-return statements.
pub(crate) enum ParsedFunctionReturn {
    /// Ends a function without returning a value.
    NoReturn,
    /// Ends a function by returning one parsed expression.
    ReturnsValue(ParsedExpression),
}
