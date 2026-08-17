//! Integration coverage for multiplication and division compilation.

use luau_rs::{compile_source, CompilationOutcome};

#[test]
fn multiplication_and_division_reach_generated_luau_through_public_api() {
    let source = r"fn main() {
    let product: number = 6 * 7;
    let quotient: number = 84 / 2;
    print(product);
    print(quotient);
}
";

    let generated_luau_text = match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected multiplication/division fixture with {} problems",
                compilation_rejection.problem_count()
            );
            return;
        }
    };

    insta::assert_snapshot!(generated_luau_text);
}
