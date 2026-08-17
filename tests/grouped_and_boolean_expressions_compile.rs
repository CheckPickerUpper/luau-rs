//! Integration coverage for grouped and boolean expressions.

use luau_rs::{compile_source, CompilationOutcome};

#[test]
fn grouped_comparative_and_logical_expressions_reach_luau_through_public_api() {
    let source = r#"fn main() {
    let grouped: number = (2 + 3) * 4;
    let ordered: boolean = grouped >= 20 && grouped != 21;
    let same_text: boolean = "luau" == "luau";
    let inverted: boolean = !false || same_text;
    print(grouped);
    print(ordered);
    print(inverted);
}
"#;

    let generated_luau_text = match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected grouped/boolean fixture with {} problems",
                compilation_rejection.problem_count()
            );
            return;
        }
    };

    insta::assert_snapshot!(generated_luau_text);
}
