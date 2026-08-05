use crate::generated_luau::LuauExpression;

/// Owns a target indexed access whose source index is zero based.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauArrayRead {
    base_expression: Box<LuauExpression>,
    index_expression: Box<LuauExpression>,
}
/// Retains the two values needed to add Luau's one-based offset at emission.
impl LuauArrayRead {
    /// Builds the target read after checked zero-based semantics are established.
    pub(crate) fn from_read(read_parts: (Box<LuauExpression>, Box<LuauExpression>)) -> Self {
        let (base_expression, index_expression) = read_parts;
        Self {
            base_expression,
            index_expression,
        }
    }
    /// Provides the table expression.
    pub(crate) const fn base_expression(&self) -> &LuauExpression {
        &self.base_expression
    }
    /// Provides the zero-based source index expression.
    pub(crate) const fn index_expression(&self) -> &LuauExpression {
        &self.index_expression
    }
}
