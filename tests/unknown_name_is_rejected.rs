//! Integration coverage for unknown name rejection.

use luau_rs::{compile_source, CompilationOutcome, CompilationProblemReason};

const MISSING_NAME_START_BYTE: usize = 18;
const MISSING_NAME_END_BYTE: usize = 25;

#[test]
fn a_reference_to_an_unknown_name_is_rejected_at_its_source_range() {
    let source = "fn main() { print(missing); }\n";

    let compilation_rejection = match compile_source(source) {
        CompilationOutcome::Rejected(compilation_rejection) => compilation_rejection,
        CompilationOutcome::Compiled(generated_luau_text) => {
            let generated_text = generated_luau_text.into_text();
            assert!(false, "expected rejection, compiled: {generated_text}");
            return;
        }
    };

    assert_eq!(compilation_rejection.problem_count(), 1);
    let compilation_problem = compilation_rejection.first_problem();
    assert_eq!(
        compilation_problem.reason(),
        &CompilationProblemReason::UnknownName
    );
    let source_range = compilation_problem.source_range();
    assert_eq!(source_range.start_byte(), MISSING_NAME_START_BYTE);
    assert_eq!(source_range.end_byte(), MISSING_NAME_END_BYTE);
}
