//! Integration coverage for boolean conditional requirements.

use roblox_rust::{compile_source, CompilationOutcome, CompilationProblemReason};

const NUMBER_CONDITION_START_BYTE: usize = 15;
const NUMBER_CONDITION_END_BYTE: usize = 16;

#[test]
fn an_if_condition_must_produce_a_boolean() {
    let source = "fn main() { if 1 { print(42); } else { print(7); } }\n";

    let compilation_rejection = match compile_source(source) {
        CompilationOutcome::Rejected(compilation_rejection) => compilation_rejection,
        CompilationOutcome::Compiled(generated_luau_text) => {
            assert!(
                false,
                "expected a non-boolean condition rejection, generated: {}",
                generated_luau_text.into_text()
            );
            return;
        }
    };
    let compilation_problem = compilation_rejection.first_problem();
    assert_eq!(
        compilation_problem.reason(),
        &CompilationProblemReason::TypesDoNotMatch
    );
    assert_eq!(
        compilation_problem.source_range().start_byte(),
        NUMBER_CONDITION_START_BYTE
    );
    assert_eq!(
        compilation_problem.source_range().end_byte(),
        NUMBER_CONDITION_END_BYTE
    );
}
