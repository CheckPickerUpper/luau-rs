//! Integration coverage for missing returns after conditional branches.

use roblox_rust::{compile_source, CompilationOutcome, CompilationProblemReason};

const FUNCTION_NAME_START_BYTE: usize = 3;
const FUNCTION_NAME_END_BYTE: usize = 17;

#[test]
fn a_value_function_needs_both_if_else_branches_to_return() {
    let source = r"fn selected_value(enabled: boolean) -> number {
    if enabled {
        return 42;
    } else {
        print(7);
    }
}

fn main() {
    print(selected_value(true));
}
";

    let compilation_rejection = match compile_source(source) {
        CompilationOutcome::Rejected(compilation_rejection) => compilation_rejection,
        CompilationOutcome::Compiled(generated_luau_text) => {
            assert!(
                false,
                "expected a missing-return rejection, generated: {}",
                generated_luau_text.into_text()
            );
            return;
        }
    };
    let compilation_problem = compilation_rejection.first_problem();
    assert_eq!(
        compilation_problem.reason(),
        &CompilationProblemReason::MissingReturn
    );
    assert_eq!(
        compilation_problem.source_range().start_byte(),
        FUNCTION_NAME_START_BYTE
    );
    assert_eq!(
        compilation_problem.source_range().end_byte(),
        FUNCTION_NAME_END_BYTE
    );
}
