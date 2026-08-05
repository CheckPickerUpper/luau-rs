//! Integration coverage for subtraction compilation.

use roblox_rust::{compile_source, CompilationOutcome};

#[test]
fn subtraction_reaches_generated_luau_through_public_api() {
    let source = r"fn main() {
    let total: number = 20 - 8;
    print(total);
}
";

    let generated_luau_text = match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected subtraction fixture with {} problems",
                compilation_rejection.problem_count()
            );
            return;
        }
    };

    assert!(
        generated_luau_text.contains("const total: number = 20 - 8"),
        "generated Luau did not preserve subtraction:\n{generated_luau_text}"
    );
}
