//! Integration coverage for compiling string functions.

use luau_rs::{compile_source, CompilationOutcome};

#[test]
fn a_string_flows_through_a_function_and_print() {
    let source = r#"fn echo(message: string) -> string {
    return message;
}

fn main() {
    let greeting: string = echo("hello");
    print(greeting);
}
"#;
    let generated_luau_text = match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected string function fixture with {} problems",
                compilation_rejection.problem_count()
            );
            return;
        }
    };

    insta::assert_snapshot!(generated_luau_text);
}
