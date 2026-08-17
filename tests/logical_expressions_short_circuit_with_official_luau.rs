//! Integration coverage for short-circuiting logical expressions in Luau.

mod support;
use support::{official_luau_tool, run_official_luau_tool_required, temporary_luau_file};

use luau_rs::{compile_source, CompilationOutcome};

#[test]
fn logical_expressions_short_circuit_with_official_luau() {
    let source = r#"fn produces_side_effect() -> boolean {
    print("called");
    return true;
}

fn main() {
    let conjunction: boolean = false && produces_side_effect();
    let disjunction: boolean = true || produces_side_effect();
    print(conjunction);
    print(disjunction);
}
"#;
    let generated_luau_text = match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected logical runtime fixture with {} problems",
                compilation_rejection.problem_count()
            );
            return;
        }
    };
    insta::assert_snapshot!(generated_luau_text);
    let generated_luau_path = temporary_luau_file("luau-rs-logical-runtime");
    match std::fs::write(generated_luau_path.path(), generated_luau_text) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write logical runtime fixture {}: {write_error}",
                generated_luau_path.path().display()
            );
            return;
        }
    }
    let luau_path = official_luau_tool(("LUAU_BIN", "luau"));
    let runtime_output = run_official_luau_tool_required((&luau_path, generated_luau_path.path()));
    assert!(
        runtime_output.status.success(),
        "official Luau rejected logical runtime fixture:\n{}",
        String::from_utf8_lossy(&runtime_output.stderr)
    );
    let expected_runtime_output: &[u8] = if cfg!(windows) {
        b"false\r\ntrue\r\n"
    } else {
        b"false\ntrue\n"
    };
    assert_eq!(runtime_output.stdout, expected_runtime_output);
}
