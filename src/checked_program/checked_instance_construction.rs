use crate::checked_program::{roblox_instance::RobloxInstance, CheckedExpression};

/// Keeps an approved class and its optional hierarchy placement inseparable through lowering.
pub struct CheckedInstanceConstruction {
    instance: RobloxInstance,
    parent_expression: Option<Box<CheckedExpression>>,
}

impl CheckedInstanceConstruction {
    pub(crate) fn from_parts(
        construction_parts: (RobloxInstance, Option<Box<CheckedExpression>>),
    ) -> Self {
        let (instance, parent_expression) = construction_parts;
        Self {
            instance,
            parent_expression,
        }
    }

    pub(crate) const fn instance(&self) -> RobloxInstance {
        self.instance
    }

    pub(crate) fn parent_expression(&self) -> Option<&CheckedExpression> {
        self.parent_expression.as_deref()
    }
}
