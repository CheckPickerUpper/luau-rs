use crate::checked_program::CheckedExpression;

/// Owns checked elements of one non-empty homogeneous array.
pub struct CheckedArrayLiteral {
    element_expressions: Vec<CheckedExpression>,
}
/// Keeps array element order available to Luau lowering.
impl CheckedArrayLiteral {
    /// Builds an array after all elements satisfy the first element's type.
    pub(crate) const fn from_elements(element_expressions: Vec<CheckedExpression>) -> Self {
        Self {
            element_expressions,
        }
    }
    /// Provides elements in source order.
    pub(crate) fn element_expressions(&self) -> &[CheckedExpression] {
        &self.element_expressions
    }
}
