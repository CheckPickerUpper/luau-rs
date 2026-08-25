//! CLI contracts for manifest-backed project discovery, checking, and compilation.

use predicates::prelude::*;
use std::path::{Path, PathBuf};

fn fixture_wasm_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/rust-hello/rust_hello.wasm")
}

fn write_project_manifest(path: &Path) -> std::result::Result<(), std::io::Error> {
    let manifest = "[project]\nsource_root = \"wasm\"\noutput_root = \"build\"\n";
    fs_err::write(path, manifest)
}

fn copy_fixture_module(
    root: &Path,
    relative_path: &str,
) -> std::result::Result<PathBuf, std::io::Error> {
    let destination = root.join(relative_path);
    if let Some(parent_directory) = destination.parent() {
        fs_err::create_dir_all(parent_directory)?;
    }
    let bytes_copied = fs_err::copy(fixture_wasm_path(), &destination)?;
    assert!(bytes_copied > 0, "fixture copy produced no bytes");
    Ok(destination)
}

/// `check` discovers and compiles modules without creating the output root.
#[test]
fn check_validates_manifest_project_without_writing_output(
) -> std::result::Result<(), std::io::Error> {
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-project-check")
        .tempdir()?;
    copy_fixture_module(temp_dir.path(), "wasm/server/entrypoint/main.wasm")?;
    let manifest_path = temp_dir.path().join("luau-rs.toml");
    write_project_manifest(&manifest_path)?;

    assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["check", "--manifest-path"])
        .arg(&manifest_path)
        .assert()
        .success();

    assert!(!temp_dir.path().join("build").exists());
    Ok(())
}

/// `compile` recursively discovers side and role directories and publishes their layout.
#[test]
fn compile_discovers_nested_modules_and_publishes_roblox_paths(
) -> std::result::Result<(), std::io::Error> {
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-project-compile")
        .tempdir()?;
    copy_fixture_module(temp_dir.path(), "wasm/shared/library/math/core.wasm")?;
    copy_fixture_module(temp_dir.path(), "wasm/server/entrypoint/game/main.wasm")?;
    let manifest_path = temp_dir.path().join("luau-rs.toml");
    write_project_manifest(&manifest_path)?;

    assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["compile", "--manifest-path"])
        .arg(&manifest_path)
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
    fs_err::write(&stale_path, "stale output")?;
    assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["compile", "--manifest-path"])
        .arg(&manifest_path)
        .assert()
        .success();
    assert!(!stale_path.exists());
    Ok(())
}

/// A malformed manifest reports the responsible project field instead of guessing defaults.
#[test]
fn check_reports_missing_manifest_field() -> std::result::Result<(), std::io::Error> {
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-project-manifest-error")
        .tempdir()?;
    let manifest_path = temp_dir.path().join("luau-rs.toml");
    fs_err::write(&manifest_path, "[project]\nsource_root = \"wasm\"\n")?;

    assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["check", "--manifest-path"])
        .arg(&manifest_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("output_root"));
    Ok(())
}

/// A failed manifest build leaves the last successfully published tree intact.
#[test]
fn compile_failure_preserves_previous_output() -> std::result::Result<(), std::io::Error> {
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-project-preserve")
        .tempdir()?;
    let wasm_path = copy_fixture_module(temp_dir.path(), "wasm/server/entrypoint/main.wasm")?;
    let manifest_path = temp_dir.path().join("luau-rs.toml");
    write_project_manifest(&manifest_path)?;
    let output_path = temp_dir
        .path()
        .join("build/ServerScriptService/main.server.luau");

    assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["compile", "--manifest-path"])
        .arg(&manifest_path)
        .assert()
        .success();
    let previous_output = fs_err::read(&output_path)?;

    fs_err::write(&wasm_path, b"not a wasm module")?;
    assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["compile", "--manifest-path"])
        .arg(&manifest_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("rejected"));

    assert_eq!(fs_err::read(&output_path)?, previous_output);
    Ok(())
}
