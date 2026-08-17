//! Integration coverage for string type mismatch ranges.

use luau_rs::{compile_source, CompilationOutcome, CompilationProblemReason};

const NUMBER_LITERAL_START_BYTE: usize = 35;
const NUMBER_LITERAL_END_BYTE: usize = 36;

#[test]
fn a_number_cannot_initialize_a_string_local() {
    let source = "fn main() { let greeting: string = 1; }\n";

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
    assert_eq!(source_range.start_byte(), NUMBER_LITERAL_START_BYTE);
    assert_eq!(source_range.end_byte(), NUMBER_LITERAL_END_BYTE);
}
