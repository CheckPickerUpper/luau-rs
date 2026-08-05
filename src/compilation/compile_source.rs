use crate::{
    checked_program::{check_parsed_library, check_parsed_program},
    generated_luau::{generate_luau_library, generate_luau_program, write_luau_text},
    source_language::{parse_source_program, split_source_into_tokens},
    CompilationOutcome, CompilationProblem, CompilationRejection,
};

/// @why Enforces the compiler's all-or-nothing artifact contract so downstream tools never execute text from a rejected program.
#[must_use]
pub fn compile_source(source: &str) -> CompilationOutcome {
    compile_source_for_purpose((source, SourceCompilationPurpose::Entrypoint))
}

/// Compiles a source module without requiring or executing a `main` function.
#[must_use]
pub fn compile_library_source(source: &str) -> CompilationOutcome {
    compile_source_for_purpose((source, SourceCompilationPurpose::Library))
}

fn compile_source_for_purpose(
    source_compilation: (&str, SourceCompilationPurpose),
) -> CompilationOutcome {
    let (source, source_purpose) = source_compilation;
    let source_tokens = match split_source_into_tokens(source) {
        Ok(source_tokens) => source_tokens,
        Err(compilation_problem) => return rejected(compilation_problem),
    };
    let parsed_program = match parse_source_program(source_tokens) {
        Ok(parsed_program) => parsed_program,
        Err(compilation_problem) => return rejected(compilation_problem),
    };
    let Some(first_project_import) = parsed_program.parsed_imports().first() else {
        return compile_parsed_source((parsed_program, source_purpose));
    };
    rejected(CompilationProblem::from_problem_at_range((
        first_project_import.import_range(),
        crate::CompilationProblemReason::ProjectImportRequiresProjectCompilation,
    )))
}

fn compile_parsed_source(
    parsed_source: (
        crate::source_language::ParsedProgram,
        SourceCompilationPurpose,
    ),
) -> CompilationOutcome {
    let (parsed_program, source_purpose) = parsed_source;
    let checked_program_result = match source_purpose {
        SourceCompilationPurpose::Entrypoint => check_parsed_program(&parsed_program),
        SourceCompilationPurpose::Library => check_parsed_library(&parsed_program),
    };
    let checked_program = match checked_program_result {
        Ok(checked_program) => checked_program,
        Err(compilation_problem) => return rejected(compilation_problem),
    };
    let luau_program = match source_purpose {
        SourceCompilationPurpose::Entrypoint => generate_luau_program(&checked_program),
        SourceCompilationPurpose::Library => generate_luau_library(&checked_program),
    };
    CompilationOutcome::Compiled(write_luau_text(&luau_program))
}

#[derive(Clone, Copy)]
enum SourceCompilationPurpose {
    Entrypoint,
    Library,
}

const fn rejected(compilation_problem: CompilationProblem) -> CompilationOutcome {
    CompilationOutcome::Rejected(CompilationRejection::from_problem(compilation_problem))
}
