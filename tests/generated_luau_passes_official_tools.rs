use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use roblox_rust::{compile_source, CompilationOutcome};

#[test]
fn official_luau_tools_execute_and_validate_generated_program() {
    let source = r#"fn add(left: number, right: number) -> number {
    return left + right;
}

fn main() {
    let total: number = add(20, 22);
    print(total);
}
"#;

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

    let generated_luau_path = std::env::temp_dir().join(format!(
        "roblox-rust-official-validation-{}.luau",
        std::process::id()
    ));
    match std::fs::write(&generated_luau_path, generated_luau_text) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write generated Luau fixture to {}: {write_error}",
                generated_luau_path.display()
            );
            return;
        }
    }

    let luau_path = match resolve_official_luau_tool(("LUAU_BIN", "luau")) {
        Some(luau_path) => luau_path,
        None => {
            fail_missing_official_luau_tools();
            return;
        }
    };
    let luau_analyze_path = match resolve_official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze")) {
        Some(luau_analyze_path) => luau_analyze_path,
        None => {
            fail_missing_official_luau_tools();
            return;
        }
    };
    let luau_compile_path = match resolve_official_luau_tool(("LUAU_COMPILE_BIN", "luau-compile")) {
        Some(luau_compile_path) => luau_compile_path,
        None => {
            fail_missing_official_luau_tools();
            return;
        }
    };

    let runtime_output = match run_official_luau_tool((&luau_path, &generated_luau_path)) {
        Some(runtime_output) => runtime_output,
        None => return,
    };
    assert!(
        runtime_output.status.success(),
        "official luau rejected execution:\n{}",
        String::from_utf8_lossy(&runtime_output.stderr)
    );
    let expected_runtime_output: &[u8] = if cfg!(windows) { b"42\r\n" } else { b"42\n" };
    assert_eq!(runtime_output.stdout, expected_runtime_output);

    let analysis_output = match run_official_luau_tool((&luau_analyze_path, &generated_luau_path)) {
        Some(analysis_output) => analysis_output,
        None => return,
    };
    assert!(
        analysis_output.status.success(),
        "official luau-analyze rejected generated Luau:\n{}",
        String::from_utf8_lossy(&analysis_output.stderr)
    );

    let compilation_output =
        match run_official_luau_tool((&luau_compile_path, &generated_luau_path)) {
            Some(compilation_output) => compilation_output,
            None => return,
        };
    assert!(
        compilation_output.status.success(),
        "official luau-compile rejected generated Luau:\n{}",
        String::from_utf8_lossy(&compilation_output.stderr)
    );
    assert!(!compilation_output.stdout.is_empty());

    match std::fs::remove_file(&generated_luau_path) {
        Ok(()) => {}
        Err(remove_error) => assert!(
            false,
            "could not remove generated Luau fixture {}: {remove_error}",
            generated_luau_path.display()
        ),
    }
}

fn resolve_official_luau_tool(tool_name: (&str, &str)) -> Option<PathBuf> {
    let (environment_variable, executable_name) = tool_name;
    match std::env::var_os(environment_variable) {
        Some(configured_path) => Some(PathBuf::from(configured_path)),
        None => {
            let executable_filename = if cfg!(windows) {
                format!("{executable_name}.exe")
            } else {
                executable_name.to_owned()
            };
            let checkout_build_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("references")
                .join("checkouts")
                .join("luau")
                .join("build")
                .join("release")
                .join(executable_filename);
            if checkout_build_path.is_file() {
                Some(checkout_build_path)
            } else {
                None
            }
        }
    }
}

fn run_official_luau_tool(tool_and_source: (&Path, &Path)) -> Option<Output> {
    let (tool_path, generated_luau_path) = tool_and_source;
    match Command::new(tool_path).arg(generated_luau_path).output() {
        Ok(tool_output) => Some(tool_output),
        Err(execution_error) => {
            assert!(
                false,
                "could not invoke official Luau tool {}: {execution_error}",
                tool_path.display()
            );
            None
        }
    }
}

fn fail_missing_official_luau_tools() {
    assert!(
        false,
        "official Luau tools are required; set LUAU_BIN, LUAU_ANALYZE_BIN, and LUAU_COMPILE_BIN to executable paths or run `make -C references/checkouts/luau config=release luau luau-analyze luau-compile`"
    );
}
