/// Represents expressions after source-language meaning has been resolved.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LuauExpression {
    /// Reads a previously resolved local or parameter.
    NameReference(String),
    /// Preserves the checked literal spelling for emission.
    NumberLiteral(String),
    /// Adds two numeric expressions in source order.
    Addition {
        /// Supplies the left operand.
        left_operand: Box<LuauExpression>,
        /// Supplies the right operand.
        right_operand: Box<LuauExpression>,
    },
    /// Invokes a resolved function with ordered arguments.
    FunctionCall(LuauFunctionCall),
}
use crate::generated_luau::LuauFunctionCall;
