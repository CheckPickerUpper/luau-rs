use crate::generated_luau::{LuauBooleanLiteral, LuauFunctionCall, LuauNumericOperation};

/// Represents expressions after source-language meaning has been resolved.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LuauExpression {
    /// Reads a previously resolved local or parameter.
    NameReference(String),
    /// Preserves the checked literal spelling for emission.
    NumberLiteral(String),
    /// Preserves a lexer-validated quoted string for lossless emission.
    StringLiteral(String),
    /// Preserves a checked boolean literal for direct emission.
    BooleanLiteral(LuauBooleanLiteral),
    /// Retains a numeric operation in source order.
    NumericOperation(LuauNumericOperation),
    /// Invokes a resolved function with ordered arguments.
    FunctionCall(LuauFunctionCall),
}
