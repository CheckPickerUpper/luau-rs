//! Integration coverage for boolean type mismatch ranges.

use roblox_rust::{compile_source, CompilationOutcome, CompilationProblemReason};

const STRING_LITERAL_START_BYTE: usize = 32;
const STRING_LITERAL_END_BYTE: usize = 45;

#[test]
fn a_string_cannot_initialize_a_boolean_local() {
    let source = "fn main() { let flag: boolean = \"not boolean\"; }\n";

    let compilation_rejection = match compile_source(source) {
        CompilationOutcome::Rejected(compilation_rejection) => compilation_rejection,
        CompilationOutcome::Compiled(generated_luau_text) => {
            let generated_text = generated_luau_text.into_text();
            assert!(false, "expected rejection, generated: {generated_text}");
            return;
        }
    };
    let compilation_problem = compilation_rejection.first_problem();
    assert_eq!(
        compilation_problem.reason(),
        &CompilationProblemReason::TypesDoNotMatch
    );
    let source_range = compilation_problem.source_range();
    assert_eq!(source_range.start_byte(), STRING_LITERAL_START_BYTE);
    assert_eq!(source_range.end_byte(), STRING_LITERAL_END_BYTE);
}
