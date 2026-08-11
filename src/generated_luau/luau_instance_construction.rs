use crate::generated_luau::LuauExpression;

/// Preserves class construction and optional hierarchy placement for deterministic emission.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauInstanceConstruction {
    instance_name: String,
    parent_expression: Option<Box<LuauExpression>>,
}

impl LuauInstanceConstruction {
    pub(crate) fn from_parts(construction_parts: (String, Option<Box<LuauExpression>>)) -> Self {
        let (instance_name, parent_expression) = construction_parts;
        Self {
            instance_name,
            parent_expression,
        }
    }

    pub(crate) fn instance_name(&self) -> &str {
        &self.instance_name
    }

    pub(crate) fn parent_expression(&self) -> Option<&LuauExpression> {
        self.parent_expression.as_deref()
    }
}
