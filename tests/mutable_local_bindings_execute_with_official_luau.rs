//! Verifies generated Luau preserves mutable local binding semantics at runtime.

mod support;
use support::{official_luau_tool, run_official_luau_tool_required, temporary_luau_file};

use full_moon::ast::LuaVersion;
use luau_rs::{compile_source, CompilationOutcome};

#[test]
fn mutable_number_string_and_boolean_locals_update_in_official_luau() {
    let source = r#"fn main() {
    let mut count: number = 1;
    let mut message: string = "before";
    let mut enabled: boolean = false;
    count = count + 41;
    message = "after";
    enabled = true;
    print(count);
    print(message);
    print(enabled);
}
"#;
    let generated_luau = match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected mutable-local fixture with {} problems",
                compilation_rejection.problem_count()
            );
            return;
        }
    };
    insta::assert_snapshot!(generated_luau);

    match full_moon::parse_fallible(&generated_luau, LuaVersion::luau()).into_result() {
        Ok(_) => {}
        Err(parse_errors) => {
            assert!(
                false,
                "Full Moon rejected mutable-local Luau: {parse_errors:?}"
            );
            return;
        }
    }

    let generated_luau_path = temporary_luau_file("luau-rs-mutable-locals");
    match std::fs::write(generated_luau_path.path(), generated_luau) {
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

    let runtime_output = run_official_luau_tool_required((&luau_path, generated_luau_path.path()));
    assert!(
        runtime_output.status.success(),
        "official luau rejected mutable-local execution:\n{}",
        String::from_utf8_lossy(&runtime_output.stderr)
    );
    let expected_runtime_output: &[u8] = if cfg!(windows) {
        b"42\r\nafter\r\ntrue\r\n"
    } else {
        b"42\nafter\ntrue\n"
    };
    assert_eq!(runtime_output.stdout, expected_runtime_output);

    let analysis_output =
        run_official_luau_tool_required((&luau_analyze_path, generated_luau_path.path()));
    assert!(
        analysis_output.status.success(),
        "official luau-analyze rejected mutable-local Luau:\n{}",
        String::from_utf8_lossy(&analysis_output.stderr)
    );
}
