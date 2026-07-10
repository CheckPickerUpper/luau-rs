use crate::{source_language::ParsedValueType, SourceRange};

/// Owns one parsed function parameter declaration.
pub(crate) struct ParsedParameter {
    parameter_name: String,
    parameter_name_range: SourceRange,
    value_type: ParsedValueType,
}

/// Keeps parameter declarations out of expression and statement variants.
impl ParsedParameter {
    /// Keeps the parsed declaration name, location, and required type together for checking.
    pub(crate) fn from_declaration(declaration: (String, SourceRange, ParsedValueType)) -> Self {
        let (parameter_name, parameter_name_range, value_type) = declaration;
        Self {
            parameter_name,
            parameter_name_range,
            value_type,
        }
    }

    /// Gives semantic checking the parameter's local binding name.
    pub(crate) fn parameter_name(&self) -> &str {
        &self.parameter_name
    }

    /// Gives semantic checking the declaration-name range for name validation failures.
    pub(crate) fn parameter_name_range(&self) -> SourceRange {
        self.parameter_name_range
    }

    /// Gives semantic checking the parameter's declared value type.
    pub(crate) fn value_type(&self) -> ParsedValueType {
        self.value_type
    }
}
