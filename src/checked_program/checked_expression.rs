/// Represents expressions whose names and types have been validated.
pub(crate) enum CheckedExpression {
    /// Refers to a resolved local value.
    NameReference(String),
    /// Preserves a validated numeric literal spelling.
    NumberLiteral(String),
    /// Adds two values proven numeric.
    Addition {
        left_operand: Box<CheckedExpression>,
        right_operand: Box<CheckedExpression>,
    },
    /// Calls a resolved function with validated argument types.
    FunctionCall(CheckedFunctionCall),
}
use crate::checked_program::CheckedFunctionCall;
