//! Integration coverage for rejecting bare value statements.

use roblox_rust::{compile_source, CompilationOutcome, CompilationProblemReason};

const VALUE_START_BYTE: usize = 12;
const VALUE_END_BYTE: usize = 13;

#[test]
fn a_bare_value_cannot_be_used_as_a_statement() {
    let source = "fn main() { 1; }\n";

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
        &CompilationProblemReason::SourceDoesNotFollowLanguageRules
    );
    let source_range = compilation_problem.source_range();
    assert_eq!(source_range.start_byte(), VALUE_START_BYTE);
    assert_eq!(source_range.end_byte(), VALUE_END_BYTE);
}
