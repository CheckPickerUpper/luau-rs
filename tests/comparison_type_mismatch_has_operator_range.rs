//! Integration coverage for comparison type mismatch ranges.

use roblox_rust::{compile_source, CompilationOutcome, CompilationProblemReason};

#[test]
fn equality_with_different_value_types_is_rejected_at_the_operator() {
    let source = "fn main() {\n    let invalid: boolean = 1 == true;\n}\n";
    let Some(operator_start) = source.find("==") else {
        assert!(false, "equality fixture lacks an equality operator");
        return;
    };

    match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => {
            assert!(
                false,
                "compiler accepted mismatched equality:\n{}",
                generated_luau_text.into_text()
            );
        }
        CompilationOutcome::Rejected(compilation_rejection) => {
            let compilation_problem = compilation_rejection.first_problem();
            assert_eq!(
                compilation_problem.reason(),
                &CompilationProblemReason::TypesDoNotMatch
            );
            assert_eq!(
                compilation_problem.source_range().start_byte(),
                operator_start
            );
            assert_eq!(
                compilation_problem.source_range().end_byte(),
                operator_start + 2
            );
        }
    }
}
