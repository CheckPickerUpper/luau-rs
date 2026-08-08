use crate::generated_luau::LuauExpression;

/// Retains a typed parent and dynamic child name for explicit `WaitForChild` lowering.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauInstanceLookup {
    instance_name: String,
    parent_expression: Box<LuauExpression>,
    child_name_expression: Box<LuauExpression>,
}

impl LuauInstanceLookup {
    /// Builds one generated hierarchy lookup from checked source expressions.
    pub(crate) fn from_parts(
        lookup_parts: (String, Box<LuauExpression>, Box<LuauExpression>),
    ) -> Self {
        let (instance_name, parent_expression, child_name_expression) = lookup_parts;
        Self {
            instance_name,
            parent_expression,
            child_name_expression,
        }
    }

    pub(crate) fn instance_name(&self) -> &str {
        &self.instance_name
    }

    pub(crate) const fn parent_expression(&self) -> &LuauExpression {
        &self.parent_expression
    }

    pub(crate) const fn child_name_expression(&self) -> &LuauExpression {
        &self.child_name_expression
    }
}
