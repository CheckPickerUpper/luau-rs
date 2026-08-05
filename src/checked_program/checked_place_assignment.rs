use crate::checked_program::CheckedExpression;

/// Names one checked path segment for a mutable assignment target.
pub enum CheckedPlaceStep {
    /// Traverses a record field known to exist.
    Field(String),
    /// Traverses a source zero-based array index known numeric.
    Index(CheckedExpression),
}

/// Represents one mutable identifier-rooted field/index target after type checking.
pub struct CheckedPlaceAssignment {
    root_binding_name: String,
    steps: Vec<CheckedPlaceStep>,
    assigned_value: CheckedExpression,
}

/// Keeps target traversal compact for Luau lowering.
impl CheckedPlaceAssignment {
    /// Builds a checked assignment after root mutability and every path type are verified.
    pub(crate) fn from_parts(parts: (String, Vec<CheckedPlaceStep>, CheckedExpression)) -> Self {
        let (root_binding_name, steps, assigned_value) = parts;
        Self {
            root_binding_name,
            steps,
            assigned_value,
        }
    }
    /// Provides the mutable root name.
    pub(crate) fn root_binding_name(&self) -> &str {
        &self.root_binding_name
    }
    /// Provides ordered checked traversal steps.
    pub(crate) fn steps(&self) -> &[CheckedPlaceStep] {
        &self.steps
    }
    /// Provides the checked right-hand side.
    pub(crate) const fn assigned_value(&self) -> &CheckedExpression {
        &self.assigned_value
    }
}
