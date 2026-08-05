use crate::checked_program::CheckedExpression;

/// Owns one checked zero-based array read.
pub struct CheckedArrayRead {
    base_expression: Box<CheckedExpression>,
    index_expression: Box<CheckedExpression>,
}
/// Keeps both operands for target-specific indexed access lowering.
impl CheckedArrayRead {
    /// Builds a read after array base and numeric index validation.
    pub(crate) fn from_read(read_parts: (Box<CheckedExpression>, Box<CheckedExpression>)) -> Self {
        let (base_expression, index_expression) = read_parts;
        Self {
            base_expression,
            index_expression,
        }
    }
    /// Provides the checked array expression.
    pub(crate) const fn base_expression(&self) -> &CheckedExpression {
        &self.base_expression
    }
    /// Provides the checked zero-based index expression.
    pub(crate) const fn index_expression(&self) -> &CheckedExpression {
        &self.index_expression
    }
}
