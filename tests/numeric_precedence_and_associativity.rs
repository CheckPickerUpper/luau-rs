//! Integration coverage for numeric precedence and associativity.

use luau_rs::{compile_source, CompilationOutcome};

#[test]
fn multiplication_binds_tighter_and_same_tier_operations_are_left_associative() {
    let source = r"fn main() {
    let precedence: number = 2 + 3 * 4;
    let order: number = 20 - 5 - 3;
    print(precedence);
    print(order);
}
";

    let generated_luau_text = match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected precedence fixture with {} problems",
                compilation_rejection.problem_count()
            );
            return;
        }
    };

    assert!(
        generated_luau_text.contains("const precedence: number = 2 + (3 * 4)"),
        "generated Luau changed multiplication precedence:\n{generated_luau_text}"
    );
    assert!(
        generated_luau_text.contains("const order: number = 20 - 5 - 3"),
        "generated Luau changed left associativity:\n{generated_luau_text}"
    );
}
