//! Behavior scenarios for manifest-backed project workflows.

use predicates::prelude::*;
use rstest::rstest;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

fn fixture_wasm_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/rust-hello/rust_hello.wasm")
}

fn write_project_manifest(path: &Path) -> Result<(), Error> {
    fs_err::write(
        path,
        "[project]\nsource_root = \"wasm\"\noutput_root = \"build\"\n",
    )
}

fn copy_fixture_module(root: &Path, relative_path: &str) -> Result<PathBuf, Error> {
    let destination = root.join(relative_path);
    if let Some(parent_directory) = destination.parent() {
        fs_err::create_dir_all(parent_directory)?;
    }
    fs_err::copy(fixture_wasm_path(), &destination)?;
    Ok(destination)
}

#[rstest]
fn given_valid_project_when_checked_then_workspace_stays_unchanged() -> Result<(), Error> {
    // Given a project with a valid server module and no generated output yet.
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-project-bdd-check")
        .tempdir()?;
    copy_fixture_module(temp_dir.path(), "wasm/server/entrypoint/main.wasm")?;
    let manifest = temp_dir.path().join("luau-rs.toml");
    write_project_manifest(&manifest)?;

    // When the project manifest is checked.
    let command = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["check", "--manifest-path"])
        .arg(&manifest)
        .output()?;

    // Then validation succeeds without creating Roblox output.
    let success = command.status.success();
    if !success {
        return Err(Error::other(format!(
            "project check failed: success={success}, stderr={}",
            String::from_utf8_lossy(&command.stderr)
        )));
    }
    let output_exists = temp_dir.path().join("build").exists();
    if output_exists {
        Err(Error::other(format!(
            "project check created Roblox output: output_exists={output_exists}"
        )))
    } else {
        Ok(())
    }
}

#[rstest]
fn given_nested_project_when_compiled_twice_then_modules_map_and_stale_file_disappears(
) -> Result<(), Error> {
    // Given a project with a shared library, a server entrypoint, and a manifest.
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-project-bdd-layout")
        .tempdir()?;
    copy_fixture_module(temp_dir.path(), "wasm/shared/library/math/core.wasm")?;
    copy_fixture_module(temp_dir.path(), "wasm/server/entrypoint/game/main.wasm")?;
    let manifest = temp_dir.path().join("luau-rs.toml");
    write_project_manifest(&manifest)?;

    // When the project is compiled, then compiled again after a stale managed file is added.
    let first_compile = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["compile", "--manifest-path"])
        .arg(&manifest)
        .output()?;
    let first_success = first_compile.status.success();
    if !first_success {
        return Err(Error::other(format!(
            "first project compile failed: success={first_success}, stderr={}",
            String::from_utf8_lossy(&first_compile.stderr)
        )));
    }
    let stale_path = temp_dir.path().join("build/stale-managed-file.txt");
    fs_err::write(&stale_path, "stale output")?;
    let second_compile = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["compile", "--manifest-path"])
        .arg(&manifest)
        .output()?;

    // Then the modules appear in their Roblox containers and the stale file disappears.
    let second_success = second_compile.status.success();
    if !second_success {
        return Err(Error::other(format!(
            "second project compile failed: success={second_success}, stderr={}",
            String::from_utf8_lossy(&second_compile.stderr)
        )));
    }
    let shared_path = temp_dir
        .path()
        .join("build/ReplicatedStorage/math/core.luau");
    let shared_exists = predicate::path::exists().eval(&shared_path);
    if !shared_exists {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "shared library was not published: exists={shared_exists}, path={}",
                shared_path.display()
            ),
        ));
    }
    let server_path = temp_dir
        .path()
        .join("build/ServerScriptService/game/main.server.luau");
    let server_exists = predicate::path::exists().eval(&server_path);
    if !server_exists {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "server entrypoint was not published: exists={server_exists}, path={}",
                server_path.display()
            ),
        ));
    }
    let stale_exists = stale_path.exists();
    if stale_exists {
        Err(Error::other(format!(
            "stale managed file survived recompilation: stale_exists={stale_exists}"
        )))
    } else {
        Ok(())
    }
}

#[rstest]
fn given_manifest_without_output_location_when_checked_then_missing_field_is_named(
) -> Result<(), Error> {
    // Given a project manifest that omits where generated files should go.
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-project-bdd-invalid")
        .tempdir()?;
    let manifest = temp_dir.path().join("luau-rs.toml");
    fs_err::write(&manifest, "[project]\nsource_root = \"wasm\"\n")?;

    // When the incomplete project manifest is checked.
    let command = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["check", "--manifest-path"])
        .arg(&manifest)
        .output()?;

    // Then validation fails and names the missing output location.
    let success = command.status.success();
    if success {
        return Err(Error::other(format!(
            "incomplete manifest unexpectedly passed: success={success}"
        )));
    }
    let stderr = String::from_utf8_lossy(&command.stderr);
    let names_output_root = predicate::str::contains("output_root").eval(&stderr);
    if names_output_root {
        Ok(())
    } else {
        Err(Error::other(format!(
            "manifest failure omitted output_root: names_output_root={names_output_root}, stderr={stderr}"
        )))
    }
}

#[rstest]
fn given_good_project_when_recompiled_with_corrupt_module_then_last_output_survives(
) -> Result<(), Error> {
    // Given a project with a valid server module and a manifest.
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-project-bdd-preserve")
        .tempdir()?;
    let wasm_path = copy_fixture_module(temp_dir.path(), "wasm/server/entrypoint/main.wasm")?;
    let manifest = temp_dir.path().join("luau-rs.toml");
    write_project_manifest(&manifest)?;
    let output_path = temp_dir
        .path()
        .join("build/ServerScriptService/main.server.luau");

    // When the project is compiled, then the module is corrupted and compiled again.
    let first_compile = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["compile", "--manifest-path"])
        .arg(&manifest)
        .output()?;
    let first_success = first_compile.status.success();
    if !first_success {
        return Err(Error::other(format!(
            "initial project compile failed: success={first_success}, stderr={}",
            String::from_utf8_lossy(&first_compile.stderr)
        )));
    }
    let previous_output = fs_err::read(&output_path)?;
    fs_err::write(&wasm_path, b"not a wasm module")?;
    let second_compile = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["compile", "--manifest-path"])
        .arg(&manifest)
        .output()?;

    // Then the failed update explains the rejection and leaves the previous output unchanged.
    let second_success = second_compile.status.success();
    if second_success {
        return Err(Error::other(format!(
            "corrupt module unexpectedly compiled: success={second_success}"
        )));
    }
    let stderr = String::from_utf8_lossy(&second_compile.stderr);
    let mentions_rejection = predicate::str::contains("rejected").eval(&stderr);
    if !mentions_rejection {
        return Err(Error::other(format!(
            "failed update omitted rejection: mentions_rejection={mentions_rejection}, stderr={stderr}"
        )));
    }
    let current_output = fs_err::read(&output_path)?;
    if current_output == previous_output {
        Ok(())
    } else {
        Err(Error::other(format!(
            "failed update changed the published output: previous_bytes={}, current_bytes={}",
            previous_output.len(),
            current_output.len()
        )))
    }
}

#[rstest]
fn given_luau_rs_help_when_requested_then_stable_project_commands_are_listed() -> Result<(), Error>
{
    let command = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["--help"])
        .output()?;
    if !command.status.success() {
        return Err(Error::other(format!(
            "luau-rs --help failed: status={}",
            command.status
        )));
    }
    let stdout = String::from_utf8_lossy(&command.stdout);
    for expected in ["build", "check", "compile"] {
        if !stdout.contains(expected) {
            return Err(Error::other(format!(
                "luau-rs --help omitted {expected:?}: stdout={stdout}"
            )));
        }
    }
    for subcommand in ["check", "compile"] {
        let subcommand_help = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
            .args([subcommand, "--help"])
            .output()?;
        if !subcommand_help.status.success() {
            return Err(Error::other(format!(
                "luau-rs {subcommand} --help failed: status={}",
                subcommand_help.status
            )));
        }
        let subcommand_stdout = String::from_utf8_lossy(&subcommand_help.stdout);
        if !subcommand_stdout.contains("--manifest-path") {
            return Err(Error::other(format!(
                "luau-rs {subcommand} --help omitted --manifest-path: stdout={subcommand_stdout}"
            )));
        }
    }
    Ok(())
}
