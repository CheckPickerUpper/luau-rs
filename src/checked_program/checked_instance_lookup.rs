use crate::checked_program::{roblox_instance::RobloxInstance, CheckedExpression};

/// Retains an explicitly yielding hierarchy lookup after parent and child-name checks.
pub struct CheckedInstanceLookup {
    instance: RobloxInstance,
    parent_expression: Box<CheckedExpression>,
    child_name_expression: Box<CheckedExpression>,
}

impl CheckedInstanceLookup {
    /// Builds a lookup whose result class and argument types are already validated.
    pub(crate) fn from_parts(
        lookup_parts: (
            RobloxInstance,
            Box<CheckedExpression>,
            Box<CheckedExpression>,
        ),
    ) -> Self {
        let (instance, parent_expression, child_name_expression) = lookup_parts;
        Self {
            instance,
            parent_expression,
            child_name_expression,
        }
    }

    pub(crate) const fn instance(&self) -> RobloxInstance {
        self.instance
    }

    pub(crate) const fn parent_expression(&self) -> &CheckedExpression {
        &self.parent_expression
    }

    pub(crate) const fn child_name_expression(&self) -> &CheckedExpression {
        &self.child_name_expression
    }
}
