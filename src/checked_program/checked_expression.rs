use crate::checked_program::{CheckedBooleanLiteral, CheckedNumericOperation};

/// Represents expressions whose names and types have been validated.
pub(crate) enum CheckedExpression {
    /// Refers to a resolved local value.
    NameReference(String),
    /// Preserves a validated numeric literal spelling.
    NumberLiteral(String),
    /// Preserves a validated quoted string spelling.
    StringLiteral(String),
    /// Preserves a checked boolean literal without string encoding.
    BooleanLiteral(CheckedBooleanLiteral),
    /// Retains a numeric operation whose operands are proven numeric.
    NumericOperation(CheckedNumericOperation),
    /// Calls a resolved function with validated argument types.
    FunctionCall(CheckedFunctionCall),
}
use crate::checked_program::CheckedFunctionCall;
