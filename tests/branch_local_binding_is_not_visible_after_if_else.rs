//! Integration coverage for branch-local binding visibility.

use roblox_rust::{compile_source, CompilationOutcome, CompilationProblemReason};

#[test]
fn a_branch_local_cannot_be_referenced_after_its_if_else_statement() {
    let source = r"fn main() {
    if true {
        let selected_number: number = 42;
        print(selected_number);
    } else {
        let fallback_number: number = 7;
        print(fallback_number);
    }
    print(selected_number);
}
";

    let compilation_rejection = match compile_source(source) {
        CompilationOutcome::Rejected(compilation_rejection) => compilation_rejection,
        CompilationOutcome::Compiled(generated_luau_text) => {
            assert!(
                false,
                "expected a branch-scope rejection, generated: {}",
                generated_luau_text.into_text()
            );
            return;
        }
    };
    assert_eq!(
        compilation_rejection.first_problem().reason(),
        &CompilationProblemReason::UnknownName
    );
}
