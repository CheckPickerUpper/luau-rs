use crate::generated_luau::{LuauExpression, LuauFunctionBody};

/// Represents a generated Luau loop after source types and lexical scope have been validated.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauWhileLoop {
    condition: LuauExpression,
    body: LuauFunctionBody,
}

/// Keeps target loop structure explicit so strict Luau serialization remains deterministic.
impl LuauWhileLoop {
    /// Joins the generated condition with the generated body it repeatedly evaluates.
    pub(crate) fn from_parts(while_loop_parts: (LuauExpression, LuauFunctionBody)) -> Self {
        let (condition, body) = while_loop_parts;
        Self { condition, body }
    }

    /// Supplies the expression that controls whether another loop iteration begins.
    pub(crate) const fn condition(&self) -> &LuauExpression {
        &self.condition
    }

    /// Supplies statements emitted for each loop iteration.
    pub(crate) const fn body(&self) -> &LuauFunctionBody {
        &self.body
    }
}
