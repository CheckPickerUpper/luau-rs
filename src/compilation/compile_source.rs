use crate::{
    checked_program::check_parsed_program,
    generated_luau::{generate_luau_program, write_luau_text},
    source_language::{parse_source_program, split_source_into_tokens},
    CompilationOutcome, CompilationProblem, CompilationRejection,
};

/// @why Enforces the compiler's all-or-nothing artifact contract so downstream tools never execute text from a rejected program.
#[must_use]
pub fn compile_source(source: &str) -> CompilationOutcome {
    let source_tokens = match split_source_into_tokens(source) {
        Ok(source_tokens) => source_tokens,
        Err(compilation_problem) => return rejected(compilation_problem),
    };
    let parsed_program = match parse_source_program(source_tokens) {
        Ok(parsed_program) => parsed_program,
        Err(compilation_problem) => return rejected(compilation_problem),
    };
    let checked_program = match check_parsed_program(&parsed_program) {
        Ok(checked_program) => checked_program,
        Err(compilation_problem) => return rejected(compilation_problem),
    };
    let luau_program = generate_luau_program(&checked_program);
    CompilationOutcome::Compiled(write_luau_text(&luau_program))
}

const fn rejected(compilation_problem: CompilationProblem) -> CompilationOutcome {
    CompilationOutcome::Rejected(CompilationRejection::from_problem(compilation_problem))
}
