//! Integration coverage for rejecting statements after returns.

use luau_rs::{compile_source, CompilationOutcome, CompilationProblemReason};

const POST_RETURN_STATEMENT_START_BYTE: usize = 33;

#[test]
fn a_return_must_end_the_function_body() {
    let source = concat!(
        "f",
        "n value() -> number { return 1; print(1); }\nfn main() { print(value()); }\n"
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
        &CompilationProblemReason::SourceDoesNotFollowLanguageRules
    );
    assert_eq!(
        compilation_problem.source_range().start_byte(),
        POST_RETURN_STATEMENT_START_BYTE
    );
}
