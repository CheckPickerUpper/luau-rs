use crate::checked_program::{
    CheckedExpression, CheckedFunctionCall, CheckedIfElse, CheckedRobloxRemoteOperation,
    CheckedValueType, CheckedWhileLoop,
};

/// Represents only validated forms allowed inside a function body.
pub enum CheckedStatement {
    /// Introduces an immutable local whose initial value has the declared type.
    ImmutableLocal {
        local_name: String,
        value_type: CheckedValueType,
        initial_value: CheckedExpression,
    },
    /// Introduces a mutable local whose initial value has the declared type.
    MutableLocal {
        local_name: String,
        value_type: CheckedValueType,
        initial_value: CheckedExpression,
    },
    /// Replaces a resolved mutable local with a value of its declared type.
    AssignLocal {
        local_name: String,
        assigned_value: CheckedExpression,
    },
    /// Replaces a value reached through a checked mutable field/index path.
    AssignPlace(crate::checked_program::CheckedPlaceAssignment),
    /// Invokes a validated function where the source discards any returned value.
    CallFunctionAndIgnoreResult(CheckedFunctionCall),
    /// Performs a validated remote operation where the source discards any returned value.
    RobloxRemoteOperation(CheckedRobloxRemoteOperation),
    /// Returns an expression whose type matches the enclosing function contract.
    ReturnsValue(CheckedExpression),
    /// Exits the innermost enclosing loop.
    BreaksLoop,
    /// Starts the next iteration of the innermost enclosing loop.
    ContinuesLoop,
    /// Selects between independently scoped checked branches.
    IfElse(CheckedIfElse),
    /// Repeats an independently scoped checked body while its condition remains true.
    WhileLoop(CheckedWhileLoop),
}
