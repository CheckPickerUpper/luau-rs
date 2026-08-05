use crate::generated_luau::{LuauExpression, LuauFunctionBody};

/// Represents a target-language decision after all source semantics have been validated.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauIfElse {
    condition: LuauExpression,
    then_body: LuauFunctionBody,
    else_body: LuauFunctionBody,
}

/// Keeps target branch structure explicit so strict Luau indentation and endings are deterministic.
impl LuauIfElse {
    /// Joins the generated condition with both required generated branches.
    pub(crate) fn from_parts(
        if_else_parts: (LuauExpression, LuauFunctionBody, LuauFunctionBody),
    ) -> Self {
        let (condition, then_body, else_body) = if_else_parts;
        Self {
            condition,
            then_body,
            else_body,
        }
    }

    /// Supplies the expression that selects the branch.
    pub(crate) const fn condition(&self) -> &LuauExpression {
        &self.condition
    }

    /// Supplies statements emitted for the true branch.
    pub(crate) const fn then_body(&self) -> &LuauFunctionBody {
        &self.then_body
    }

    /// Supplies statements emitted for the false branch.
    pub(crate) const fn else_body(&self) -> &LuauFunctionBody {
        &self.else_body
    }
}
