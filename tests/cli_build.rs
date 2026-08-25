//! CLI tests: `luau-rs build` compiles the fixture into a Roblox layout.

use assert_cmd::Command;
use predicates::prelude::*;

/// Creates a temporary directory for CLI output.
fn temporary_directory(prefix: &str) -> tempfile::TempDir {
    match tempfile::Builder::new().prefix(prefix).tempdir() {
        Ok(directory) => directory,
        Err(error) => {
            assert!(false, "could not create temporary directory: {error}");
            std::process::exit(1)
        }
    }
}

/// The compiled `luau-rs` binary as an `assert_cmd` command.
fn cargo_bin() -> Command {
    if let Ok(command) = Command::cargo_bin("luau-rs") {
        return command;
    }
    assert!(false, "luau-rs binary unavailable");
    std::process::exit(1)
}

/// Renders a path as a UTF-8 string for CLI arguments.
fn path_text(path: &std::path::Path) -> String {
    if let Some(text) = path.to_str() {
        return text.to_string();
    }
    assert!(false, "path is not valid UTF-8: {}", path.display());
    String::new()
}

/// The fixture wasm committed from `fixtures/rust-hello`.
fn fixture_wasm_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/rust-hello/rust_hello.wasm")
}

/// Reads a generated artifact, failing the test when it is missing.
fn read_artifact(path: &std::path::Path) -> String {
    match fs_err::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            assert!(false, "could not read artifact {}: {error}", path.display());
            String::new()
        }
    }
}

/// `luau-rs build` writes the entrypoint script to the Roblox layout path.

#[test]
fn build_writes_entrypoint_script() {
    let temp_dir = temporary_directory("luau-rs-cli");
    let output_root = temp_dir.path().join("build");

    cargo_bin()
        .args([
            "build",
            &path_text(&fixture_wasm_path()),
            "--out",
            &path_text(&output_root),
            "--entrypoint",
        ])
        .assert()
        .success();

    let artifact = output_root.join("ServerScriptService/main.server.luau");
    assert!(
        artifact.exists(),
        "entrypoint artifact missing at {}",
        artifact.display()
    );
    let text = read_artifact(&artifact);
    assert!(
        text.starts_with("--!strict"),
        "artifact must be strict Luau"
    );
    assert!(
        text.contains("instantiate"),
        "artifact must export a factory"
    );
}

/// `luau-rs build` rejects a non-wasm file with a typed error.
#[test]
fn build_rejects_garbage_input() {
    let temp_dir = temporary_directory("luau-rs-cli-garbage");
    let garbage_path = temp_dir.path().join("garbage.wasm");
    match fs_err::write(&garbage_path, b"definitely not wasm") {
        Ok(()) => {}
        Err(error) => {
            assert!(false, "could not write garbage file: {error}");
            return;
        }
    }
    let output_root = temp_dir.path().join("build");

    cargo_bin()
        .args([
            "build",
            &path_text(&garbage_path),
            "--out",
            &path_text(&output_root),
            "--entrypoint",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("rejected"));
}

/// `luau-rs build` with `--side client` places the script under
/// `StarterPlayerScripts`.
#[test]
fn build_client_side_uses_starter_player_path() {
    let temp_dir = temporary_directory("luau-rs-cli-client");
    let output_root = temp_dir.path().join("build");

    cargo_bin()
        .args([
            "build",
            &path_text(&fixture_wasm_path()),
            "--out",
            &path_text(&output_root),
            "--entrypoint",
            "--side",
            "client",
            "--module-path",
            "game/main",
        ])
        .assert()
        .success();

    let artifact = output_root.join("StarterPlayer/StarterPlayerScripts/game/main.client.luau");
    assert!(
        artifact.exists(),
        "client artifact missing at {}",
        artifact.display()
    );
}
