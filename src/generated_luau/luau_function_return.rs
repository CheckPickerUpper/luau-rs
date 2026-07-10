use crate::generated_luau::LuauExpression;

/// Describes the final generated clause of a Luau function body.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LuauFunctionReturn {
    /// Ends a function without a return statement.
    NoReturn,
    /// Ends a function with a generated returned expression.
    ReturnsValue(LuauExpression),
}
