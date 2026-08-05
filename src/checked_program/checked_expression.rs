use crate::checked_program::{
    CheckedArrayLiteral, CheckedArrayRead, CheckedBooleanLiteral, CheckedComparisonOperation,
    CheckedEqualityOperation, CheckedFieldRead, CheckedLogicalNegation, CheckedLogicalOperation,
    CheckedNumericOperation, CheckedRecordLiteral,
};

/// Represents expressions whose names and types have been validated.
pub enum CheckedExpression {
    /// Refers to a resolved local value.
    NameReference(String),
    /// Preserves a validated numeric literal spelling.
    NumberLiteral(String),
    /// Preserves a validated quoted string spelling.
    StringLiteral(String),
    /// Preserves a checked boolean literal without string encoding.
    BooleanLiteral(CheckedBooleanLiteral),
    /// Preserves one catalogued service acquisition for writer-owned `GetService` lowering.
    RobloxServiceAcquisition(super::roblox_service::RobloxService),
    /// Constructs a checked homogeneous array.
    ArrayLiteral(CheckedArrayLiteral),
    /// Constructs a record whose declared fields and initializer types were checked.
    RecordLiteral(CheckedRecordLiteral),
    /// Reads a declared field from a checked named record.
    FieldRead(CheckedFieldRead),
    /// Reads a checked array element.
    ArrayRead(CheckedArrayRead),
    /// Retains a numeric operation whose operands are proven numeric.
    NumericOperation(CheckedNumericOperation),
    /// Retains a numeric comparison whose operands are proven numeric.
    ComparisonOperation(CheckedComparisonOperation),
    /// Retains an equality operation whose operands have the same value type.
    EqualityOperation(CheckedEqualityOperation),
    /// Retains a boolean negation whose operand is proven boolean.
    LogicalNegation(CheckedLogicalNegation),
    /// Retains a short-circuit logical operation whose operands are proven boolean.
    LogicalOperation(CheckedLogicalOperation),
    /// Calls a resolved function with validated argument types.
    FunctionCall(CheckedFunctionCall),
}
use crate::checked_program::CheckedFunctionCall;
