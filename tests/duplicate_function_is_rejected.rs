//! Integration coverage for duplicate function rejection.

use roblox_rust::{compile_source, CompilationOutcome, CompilationProblemReason};

const SECOND_FUNCTION_NAME_START_BYTE: usize = 16;
const SECOND_FUNCTION_NAME_END_BYTE: usize = 20;

#[test]
fn a_function_name_can_only_be_declared_once() {
    let source = "fn same() {}\nfn same() {}\nfn main() {}\n";

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
        &CompilationProblemReason::NameAlreadyDefined
    );
    let source_range = compilation_problem.source_range();
    assert_eq!(source_range.start_byte(), SECOND_FUNCTION_NAME_START_BYTE);
    assert_eq!(source_range.end_byte(), SECOND_FUNCTION_NAME_END_BYTE);
}
