use std::cmp::Ordering;

use crate::{
    checked_program::{
        check_declaration_names::DeclarationNameChecker, check_expression::ExpressionChecker,
        check_function::FunctionChecker, program_check_context::ProgramCheckContext,
        CheckedProgram, CheckedValueType,
    },
    source_language::ParsedProgram,
    ArgumentCount, CompilationProblem, CompilationProblemReason,
};

/// Resolves source-ordered declarations and validates the program entrypoint contract.
pub fn check_parsed_program(
    parsed_program: &ParsedProgram,
) -> Result<CheckedProgram, CompilationProblem> {
    check_source_program((parsed_program, SourceEntrypointRequirement::Required))
}

/// Validates a source module whose functions are only reached through a future import surface.
pub fn check_parsed_library(
    parsed_program: &ParsedProgram,
) -> Result<CheckedProgram, CompilationProblem> {
    check_source_program((parsed_program, SourceEntrypointRequirement::NotRequired))
}

/// Checks a project entrypoint after project import resolution supplies its visible signatures.
pub fn check_project_entrypoint(
    project_source: (
        &ParsedProgram,
        &[ImportedFunctionSignature],
        crate::ModuleExecutionSide,
    ),
) -> Result<CheckedProgram, CompilationProblem> {
    check_project_source((
        project_source.0,
        project_source.1,
        SourceEntrypointRequirement::Required,
        super::program_check_context::ServiceAcquisitionContext::Project(project_source.2),
    ))
}

/// Checks a project library after project import resolution supplies its visible signatures.
pub fn check_project_library(
    project_source: (
        &ParsedProgram,
        &[ImportedFunctionSignature],
        crate::ModuleExecutionSide,
    ),
) -> Result<CheckedProgram, CompilationProblem> {
    check_project_source((
        project_source.0,
        project_source.1,
        SourceEntrypointRequirement::NotRequired,
        super::program_check_context::ServiceAcquisitionContext::Project(project_source.2),
    ))
}

fn check_source_program(
    source_program_check: (&ParsedProgram, SourceEntrypointRequirement),
) -> Result<CheckedProgram, CompilationProblem> {
    let (parsed_program, entrypoint_requirement) = source_program_check;
    check_project_source((
        parsed_program,
        &[],
        entrypoint_requirement,
        super::program_check_context::ServiceAcquisitionContext::Standalone,
    ))
}

fn check_project_source(
    source_program_check: (
        &ParsedProgram,
        &[ImportedFunctionSignature],
        SourceEntrypointRequirement,
        super::program_check_context::ServiceAcquisitionContext,
    ),
) -> Result<CheckedProgram, CompilationProblem> {
    let (parsed_program, imported_signatures, entrypoint_requirement, service_acquisition_context) =
        source_program_check;
    let mut check_context = ProgramCheckContext::from_parsed_program_and_imports((
        parsed_program,
        imported_signatures,
        service_acquisition_context,
    ));
    match check_context.register_record_declarations() {
        Ok(()) => {}
        Err(compilation_problem) => {
            return Err(check_context.attach_macro_backtrace(compilation_problem))
        }
    }
    let mut checked_functions = Vec::new();
    for parsed_function in parsed_program.parsed_functions() {
        match DeclarationNameChecker::check_function_name((&check_context, parsed_function)) {
            Ok(()) => {}
            Err(compilation_problem) => {
                return Err(check_context.attach_macro_backtrace(compilation_problem))
            }
        }
        match check_context.add_visible_function(parsed_function) {
            Ok(()) => {}
            Err(compilation_problem) => {
                return Err(check_context.attach_macro_backtrace(compilation_problem))
            }
        }
        let checked_function = {
            let mut function_checker = FunctionChecker::from_context(&mut check_context);
            match function_checker.check_function(parsed_function) {
                Ok(checked_function) => checked_function,
                Err(compilation_problem) => {
                    return Err(check_context.attach_macro_backtrace(compilation_problem))
                }
            }
        };
        checked_functions.push(checked_function);
    }
    match entrypoint_requirement {
        SourceEntrypointRequirement::Required => {
            match check_entrypoint_contract(&mut check_context) {
                Ok(()) => Ok(CheckedProgram::from_declarations((
                    check_context.take_checked_record_declarations(),
                    checked_functions,
                ))),
                Err(compilation_problem) => {
                    Err(check_context.attach_macro_backtrace(compilation_problem))
                }
            }
        }
        SourceEntrypointRequirement::NotRequired => Ok(CheckedProgram::from_declarations((
            check_context.take_checked_record_declarations(),
            checked_functions,
        ))),
    }
}

/// Supplies a resolved cross-module callable contract to body checking without exposing its origin.
#[derive(Clone)]
pub struct ImportedFunctionSignature {
    function_name: String,
    parameter_types: Vec<CheckedValueType>,
    returned_value_type: CheckedValueType,
}

/// Keeps imported signature construction at the semantic boundary where parsed types become checked types.
impl ImportedFunctionSignature {
    pub(crate) fn from_parts(
        signature_parts: (String, Vec<CheckedValueType>, CheckedValueType),
    ) -> Self {
        let (function_name, parameter_types, returned_value_type) = signature_parts;
        Self {
            function_name,
            parameter_types,
            returned_value_type,
        }
    }

    pub(crate) fn function_name(&self) -> &str {
        &self.function_name
    }

    pub(crate) fn parameter_types(&self) -> &[CheckedValueType] {
        &self.parameter_types
    }

    pub(crate) fn returned_value_type(&self) -> CheckedValueType {
        self.returned_value_type.clone()
    }
}

enum SourceEntrypointRequirement {
    Required,
    NotRequired,
}

fn check_entrypoint_contract(
    check_context: &mut ProgramCheckContext<'_>,
) -> Result<(), CompilationProblem> {
    let entrypoint_range = match check_context
        .parsed_program()
        .parsed_functions()
        .iter()
        .find(
            |parsed_function| match parsed_function.function_name().cmp("main") {
                Ordering::Equal => true,
                Ordering::Less | Ordering::Greater => false,
            },
        ) {
        Some(entrypoint_function) => entrypoint_function.function_name_range(),
        None => {
            return Err(CompilationProblem::from_problem_at_range((
                check_context.parsed_program().end_of_source_range(),
                CompilationProblemReason::MissingEntrypoint,
            )));
        }
    };
    let (entrypoint_parameters, entrypoint_returned_value_type) = {
        let expression_checker = ExpressionChecker::from_context(check_context);
        match expression_checker.resolve_function_signature(("main", entrypoint_range)) {
            Ok(function_signature) => function_signature,
            Err(compilation_problem) => return Err(compilation_problem),
        }
    };
    match entrypoint_parameters.len().cmp(&0) {
        Ordering::Equal => {}
        Ordering::Less | Ordering::Greater => {
            return Err(CompilationProblem::from_problem_at_range((
                entrypoint_range,
                CompilationProblemReason::WrongArgumentCount {
                    expected: ArgumentCount::from_number_of_arguments(0),
                    actual: ArgumentCount::from_number_of_arguments(entrypoint_parameters.len()),
                },
            )));
        }
    }
    match ExpressionChecker::require_matching_type((
        entrypoint_returned_value_type,
        CheckedValueType::NoReturnedValues,
        entrypoint_range,
    )) {
        Ok(()) => Ok(()),
        Err(compilation_problem) => Err(compilation_problem),
    }
}
