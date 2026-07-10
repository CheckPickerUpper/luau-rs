use crate::checked_program::CheckedExpression;

/// Describes the validated final clause of a checked function body.
pub(crate) enum CheckedFunctionReturn {
    /// Ends a function proven not to return values.
    NoReturn,
    /// Ends a function with a type-compatible returned expression.
    ReturnsValue(CheckedExpression),
}
