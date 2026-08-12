//! Integration coverage for executing generated numeric Luau.

mod support;
use support::{official_luau_tool, run_official_luau_tool_required, temporary_luau_file};

use luau_rs::{compile_source, CompilationOutcome};

#[test]
fn subtraction_multiplication_and_division_execute_with_official_luau() {
    let source = r"fn main() {
    let subtraction: number = 20 - 8;
    let multiplication: number = 6 * 7;
    let division: number = 84 / 2;
    print(subtraction);
    print(multiplication);
    print(division);
}
";
    let generated_luau_text = match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected numeric runtime fixture with {} problems",
                compilation_rejection.problem_count()
            );
            return;
        }
    };
    insta::assert_snapshot!(generated_luau_text);
    let generated_luau_path = temporary_luau_file("luau-rs-numeric-runtime");
    match std::fs::write(generated_luau_path.path(), generated_luau_text) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write numeric runtime fixture {}: {write_error}",
                generated_luau_path.path().display()
            );
            return;
        }
    }
    let luau_path = official_luau_tool(("LUAU_BIN", "luau"));
    let runtime_output = run_official_luau_tool_required((&luau_path, generated_luau_path.path()));
    assert!(
        runtime_output.status.success(),
        "official Luau rejected numeric runtime fixture:\n{}",
        String::from_utf8_lossy(&runtime_output.stderr)
    );
    let expected_runtime_output: &[u8] = if cfg!(windows) {
        b"12\r\n42\r\n42\r\n"
    } else {
        b"12\n42\n42\n"
    };
    assert_eq!(runtime_output.stdout, expected_runtime_output);
}
