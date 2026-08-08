use crate::{
    checked_program::{
        roblox_service::RobloxService, CheckedLocalBinding, CheckedRecordDeclaration,
        CheckedRecordField, CheckedValueType,
    },
    source_language::{ParsedFunction, ParsedProgram, ParsedValueType},
    CompilationProblem, CompilationProblemReason, SourceRange,
};

/// States whether semantic checking has a Roblox execution side for service acquisition.
#[derive(Clone, Copy)]
pub(super) enum ServiceAcquisitionContext {
    Standalone,
    Project(crate::ModuleExecutionSide),
}

/// Owns the source-ordered declarations and active local scope during semantic checking.
pub(super) struct ProgramCheckContext<'a> {
    parsed_program: &'a ParsedProgram,
    local_bindings: Vec<CheckedLocalBinding>,
    visible_function_signatures: Vec<(String, Vec<CheckedValueType>, CheckedValueType)>,
    checked_record_declarations: Vec<CheckedRecordDeclaration>,
    expected_returned_value_type: CheckedValueType,
    service_acquisition_context: ServiceAcquisitionContext,
}

/// Restricts mutable semantic state to the checked-program phase.
impl<'a> ProgramCheckContext<'a> {
    /// Starts project body checking with signatures resolved from imported libraries.
    pub(super) fn from_parsed_program_and_imports(
        context_parts: (
            &'a ParsedProgram,
            &[super::ImportedFunctionSignature],
            ServiceAcquisitionContext,
        ),
    ) -> Self {
        let (parsed_program, imported_signatures, service_acquisition_context) = context_parts;
        Self {
            parsed_program,
            local_bindings: Vec::new(),
            visible_function_signatures: imported_signatures
                .iter()
                .map(|signature| {
                    (
                        signature.function_name().to_owned(),
                        signature.parameter_types().to_vec(),
                        signature.returned_value_type(),
                    )
                })
                .collect(),
            checked_record_declarations: Vec::new(),
            expected_returned_value_type: CheckedValueType::NoReturnedValues,
            service_acquisition_context,
        }
    }

    /// Registers every record shape before function checking permits values to refer to aliases.
    pub(super) fn register_record_declarations(&mut self) -> Result<(), CompilationProblem> {
        for parsed_record in self.parsed_program.parsed_records() {
            if self
                .checked_record_declarations
                .iter()
                .any(|candidate| candidate.record_name() == parsed_record.record_name())
            {
                return Err(CompilationProblem::from_problem_at_range((
                    parsed_record.record_name_range(),
                    CompilationProblemReason::NameAlreadyDefined,
                )));
            }
            let mut checked_fields = Vec::new();
            for parsed_field in parsed_record.record_fields() {
                if checked_fields
                    .iter()
                    .any(|checked_field: &CheckedRecordField| {
                        checked_field.field_name() == parsed_field.field_name()
                    })
                {
                    return Err(CompilationProblem::from_problem_at_range((
                        parsed_field.field_name_range(),
                        CompilationProblemReason::DuplicateRecordField,
                    )));
                }
                Self::reject_service_type_outside_local_acquisition(parsed_field.value_type())?;
                let checked_type = self.resolve_value_type(parsed_field.value_type())?;
                checked_fields.push(CheckedRecordField::from_declaration((
                    parsed_field.field_name().to_owned(),
                    checked_type,
                )));
            }
            self.checked_record_declarations
                .push(CheckedRecordDeclaration::from_declaration((
                    parsed_record.record_name().to_owned(),
                    checked_fields,
                )));
        }
        Ok(())
    }

    /// Provides the complete source declaration set for forward-reference classification.
    pub(super) const fn parsed_program(&self) -> &ParsedProgram {
        self.parsed_program
    }

    /// Starts the next function with an empty local scope and its declared return contract.
    pub(super) fn begin_function(
        &mut self,
        parsed_function: &ParsedFunction,
    ) -> Result<(), CompilationProblem> {
        self.local_bindings.clear();
        let returned_value_type = parsed_function.returned_value_type();
        Self::reject_service_type_outside_local_acquisition(&returned_value_type)?;
        self.expected_returned_value_type = self.resolve_value_type(&returned_value_type)?;
        Ok(())
    }

    /// Makes the current function visible before its body is checked so recursion remains valid.
    pub(super) fn add_visible_function(
        &mut self,
        parsed_function: &ParsedFunction,
    ) -> Result<(), CompilationProblem> {
        let mut parameter_types = Vec::new();
        for parameter in parsed_function.function_parameters() {
            let parameter_value_type = parameter.value_type();
            Self::reject_service_type_outside_local_acquisition(&parameter_value_type)?;
            parameter_types.push(self.resolve_value_type(&parameter_value_type)?);
        }
        let returned_value_type = parsed_function.returned_value_type();
        Self::reject_service_type_outside_local_acquisition(&returned_value_type)?;
        self.visible_function_signatures.push((
            parsed_function.function_name().to_owned(),
            parameter_types,
            self.resolve_value_type(&returned_value_type)?,
        ));
        Ok(())
    }

    /// Provides source-ordered function signatures for declaration and call checks.
    pub(super) fn visible_function_signatures(
        &self,
    ) -> &[(String, Vec<CheckedValueType>, CheckedValueType)] {
        &self.visible_function_signatures
    }

    /// Makes a validated parameter or local visible to following expressions.
    pub(super) fn add_local_binding(&mut self, local_binding: CheckedLocalBinding) {
        self.local_bindings.push(local_binding);
    }

    /// Records the active lexical-scope boundary before checking a nested body.
    pub(super) const fn local_scope_boundary(&self) -> usize {
        self.local_bindings.len()
    }

    /// Removes bindings introduced by a completed nested lexical scope.
    pub(super) fn end_local_scope(&mut self, local_scope_boundary: usize) {
        self.local_bindings.truncate(local_scope_boundary);
    }

    /// Provides active parameter and local bindings for collision and reference checks.
    pub(super) fn local_bindings(&self) -> &[CheckedLocalBinding] {
        &self.local_bindings
    }

    /// Provides the current function's checked return contract.
    pub(super) fn expected_returned_value_type(&self) -> CheckedValueType {
        self.expected_returned_value_type.clone()
    }

    /// Resolves a source type against the record aliases declared in this source file.
    pub(super) fn resolve_value_type(
        &self,
        parsed_value_type: &ParsedValueType,
    ) -> Result<CheckedValueType, CompilationProblem> {
        match parsed_value_type {
            ParsedValueType::Number => Ok(CheckedValueType::Number),
            ParsedValueType::String => Ok(CheckedValueType::String),
            ParsedValueType::Boolean => Ok(CheckedValueType::Boolean),
            ParsedValueType::Array(element_type) => Ok(CheckedValueType::Array(Box::new(
                self.resolve_value_type(element_type)?,
            ))),
            ParsedValueType::NoReturnedValues => Ok(CheckedValueType::NoReturnedValues),
            ParsedValueType::NamedRecord {
                record_name,
                record_name_range,
            } => {
                if let Some(roblox_service) = RobloxService::from_type_name(record_name) {
                    return Ok(CheckedValueType::RobloxService(roblox_service));
                }
                if self
                    .parsed_program
                    .parsed_records()
                    .iter()
                    .any(|record| record.record_name() == record_name)
                {
                    Ok(CheckedValueType::NamedRecord(record_name.to_owned()))
                } else {
                    Err(CompilationProblem::from_problem_at_range((
                        *record_name_range,
                        CompilationProblemReason::UnknownRecordType,
                    )))
                }
            }
        }
    }

    /// Resolves a local declaration while keeping service handles at their acquisition boundary.
    pub(super) fn resolve_local_value_type(
        &self,
        parsed_value_type: &ParsedValueType,
    ) -> Result<CheckedValueType, CompilationProblem> {
        if let Some(service_type_range) = Self::nested_service_type_range(parsed_value_type) {
            return Err(CompilationProblem::from_problem_at_range((
                service_type_range,
                CompilationProblemReason::RobloxServiceTypeMayOnlyBeUsedForLocalAcquisition,
            )));
        }
        self.resolve_value_type(parsed_value_type)
    }

    /// Resolves one intrinsic service only when project compilation supplies the module side.
    pub(super) fn acquire_roblox_service(
        &self,
        service_type_at_range: (&str, SourceRange),
    ) -> Result<RobloxService, CompilationProblem> {
        let (service_type_name, service_type_range) = service_type_at_range;
        let Some(roblox_service) = RobloxService::from_type_name(service_type_name) else {
            return Err(CompilationProblem::from_problem_at_range((
                service_type_range,
                CompilationProblemReason::UnknownRobloxService,
            )));
        };
        match self.service_acquisition_context {
            ServiceAcquisitionContext::Standalone => {
                Err(CompilationProblem::from_problem_at_range((
                    service_type_range,
                    CompilationProblemReason::RobloxServiceAcquisitionRequiresProjectCompilation,
                )))
            }
            ServiceAcquisitionContext::Project(execution_side)
                if roblox_service.is_available_on(execution_side) =>
            {
                Ok(roblox_service)
            }
            ServiceAcquisitionContext::Project(_) => {
                Err(CompilationProblem::from_problem_at_range((
                    service_type_range,
                    CompilationProblemReason::RobloxServiceUnavailableOnModuleExecutionSide,
                )))
            }
        }
    }

    fn reject_service_type_outside_local_acquisition(
        parsed_value_type: &ParsedValueType,
    ) -> Result<(), CompilationProblem> {
        let Some(service_type_range) = Self::service_type_range(parsed_value_type) else {
            return Ok(());
        };
        Err(CompilationProblem::from_problem_at_range((
            service_type_range,
            CompilationProblemReason::RobloxServiceTypeMayOnlyBeUsedForLocalAcquisition,
        )))
    }

    fn service_type_range(parsed_value_type: &ParsedValueType) -> Option<SourceRange> {
        match parsed_value_type {
            ParsedValueType::Array(element_type) => Self::service_type_range(element_type),
            ParsedValueType::NamedRecord {
                record_name,
                record_name_range,
            } if RobloxService::from_type_name(record_name).is_some() => Some(*record_name_range),
            ParsedValueType::Number
            | ParsedValueType::String
            | ParsedValueType::Boolean
            | ParsedValueType::NamedRecord { .. }
            | ParsedValueType::NoReturnedValues => None,
        }
    }

    fn nested_service_type_range(parsed_value_type: &ParsedValueType) -> Option<SourceRange> {
        match parsed_value_type {
            ParsedValueType::Array(element_type) => Self::service_type_range(element_type),
            ParsedValueType::Number
            | ParsedValueType::String
            | ParsedValueType::Boolean
            | ParsedValueType::NamedRecord { .. }
            | ParsedValueType::NoReturnedValues => None,
        }
    }

    /// Resolves a checked named record alias to its complete field declaration set.
    pub(super) fn checked_record_declaration(
        &self,
        record_name_at_range: (&str, SourceRange),
    ) -> Result<&CheckedRecordDeclaration, CompilationProblem> {
        let (record_name, record_name_range) = record_name_at_range;
        self.checked_record_declarations
            .iter()
            .find(|record| record.record_name() == record_name)
            .ok_or_else(|| {
                CompilationProblem::from_problem_at_range((
                    record_name_range,
                    CompilationProblemReason::UnknownRecordType,
                ))
            })
    }

    /// Moves the source-file record aliases into the checked program after all functions validate.
    pub(super) fn take_checked_record_declarations(&mut self) -> Vec<CheckedRecordDeclaration> {
        std::mem::take(&mut self.checked_record_declarations)
    }
}
