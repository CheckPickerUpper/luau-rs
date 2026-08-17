//! Integration coverage for validating generated Luau with official tools.

mod support;
use support::{official_luau_tool, run_official_luau_tool_required, temporary_luau_file};

use luau_rs::{compile_source, CompilationOutcome};

#[test]
fn official_luau_tools_execute_and_validate_generated_program() {
    let source = r"fn add(left: number, right: number) -> number {
    return left + right;
}

fn main() {
    let total: number = add(20, 22);
    print(total);
}
";

    let generated_luau_text = match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected validation fixture with {} problems",
                compilation_rejection.problem_count()
            );
            return;
        }
    };
    insta::assert_snapshot!(generated_luau_text);

    let generated_luau_path = temporary_luau_file("luau-rs-official-validation");
    match std::fs::write(generated_luau_path.path(), generated_luau_text) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write generated Luau fixture to {}: {write_error}",
                generated_luau_path.path().display()
            );
            return;
        }
    }

    let luau_path = official_luau_tool(("LUAU_BIN", "luau"));
    let luau_analyze_path = official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"));
    let luau_compile_path = official_luau_tool(("LUAU_COMPILE_BIN", "luau-compile"));

    let runtime_output = run_official_luau_tool_required((&luau_path, generated_luau_path.path()));
    assert!(
        runtime_output.status.success(),
        "official luau rejected execution:\n{}",
        String::from_utf8_lossy(&runtime_output.stderr)
    );
    let expected_runtime_output: &[u8] = if cfg!(windows) { b"42\r\n" } else { b"42\n" };
    assert_eq!(runtime_output.stdout, expected_runtime_output);

    let analysis_output =
        run_official_luau_tool_required((&luau_analyze_path, generated_luau_path.path()));
    assert!(
        analysis_output.status.success(),
        "official luau-analyze rejected generated Luau:\n{}",
        String::from_utf8_lossy(&analysis_output.stderr)
    );

    let compilation_output =
        run_official_luau_tool_required((&luau_compile_path, generated_luau_path.path()));
    assert!(
        compilation_output.status.success(),
        "official luau-compile rejected generated Luau:\n{}",
        String::from_utf8_lossy(&compilation_output.stderr)
    );
    assert!(!compilation_output.stdout.is_empty());
}
