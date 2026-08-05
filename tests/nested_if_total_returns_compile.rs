//! Integration coverage for total nested conditional returns.

use roblox_rust::{compile_source, CompilationOutcome};

#[test]
fn nested_if_else_branches_satisfy_a_number_return_contract() {
    let source = r"fn select_value(outer: boolean, inner: boolean) -> number {
    if outer {
        if inner {
            return 1;
        } else {
            return 2;
        }
    } else {
        return 3;
    }
}

fn main() {
    print(select_value(true, false));
}
";

    match compile_source(source) {
        CompilationOutcome::Compiled(_) => {}
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected nested total-return fixture with {} problems",
                compilation_rejection.problem_count()
            );
        }
    }
}
