use crate::checked_program::{
    CheckedExpression, CheckedFunctionCall, CheckedIfElse, CheckedValueType,
};

/// Represents only validated forms allowed inside a function body.
pub enum CheckedStatement {
    /// Introduces an immutable local whose initial value has the declared type.
    ImmutableLocal {
        local_name: String,
        value_type: CheckedValueType,
        initial_value: CheckedExpression,
    },
    /// Invokes a validated function where the source discards any returned value.
    CallFunctionAndIgnoreResult(CheckedFunctionCall),
    /// Returns an expression whose type matches the enclosing function contract.
    ReturnsValue(CheckedExpression),
    /// Selects between independently scoped checked branches.
    IfElse(CheckedIfElse),
}
