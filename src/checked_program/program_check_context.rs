use crate::{
    checked_program::CheckedValueType,
    source_language::{ParsedFunction, ParsedProgram, ParsedValueType},
};

/// Owns the source-ordered declarations and active local scope during semantic checking.
pub(super) struct ProgramCheckContext<'a> {
    parsed_program: &'a ParsedProgram,
    local_bindings: Vec<(String, CheckedValueType)>,
    visible_function_signatures: Vec<(String, Vec<CheckedValueType>, CheckedValueType)>,
    expected_returned_value_type: CheckedValueType,
}

/// Restricts mutable semantic state to the checked-program phase.
impl<'a> ProgramCheckContext<'a> {
    /// Starts checking with no visible declarations or active local bindings.
    pub(super) fn from_parsed_program(parsed_program: &'a ParsedProgram) -> Self {
        Self {
            parsed_program,
            local_bindings: Vec::new(),
            visible_function_signatures: Vec::new(),
            expected_returned_value_type: CheckedValueType::NoReturnedValues,
        }
    }

    /// Provides the complete source declaration set for forward-reference classification.
    pub(super) fn parsed_program(&self) -> &ParsedProgram {
        self.parsed_program
    }

    /// Starts the next function with an empty local scope and its declared return contract.
    pub(super) fn begin_function(&mut self, parsed_function: &ParsedFunction) {
        self.local_bindings.clear();
        self.expected_returned_value_type =
            Self::to_checked_value_type(parsed_function.returned_value_type());
    }

    /// Makes the current function visible before its body is checked so recursion remains valid.
    pub(super) fn add_visible_function(&mut self, parsed_function: &ParsedFunction) {
        let parameter_types = parsed_function
            .function_parameters()
            .iter()
            .map(|parameter| Self::to_checked_value_type(parameter.value_type()))
            .collect();
        self.visible_function_signatures.push((
            parsed_function.function_name().to_owned(),
            parameter_types,
            Self::to_checked_value_type(parsed_function.returned_value_type()),
        ));
    }

    /// Provides source-ordered function signatures for declaration and call checks.
    pub(super) fn visible_function_signatures(
        &self,
    ) -> &[(String, Vec<CheckedValueType>, CheckedValueType)] {
        &self.visible_function_signatures
    }

    /// Makes a validated parameter or local visible to following expressions.
    pub(super) fn add_local_binding(&mut self, local_binding: (String, CheckedValueType)) {
        self.local_bindings.push(local_binding);
    }

    /// Records the active lexical-scope boundary before checking a nested body.
    pub(super) fn local_scope_boundary(&self) -> usize {
        self.local_bindings.len()
    }

    /// Removes bindings introduced by a completed nested lexical scope.
    pub(super) fn end_local_scope(&mut self, local_scope_boundary: usize) {
        self.local_bindings.truncate(local_scope_boundary);
    }

    /// Provides active parameter and local bindings for collision and reference checks.
    pub(super) fn local_bindings(&self) -> &[(String, CheckedValueType)] {
        &self.local_bindings
    }

    /// Provides the current function's checked return contract.
    pub(super) fn expected_returned_value_type(&self) -> CheckedValueType {
        self.expected_returned_value_type
    }

    /// Converts the parsed type vocabulary into the checked-program vocabulary.
    pub(super) fn to_checked_value_type(parsed_value_type: ParsedValueType) -> CheckedValueType {
        match parsed_value_type {
            ParsedValueType::Number => CheckedValueType::Number,
            ParsedValueType::String => CheckedValueType::String,
            ParsedValueType::Boolean => CheckedValueType::Boolean,
            ParsedValueType::NoReturnedValues => CheckedValueType::NoReturnedValues,
        }
    }
}
