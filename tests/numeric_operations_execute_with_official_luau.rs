//! Integration coverage for executing generated numeric Luau.

use std::{path::Path, process::Command};

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
    let generated_luau_path = std::env::temp_dir().join(format!(
        "roblox-rust-numeric-runtime-{}.luau",
        std::process::id()
    ));
    match std::fs::write(&generated_luau_path, generated_luau_text) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write numeric runtime fixture {}: {write_error}",
                generated_luau_path.display()
            );
            return;
        }
    }
    let Some(luau_path) = resolve_official_luau_path() else {
        assert!(
            false,
            "official luau is required; set LUAU_BIN or build references/checkouts/luau"
        );
        return;
    };
    let runtime_output = match Command::new(&luau_path).arg(&generated_luau_path).output() {
        Ok(runtime_output) => runtime_output,
        Err(execution_error) => {
            assert!(
                false,
                "could not invoke official Luau {}: {execution_error}",
                luau_path.display()
            );
            return;
        }
    };
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
    match std::fs::remove_file(&generated_luau_path) {
        Ok(()) => {}
        Err(remove_error) => assert!(
            false,
            "could not remove numeric runtime fixture {}: {remove_error}",
            generated_luau_path.display()
        ),
    }
}

fn resolve_official_luau_path() -> Option<std::path::PathBuf> {
    std::env::var_os("LUAU_BIN").map_or_else(
        || {
            let executable_name = if cfg!(windows) { "luau.exe" } else { "luau" };
            let checkout_build_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("references")
                .join("checkouts")
                .join("luau")
                .join("build")
                .join("release")
                .join(executable_name);
            if checkout_build_path.is_file() {
                Some(checkout_build_path)
            } else {
                None
            }
        },
        |configured_path| Some(std::path::PathBuf::from(configured_path)),
    )
}
