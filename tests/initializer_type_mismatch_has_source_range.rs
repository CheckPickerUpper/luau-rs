//! Integration coverage for initializer mismatch ranges.

use luau_rs::{compile_source, CompilationOutcome, CompilationProblemReason};

const INITIALIZER_START_BYTE: usize = 28;
const INITIALIZER_END_BYTE: usize = 36;

#[test]
fn an_initializer_type_mismatch_points_to_the_initial_value() {
    let source = "fn main() { let x: number = print(1); }\n";

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
    assert_eq!(source_range.start_byte(), INITIALIZER_START_BYTE);
    assert_eq!(source_range.end_byte(), INITIALIZER_END_BYTE);
}
