use crate::checked_program::CheckedExpression;

/// Owns one validated postfix record field read.
pub struct CheckedFieldRead {
    base_expression: Box<CheckedExpression>,
    field_name: String,
}

/// Retains the checked base and declared field spelling for Luau dot-access emission.
impl CheckedFieldRead {
    /// Builds a field read only after the base and field resolve to a declared record field.
    pub(crate) fn from_read(read: (Box<CheckedExpression>, String)) -> Self {
        let (base_expression, field_name) = read;
        Self {
            base_expression,
            field_name,
        }
    }

    /// Gives lowering the checked read base.
    pub(crate) fn base_expression(&self) -> &CheckedExpression {
        &self.base_expression
    }

    /// Gives lowering the validated field spelling.
    pub(crate) fn field_name(&self) -> &str {
        &self.field_name
    }
}
