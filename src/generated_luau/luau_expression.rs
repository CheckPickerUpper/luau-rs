use crate::generated_luau::{
    LuauArrayLiteral, LuauArrayRead, LuauBooleanLiteral, LuauComparisonOperation,
    LuauEqualityOperation, LuauFieldRead, LuauFunctionCall, LuauFunctionLiteral,
    LuauInstanceConstruction, LuauInstanceLookup, LuauLogicalNegation, LuauLogicalOperation,
    LuauNumericOperation, LuauRecordLiteral,
};

/// Represents expressions after source-language meaning has been resolved.
#[derive(Debug, PartialEq, Eq)]
pub enum LuauExpression {
    /// Reads a previously resolved local or parameter.
    NameReference(String),
    /// Refers to a visible generated function declaration.
    FunctionReference(String),
    /// Preserves the checked literal spelling for emission.
    NumberLiteral(String),
    /// Preserves a lexer-validated quoted string for lossless emission.
    StringLiteral(String),
    /// Preserves a checked boolean literal for direct emission.
    BooleanLiteral(LuauBooleanLiteral),
    /// Acquires a fixed Roblox service through the sole generated runtime bridge.
    RobloxServiceAcquisition(String),
    /// Constructs one approved Roblox Instance class.
    RobloxInstanceAcquisition(LuauInstanceConstruction),
    /// Performs an explicit `WaitForChild` hierarchy lookup.
    RobloxInstanceWaitForChild(LuauInstanceLookup),
    /// Constructs a target table whose positions match source array order.
    ArrayLiteral(LuauArrayLiteral),
    /// Constructs a checked record value as a Luau table literal.
    RecordLiteral(LuauRecordLiteral),
    /// Reads a validated table field with Luau dot access.
    FieldRead(LuauFieldRead),
    /// Reads a source zero-based array through Luau's one-based table indexing.
    ArrayRead(LuauArrayRead),
    /// Retains a numeric operation in source order.
    NumericOperation(LuauNumericOperation),
    /// Retains a numeric comparison in source order.
    ComparisonOperation(LuauComparisonOperation),
    /// Retains an equality operation in source order.
    EqualityOperation(LuauEqualityOperation),
    /// Retains a boolean negation in source order.
    LogicalNegation(LuauLogicalNegation),
    /// Retains a short-circuit logical operation in source order.
    LogicalOperation(LuauLogicalOperation),
    /// Invokes a resolved function with ordered arguments.
    FunctionCall(LuauFunctionCall),
    /// Emits an anonymous function expression that closes over active Luau locals.
    FunctionLiteral(LuauFunctionLiteral),
}
