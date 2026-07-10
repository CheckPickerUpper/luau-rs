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
pub(crate) fn check_parsed_program(
    parsed_program: &ParsedProgram,
) -> Result<CheckedProgram, CompilationProblem> {
    let mut check_context = ProgramCheckContext::from_parsed_program(parsed_program);
    let mut checked_functions = Vec::new();
    for parsed_function in parsed_program.parsed_functions() {
        match DeclarationNameChecker::check_function_name((&check_context, parsed_function)) {
            Ok(()) => {}
            Err(compilation_problem) => return Err(compilation_problem),
        }
        check_context.add_visible_function(parsed_function);
        let checked_function = {
            let mut function_checker = FunctionChecker::from_context(&mut check_context);
            match function_checker.check_function(parsed_function) {
                Ok(checked_function) => checked_function,
                Err(compilation_problem) => return Err(compilation_problem),
            }
        };
        checked_functions.push(checked_function);
    }
    match check_entrypoint_contract(&mut check_context) {
        Ok(()) => Ok(CheckedProgram::from_functions(checked_functions)),
        Err(compilation_problem) => Err(compilation_problem),
    }
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
