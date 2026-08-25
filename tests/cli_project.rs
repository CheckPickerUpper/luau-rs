//! Behaviour-driven CLI coverage for manifest-backed project workflows.

use predicates::prelude::*;
use rstest::fixture;
use rstest_bdd::Slot;
use rstest_bdd_macros::{given, scenario, then, when, ScenarioState};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Output;
use tempfile::TempDir;

#[derive(Default, ScenarioState)]
struct ProjectState {
    root: Slot<TempDir>,
    manifest: Slot<PathBuf>,
    server_module: Slot<PathBuf>,
    command: Slot<Output>,
    previous_output: Slot<Vec<u8>>,
}

#[fixture]
fn state() -> ProjectState {
    ProjectState::default()
}

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

fn required_root(state: &ProjectState) -> Result<PathBuf, Error> {
    state
        .root
        .with_ref(|root| root.path().to_path_buf())
        .ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                "the project root was not prepared before the project step",
            )
        })
}

fn required_manifest(state: &ProjectState) -> Result<PathBuf, Error> {
    state.manifest.get().ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            "the project manifest was not prepared before the project step",
        )
    })
}

fn required_server_module(state: &ProjectState) -> Result<PathBuf, Error> {
    state.server_module.get().ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            "the server module was not prepared before the project step",
        )
    })
}

fn command_succeeded(state: &ProjectState) -> Result<bool, Error> {
    state
        .command
        .with_ref(|output| output.status.success())
        .ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                "the project command did not run before its result was checked",
            )
        })
}

fn command_stderr(state: &ProjectState) -> Result<String, Error> {
    state
        .command
        .with_ref(|output| String::from_utf8_lossy(&output.stderr).into_owned())
        .ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                "the project command did not run before its error was checked",
            )
        })
}

#[given("a project with a valid server module")]
fn valid_project_with_server_module(state: &ProjectState) -> Result<(), Error> {
    let root = tempfile::Builder::new()
        .prefix("luau-rs-project-bdd-server")
        .tempdir()?;
    let server_module = copy_fixture_module(root.path(), "wasm/server/entrypoint/main.wasm")?;
    let manifest = root.path().join("luau-rs.toml");
    write_project_manifest(&manifest)?;
    state.server_module.set(server_module);
    state.manifest.set(manifest);
    state.root.set(root);
    Ok(())
}

#[given("a project with a shared library and a server entrypoint")]
fn valid_project_with_nested_modules(state: &ProjectState) -> Result<(), Error> {
    let root = tempfile::Builder::new()
        .prefix("luau-rs-project-bdd-nested")
        .tempdir()?;
    copy_fixture_module(root.path(), "wasm/shared/library/math/core.wasm")?;
    let server_module = copy_fixture_module(root.path(), "wasm/server/entrypoint/game/main.wasm")?;
    let manifest = root.path().join("luau-rs.toml");
    write_project_manifest(&manifest)?;
    state.server_module.set(server_module);
    state.manifest.set(manifest);
    state.root.set(root);
    Ok(())
}

#[given("a project manifest that omits where generated files should go")]
fn malformed_project(state: &ProjectState) -> Result<(), Error> {
    let root = tempfile::Builder::new()
        .prefix("luau-rs-project-bdd-invalid")
        .tempdir()?;
    let manifest = root.path().join("luau-rs.toml");
    fs_err::write(&manifest, "[project]\nsource_root = \"wasm\"\n")?;
    state.manifest.set(manifest);
    state.root.set(root);
    Ok(())
}

#[when("I check the project manifest")]
fn check_manifest_project(state: &ProjectState) -> Result<(), Error> {
    let manifest = required_manifest(state)?;
    let command = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["check", "--manifest-path"])
        .arg(manifest)
        .output()?;
    state.command.set(command);
    Ok(())
}

#[when("I compile the project")]
fn compile_manifest_project(state: &ProjectState) -> Result<(), Error> {
    let manifest = required_manifest(state)?;
    let command = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["compile", "--manifest-path"])
        .arg(manifest)
        .output()?;
    state.command.set(command);
    Ok(())
}

#[when("I compile it again after adding a file under the managed output")]
fn recompile_after_adding_stale_file(state: &ProjectState) -> Result<(), Error> {
    let root = required_root(state)?;
    let stale_path = root.join("build/stale-managed-file.txt");
    fs_err::write(&stale_path, "stale output")?;
    let manifest = required_manifest(state)?;
    let command = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["compile", "--manifest-path"])
        .arg(manifest)
        .output()?;
    state.command.set(command);
    Ok(())
}

#[when("I remember the published server output")]
fn remember_server_output(state: &ProjectState) -> Result<(), Error> {
    let root = required_root(state)?;
    let output_path = root.join("build/ServerScriptService/main.server.luau");
    state.previous_output.set(fs_err::read(output_path)?);
    Ok(())
}

#[when("I replace the module with bytes that are not WebAssembly and compile again")]
fn recompile_after_corrupting_server_module(state: &ProjectState) -> Result<(), Error> {
    let server_module = required_server_module(state)?;
    fs_err::write(server_module, b"not a wasm module")?;
    let manifest = required_manifest(state)?;
    let command = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["compile", "--manifest-path"])
        .arg(manifest)
        .output()?;
    state.command.set(command);
    Ok(())
}

#[then("project validation succeeds")]
fn checking_succeeds(state: &ProjectState) -> Result<(), Error> {
    let succeeded = command_succeeded(state)?;
    if succeeded {
        Ok(())
    } else {
        Err(Error::other(format!(
            "checking failed unexpectedly: command_succeeded={succeeded}, stderr={}",
            command_stderr(state)?
        )))
    }
}

#[then("no Roblox output is created")]
fn no_build_output_is_created(state: &ProjectState) -> Result<(), Error> {
    let root = required_root(state)?;
    let output_exists = root.join("build").exists();
    if output_exists {
        Err(Error::other(format!(
            "check created a build directory: output_exists={output_exists}"
        )))
    } else {
        Ok(())
    }
}

#[then("the shared library appears in ReplicatedStorage")]
fn shared_module_is_published(state: &ProjectState) -> Result<(), Error> {
    let root = required_root(state)?;
    let path = root.join("build/ReplicatedStorage/math/core.luau");
    let exists = predicate::path::exists().eval(&path);
    if exists {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "shared module was not published at {}: exists={exists}",
                path.display()
            ),
        ))
    }
}

#[then("the server entrypoint appears in ServerScriptService")]
fn server_entrypoint_is_published(state: &ProjectState) -> Result<(), Error> {
    let root = required_root(state)?;
    let path = root.join("build/ServerScriptService/game/main.server.luau");
    let exists = predicate::path::exists().eval(&path);
    if exists {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "server entrypoint was not published at {}: exists={exists}",
                path.display()
            ),
        ))
    }
}

#[then("the stale managed file disappears")]
fn stale_managed_file_is_removed(state: &ProjectState) -> Result<(), Error> {
    let root = required_root(state)?;
    let stale_path = root.join("build/stale-managed-file.txt");
    let exists = stale_path.exists();
    if exists {
        Err(Error::other(format!(
            "stale managed file survived compilation: exists={exists}"
        )))
    } else {
        Ok(())
    }
}

#[then("validation names the missing output location")]
fn missing_output_root_is_reported(state: &ProjectState) -> Result<(), Error> {
    let succeeded = command_succeeded(state)?;
    if succeeded {
        return Err(Error::other(format!(
            "malformed manifest unexpectedly passed: command_succeeded={succeeded}"
        )));
    }
    let stderr = command_stderr(state)?;
    if predicate::str::contains("output_root").eval(&stderr) {
        Ok(())
    } else {
        Err(Error::other(format!(
            "manifest error omitted output_root: stderr={stderr}"
        )))
    }
}

#[then("compilation explains that the module was rejected")]
fn corrupt_module_is_rejected(state: &ProjectState) -> Result<(), Error> {
    let succeeded = command_succeeded(state)?;
    if succeeded {
        return Err(Error::other(format!(
            "corrupt module unexpectedly compiled: command_succeeded={succeeded}"
        )));
    }
    let stderr = command_stderr(state)?;
    if predicate::str::contains("rejected").eval(&stderr) {
        Ok(())
    } else {
        Err(Error::other(format!(
            "corrupt module failure omitted rejection: stderr={stderr}"
        )))
    }
}

#[then("the last good server output is unchanged")]
fn previous_server_output_is_preserved(state: &ProjectState) -> Result<(), Error> {
    let root = required_root(state)?;
    let previous = state.previous_output.get().ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            "the previous server output was not remembered before it was compared",
        )
    })?;
    let current = fs_err::read(root.join("build/ServerScriptService/main.server.luau"))?;
    if current == previous {
        Ok(())
    } else {
        Err(Error::other(format!(
            "failed compilation changed the published output: previous_bytes={}, current_bytes={}",
            previous.len(),
            current.len()
        )))
    }
}

#[scenario(path = "tests/features/cli_project.feature")]
fn check_valid_project(_state: ProjectState) {}

#[scenario(path = "tests/features/cli_project.feature")]
fn compile_nested_project(_state: ProjectState) {}

#[scenario(path = "tests/features/cli_project.feature")]
fn report_missing_output_root(_state: ProjectState) {}

#[scenario(path = "tests/features/cli_project.feature")]
fn preserve_previous_project_output(_state: ProjectState) {}
