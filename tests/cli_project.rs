//! CLI contracts for manifest-backed project discovery, checking, and compilation.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::{Path, PathBuf};

fn temporary_directory(prefix: &str) -> tempfile::TempDir {
    match tempfile::Builder::new().prefix(prefix).tempdir() {
        Ok(directory) => directory,
        Err(error) => {
            assert!(false, "could not create temporary directory: {error}");
            std::process::exit(1)
        }
    }
}

fn cargo_bin() -> Command {
    if let Ok(command) = Command::cargo_bin("luau-rs") {
        return command;
    }
    assert!(false, "luau-rs binary unavailable");
    std::process::exit(1)
}

fn path_text(path: &Path) -> String {
    if let Some(text) = path.to_str() {
        return text.to_owned();
    }
    assert!(false, "path is not valid UTF-8: {}", path.display());
    String::new()
}

fn fixture_wasm_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/rust-hello/rust_hello.wasm")
}

fn write_project_manifest(path: &Path) {
    let manifest = "[project]\nsource_root = \"wasm\"\noutput_root = \"build\"\n";
    match fs_err::write(path, manifest) {
        Ok(()) => {}
        Err(error) => assert!(false, "could not write project manifest: {error}"),
    }
}

fn copy_fixture_module(root: &Path, relative_path: &str) -> PathBuf {
    let destination = root.join(relative_path);
    if let Some(parent_directory) = destination.parent() {
        match fs_err::create_dir_all(parent_directory) {
            Ok(()) => {}
            Err(error) => assert!(false, "could not create module directory: {error}"),
        }
    }
    match fs_err::copy(fixture_wasm_path(), &destination) {
        Ok(bytes_copied) => assert!(bytes_copied > 0, "fixture copy produced no bytes"),
        Err(error) => assert!(false, "could not copy fixture module: {error}"),
    }
    destination
}

fn read_bytes(path: &Path) -> Vec<u8> {
    match fs_err::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            assert!(false, "could not read {}: {error}", path.display());
            Vec::new()
        }
    }
}

/// `check` discovers and compiles modules without creating the output root.
#[test]
fn check_validates_manifest_project_without_writing_output() {
    let temp_dir = temporary_directory("luau-rs-project-check");
    copy_fixture_module(temp_dir.path(), "wasm/server/entrypoint/main.wasm");
    let manifest_path = temp_dir.path().join("luau-rs.toml");
    write_project_manifest(&manifest_path);

    cargo_bin()
        .args(["check", "--manifest-path", &path_text(&manifest_path)])
        .assert()
        .success();

    assert!(!temp_dir.path().join("build").exists());
}

/// `compile` recursively discovers side and role directories and publishes their layout.
#[test]
fn compile_discovers_nested_modules_and_publishes_roblox_paths() {
    let temp_dir = temporary_directory("luau-rs-project-compile");
    copy_fixture_module(temp_dir.path(), "wasm/shared/library/math/core.wasm");
    copy_fixture_module(temp_dir.path(), "wasm/server/entrypoint/game/main.wasm");
    let manifest_path = temp_dir.path().join("luau-rs.toml");
    write_project_manifest(&manifest_path);

    cargo_bin()
        .args(["compile", "--manifest-path", &path_text(&manifest_path)])
        .assert()
        .success();

    assert!(temp_dir
        .path()
        .join("build/ReplicatedStorage/math/core.luau")
        .exists());
    assert!(temp_dir
        .path()
        .join("build/ServerScriptService/game/main.server.luau")
        .exists());

    let stale_path = temp_dir.path().join("build/stale-managed-file.txt");
    match fs_err::write(&stale_path, "stale output") {
        Ok(()) => {}
        Err(error) => assert!(false, "could not create stale output: {error}"),
    }
    cargo_bin()
        .args(["compile", "--manifest-path", &path_text(&manifest_path)])
        .assert()
        .success();
    assert!(!stale_path.exists());
}

/// A malformed manifest reports the responsible project field instead of guessing defaults.
#[test]
fn check_reports_missing_manifest_field() {
    let temp_dir = temporary_directory("luau-rs-project-manifest-error");
    let manifest_path = temp_dir.path().join("luau-rs.toml");
    match fs_err::write(&manifest_path, "[project]\nsource_root = \"wasm\"\n") {
        Ok(()) => {}
        Err(error) => assert!(false, "could not write project manifest: {error}"),
    }

    cargo_bin()
        .args(["check", "--manifest-path", &path_text(&manifest_path)])
        .assert()
        .failure()
        .stderr(predicate::str::contains("output_root"));
}

/// A failed manifest build leaves the last successfully published tree intact.
#[test]
fn compile_failure_preserves_previous_output() {
    let temp_dir = temporary_directory("luau-rs-project-preserve");
    let wasm_path = copy_fixture_module(temp_dir.path(), "wasm/server/entrypoint/main.wasm");
    let manifest_path = temp_dir.path().join("luau-rs.toml");
    write_project_manifest(&manifest_path);
    let output_path = temp_dir
        .path()
        .join("build/ServerScriptService/main.server.luau");

    cargo_bin()
        .args(["compile", "--manifest-path", &path_text(&manifest_path)])
        .assert()
        .success();
    let previous_output = read_bytes(&output_path);

    match fs_err::write(&wasm_path, b"not a wasm module") {
        Ok(()) => {}
        Err(error) => assert!(false, "could not corrupt fixture module: {error}"),
    }
    cargo_bin()
        .args(["compile", "--manifest-path", &path_text(&manifest_path)])
        .assert()
        .failure()
        .stderr(predicate::str::contains("rejected"));

    assert_eq!(read_bytes(&output_path), previous_output);
}
