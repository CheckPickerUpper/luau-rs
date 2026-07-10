use crate::generated_luau::{LuauExpression, LuauFunctionCall, LuauValueType};

/// Represents statements using only constructs supported by the Luau writer.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LuauStatement {
    /// Introduces a typed immutable local binding.
    ImmutableLocal {
        /// Keeps the resolved local spelling.
        local_name: String,
        /// Carries the type already verified by semantic checking.
        value_type: LuauValueType,
        /// Supplies the expression evaluated for the binding.
        initial_value: LuauExpression,
    },
    /// Invokes a function only for its effects.
    CallFunctionAndIgnoreResult(LuauFunctionCall),
}
