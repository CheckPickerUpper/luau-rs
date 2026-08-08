use crate::generated_luau::{
    LuauExpression, LuauFunctionCall, LuauIfElse, LuauRobloxRemoteOperation, LuauValueType,
    LuauWhileLoop,
};

/// Represents statements using only constructs supported by the Luau writer.
#[derive(Debug, PartialEq, Eq)]
pub enum LuauStatement {
    /// Introduces a typed immutable local binding.
    ImmutableLocal {
        /// Keeps the resolved local spelling.
        local_name: String,
        /// Carries the type already verified by semantic checking.
        value_type: LuauValueType,
        /// Supplies the expression evaluated for the binding.
        initial_value: LuauExpression,
    },
    /// Introduces a typed local that generated assignments may update.
    MutableLocal {
        /// Keeps the resolved local spelling.
        local_name: String,
        /// Carries the type already verified by semantic checking.
        value_type: LuauValueType,
        /// Supplies the expression evaluated for the binding.
        initial_value: LuauExpression,
    },
    /// Replaces a generated mutable local value.
    AssignLocal {
        /// Keeps the resolved local spelling.
        local_name: String,
        /// Supplies the checked replacement expression.
        assigned_value: LuauExpression,
    },
    /// Replaces a value reached from a generated mutable field/index path.
    AssignPlace(crate::generated_luau::LuauPlaceAssignment),
    /// Invokes a function only for its effects.
    CallFunctionAndIgnoreResult(LuauFunctionCall),
    /// Emits a remote operation only for its effects.
    RobloxRemoteOperation(LuauRobloxRemoteOperation),
    /// Returns one generated expression from the enclosing Luau function.
    ReturnsValue(LuauExpression),
    /// Exits the innermost generated loop.
    BreaksLoop,
    /// Starts the next iteration of the innermost generated loop.
    ContinuesLoop,
    /// Chooses between two generated lexical scopes.
    IfElse(LuauIfElse),
    /// Repeats a generated lexical scope while its condition remains true.
    WhileLoop(LuauWhileLoop),
}
