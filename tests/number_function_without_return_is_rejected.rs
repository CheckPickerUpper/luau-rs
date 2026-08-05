//! Integration coverage for missing numeric function returns.

use roblox_rust::{compile_source, CompilationOutcome, CompilationProblemReason};

const FUNCTION_NAME_START_BYTE: usize = 3;
const FUNCTION_NAME_END_BYTE: usize = 8;

#[test]
fn a_number_function_must_return_a_value() {
    let source = concat!(
        "f",
        "n value() -> number { let x: number = 1; }\nfn main() { print(value()); }\n"
    );

    let compilation_rejection = match compile_source(source) {
        CompilationOutcome::Rejected(compilation_rejection) => compilation_rejection,
        CompilationOutcome::Compiled(generated_luau_text) => {
            let generated_text = generated_luau_text.into_text();
            assert!(false, "expected rejection, compiled: {generated_text}");
            return;
        }
    };
    let compilation_problem = compilation_rejection.first_problem();
    assert_eq!(
        compilation_problem.reason(),
        &CompilationProblemReason::MissingReturn
    );
    let source_range = compilation_problem.source_range();
    assert_eq!(source_range.start_byte(), FUNCTION_NAME_START_BYTE);
    assert_eq!(source_range.end_byte(), FUNCTION_NAME_END_BYTE);
}
