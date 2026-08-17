//! Integration coverage for compiling boolean functions.

use luau_rs::{compile_source, CompilationOutcome};

#[test]
fn both_boolean_literals_flow_through_a_function_and_print() {
    let source = r"fn identity(flag: boolean) -> boolean {
    return flag;
}

fn main() {
    let enabled: boolean = identity(true);
    let disabled: boolean = false;
    print(enabled);
    print(disabled);
}
";
    let generated_luau_text = match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected boolean fixture with {} problems",
                compilation_rejection.problem_count()
            );
            return;
        }
    };

    insta::assert_snapshot!(generated_luau_text);
}
