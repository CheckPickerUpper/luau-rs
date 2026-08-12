//! Integration coverage for executing conditional branches with Luau.

use std::{path::Path, process::Command};

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
    let generated_luau_path = std::env::temp_dir().join(format!(
        "roblox-rust-if-else-runtime-{}.luau",
        std::process::id()
    ));
    match std::fs::write(&generated_luau_path, generated_luau_text) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write control-flow runtime fixture {}: {write_error}",
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
        "official Luau rejected control-flow runtime fixture:\n{}",
        String::from_utf8_lossy(&runtime_output.stderr)
    );
    let expected_runtime_output: &[u8] = if cfg!(windows) {
        b"42\r\n7\r\n"
    } else {
        b"42\n7\n"
    };
    assert_eq!(runtime_output.stdout, expected_runtime_output);
    match std::fs::remove_file(&generated_luau_path) {
        Ok(()) => {}
        Err(remove_error) => assert!(
            false,
            "could not remove control-flow runtime fixture {}: {remove_error}",
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
