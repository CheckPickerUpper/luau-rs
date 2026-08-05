use crate::generated_luau::LuauExpression;

/// Owns the ordered elements of a Luau table used as a source array.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauArrayLiteral {
    element_expressions: Vec<LuauExpression>,
}
/// Keeps target array elements ready for textual emission.
impl LuauArrayLiteral {
    /// Builds a target array from checked source-order expressions.
    pub(crate) const fn from_elements(element_expressions: Vec<LuauExpression>) -> Self {
        Self {
            element_expressions,
        }
    }
    /// Provides emitted elements in source order.
    pub(crate) fn element_expressions(&self) -> &[LuauExpression] {
        &self.element_expressions
    }
}
