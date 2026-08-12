//! Integration coverage for executing conditional branches with Luau.

mod support;
use support::{official_luau_tool, run_official_luau_tool_required, temporary_luau_file};

use luau_rs::{compile_source, CompilationOutcome};

#[test]
fn if_else_branches_scope_locals_and_return_on_every_path() {
    let source = r"fn select_value(enabled: boolean) -> number {
    if enabled {
        let selected: number = 40;
        return selected + 2;
    } else {
        let fallback: number = 7;
        return fallback;
    }
}

fn main() {
    print(select_value(true));
    print(select_value(false));
}
";
    let generated_luau_text = match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected control-flow fixture with {} problems",
                compilation_rejection.problem_count()
            );
            return;
        }
    };
    insta::assert_snapshot!(generated_luau_text);
    let generated_luau_path = temporary_luau_file("luau-rs-if-else-runtime");
    match std::fs::write(generated_luau_path.path(), generated_luau_text) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write control-flow runtime fixture {}: {write_error}",
                generated_luau_path.path().display()
            );
            return;
        }
    }
    let luau_path = official_luau_tool(("LUAU_BIN", "luau"));
    let runtime_output = run_official_luau_tool_required((&luau_path, generated_luau_path.path()));
    assert!(
        runtime_output.status.success(),
        "official Luau rejected control-flow runtime fixture:\n{}",
        String::from_utf8_lossy(&runtime_output.stderr)
    );
    let expected_runtime_output: &[u8] = if cfg!(windows) {
        b"42\r\n7\r\n"
    } else {
        b"42\n7\n"
    };
    assert_eq!(runtime_output.stdout, expected_runtime_output);
}
