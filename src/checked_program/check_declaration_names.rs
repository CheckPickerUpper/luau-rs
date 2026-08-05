use std::cmp::Ordering;

use crate::{
    checked_program::program_check_context::ProgramCheckContext, source_language::ParsedFunction,
    CompilationProblem, CompilationProblemReason, SourceRange,
};

/// Validates declaration names against target, builtin, and visible-source constraints.
pub(super) struct DeclarationNameChecker;

/// Keeps every raw-name rejection rule at the checked-program boundary.
impl DeclarationNameChecker {
    /// Rejects a function name before it enters the source-ordered visible signature set.
    pub(super) fn check_function_name(
        function_name_check: (&ProgramCheckContext<'_>, &ParsedFunction),
    ) -> Result<(), CompilationProblem> {
        let (check_context, parsed_function) = function_name_check;
        match Self::check_builtin_name((
            parsed_function.function_name(),
            parsed_function.function_name_range(),
        )) {
            Ok(()) => {}
            Err(compilation_problem) => return Err(compilation_problem),
        }
        match Self::check_luau_name((
            parsed_function.function_name(),
            parsed_function.function_name_range(),
        )) {
            Ok(()) => {}
            Err(compilation_problem) => return Err(compilation_problem),
        }
        for (visible_name, _, _) in check_context.visible_function_signatures() {
            match visible_name.as_str().cmp(parsed_function.function_name()) {
                Ordering::Equal => {
                    return Err(CompilationProblem::from_problem_at_range((
                        parsed_function.function_name_range(),
                        CompilationProblemReason::NameAlreadyDefined,
                    )));
                }
                Ordering::Less | Ordering::Greater => {}
            }
        }
        Ok(())
    }

    /// Rejects a parameter or local name before it enters the active function scope.
    pub(super) fn check_local_name(
        local_name_check: (&ProgramCheckContext<'_>, &str, SourceRange),
    ) -> Result<(), CompilationProblem> {
        let (check_context, local_name, local_name_range) = local_name_check;
        match Self::check_builtin_name((local_name, local_name_range)) {
            Ok(()) => {}
            Err(compilation_problem) => return Err(compilation_problem),
        }
        match Self::check_luau_name((local_name, local_name_range)) {
            Ok(()) => {}
            Err(compilation_problem) => return Err(compilation_problem),
        }
        for (visible_function_name, _, _) in check_context.visible_function_signatures() {
            match visible_function_name.as_str().cmp(local_name) {
                Ordering::Equal => {
                    return Err(CompilationProblem::from_problem_at_range((
                        local_name_range,
                        CompilationProblemReason::NameAlreadyDefined,
                    )));
                }
                Ordering::Less | Ordering::Greater => {}
            }
        }
        for visible_local_binding in check_context.local_bindings() {
            match visible_local_binding.local_name().cmp(local_name) {
                Ordering::Equal => {
                    return Err(CompilationProblem::from_problem_at_range((
                        local_name_range,
                        CompilationProblemReason::NameAlreadyDefined,
                    )));
                }
                Ordering::Less | Ordering::Greater => {}
            }
        }
        Ok(())
    }

    fn check_builtin_name(
        source_name_at_range: (&str, SourceRange),
    ) -> Result<(), CompilationProblem> {
        let (source_name, source_range) = source_name_at_range;
        match source_name.cmp("print") {
            Ordering::Equal => Err(CompilationProblem::from_problem_at_range((
                source_range,
                CompilationProblemReason::NameReservedForBuiltInFunction,
            ))),
            Ordering::Less | Ordering::Greater => Ok(()),
        }
    }

    fn check_luau_name(
        source_name_at_range: (&str, SourceRange),
    ) -> Result<(), CompilationProblem> {
        let (source_name, source_range) = source_name_at_range;
        match source_name {
            "and" | "break" | "do" | "else" | "elseif" | "end" | "false" | "for" | "function"
            | "if" | "in" | "local" | "nil" | "not" | "or" | "repeat" | "return" | "then"
            | "true" | "until" | "while" | "continue" | "type" | "export" | "typeof" | "const" => {
                Err(CompilationProblem::from_problem_at_range((
                    source_range,
                    CompilationProblemReason::NameNotAllowedInLuau,
                )))
            }
            _ => Ok(()),
        }
    }
}
