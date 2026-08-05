use crate::{source_language::ParsedExpression, SourceRange};

/// Names one validated-syntax traversal after an identifier assignment root.
pub enum ParsedPlaceStep {
    /// Traverses a named record field.
    Field {
        field_name: String,
        field_range: SourceRange,
        base_range: SourceRange,
    },
    /// Traverses a source zero-based array index.
    Index {
        index_expression: ParsedExpression,
        base_range: SourceRange,
    },
}

/// Represents an assignable identifier-rooted, non-empty field/index path.
pub struct ParsedPlaceAssignment {
    root_binding_name: String,
    root_binding_range: SourceRange,
    steps: Vec<ParsedPlaceStep>,
    assigned_value: ParsedExpression,
}

/// Preserves all source path detail for semantic checking without assignment variants per container.
impl ParsedPlaceAssignment {
    /// Builds a target after parser flattening proves its identifier root and non-empty traversal.
    pub(crate) fn from_parts(
        parts: (String, SourceRange, Vec<ParsedPlaceStep>, ParsedExpression),
    ) -> Self {
        let (root_binding_name, root_binding_range, steps, assigned_value) = parts;
        Self {
            root_binding_name,
            root_binding_range,
            steps,
            assigned_value,
        }
    }
    /// Provides the binding that controls mutability.
    pub(crate) fn root_binding_name(&self) -> &str {
        &self.root_binding_name
    }
    /// Provides the binding source range.
    pub(crate) const fn root_binding_range(&self) -> SourceRange {
        self.root_binding_range
    }
    /// Provides ordered traversal steps.
    pub(crate) fn steps(&self) -> &[ParsedPlaceStep] {
        &self.steps
    }
    /// Provides the assignment right-hand side.
    pub(crate) const fn assigned_value(&self) -> &ParsedExpression {
        &self.assigned_value
    }
    /// Provides the first source location for unreachable statement diagnostics.
    pub(crate) const fn source_range(&self) -> SourceRange {
        self.root_binding_range
    }
}
