use crate::generated_luau::LuauExpression;

/// Names one target traversal in lowered mutable assignment code.
#[derive(Debug, PartialEq, Eq)]
pub enum LuauPlaceStep {
    Field(String),
    Index(LuauExpression),
}
/// Owns a lowered mutable field/index assignment path.
#[derive(Debug, PartialEq, Eq)]
pub struct LuauPlaceAssignment {
    root_binding_name: String,
    steps: Vec<LuauPlaceStep>,
    assigned_value: LuauExpression,
}
/// Keeps target and RHS together for deterministic text generation.
impl LuauPlaceAssignment {
    /// Builds a lowered assignment after semantic checking validates its path.
    pub(crate) fn from_parts(parts: (String, Vec<LuauPlaceStep>, LuauExpression)) -> Self {
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
    /// Provides ordered target traversal steps.
    pub(crate) fn steps(&self) -> &[LuauPlaceStep] {
        &self.steps
    }
    /// Provides the assignment RHS.
    pub(crate) const fn assigned_value(&self) -> &LuauExpression {
        &self.assigned_value
    }
}
