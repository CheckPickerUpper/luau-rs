use crate::generated_luau::LuauExpression;

/// Owns one strict Luau postfix table field read.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauFieldRead {
    base_expression: Box<LuauExpression>,
    field_name: String,
}

/// Retains the base expression and checked field spelling for dot-access emission.
impl LuauFieldRead {
    /// Builds a target field read from a checked source read.
    pub(crate) fn from_read(read: (Box<LuauExpression>, String)) -> Self {
        let (base_expression, field_name) = read;
        Self {
            base_expression,
            field_name,
        }
    }

    /// Gives the writer the lowered base expression.
    pub(crate) fn base_expression(&self) -> &LuauExpression {
        &self.base_expression
    }

    /// Gives the writer the field spelling.
    pub(crate) fn field_name(&self) -> &str {
        &self.field_name
    }
}
