//! Behaviour-driven CLI coverage for building individual WebAssembly modules.

use predicates::prelude::*;
use rstest::fixture;
use rstest_bdd::Slot;
use rstest_bdd_macros::{given, scenario, then, when, ScenarioState};
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::process::Output;
use tempfile::TempDir;

#[derive(Default, ScenarioState)]
struct BuildState {
    root: Slot<TempDir>,
    input: Slot<PathBuf>,
    output: Slot<PathBuf>,
    command: Slot<Output>,
}

#[fixture]
fn state() -> BuildState {
    BuildState::default()
}

fn fixture_wasm_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/rust-hello/rust_hello.wasm")
}

fn required_input(state: &BuildState) -> Result<PathBuf, Error> {
    state.input.get().ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            "the build input was not prepared before the build step",
        )
    })
}

fn required_output(state: &BuildState) -> Result<PathBuf, Error> {
    state.output.get().ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            "the build output was not prepared before the build step",
        )
    })
}

fn command_succeeded(state: &BuildState) -> Result<bool, Error> {
    state
        .command
        .with_ref(|output| output.status.success())
        .ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                "the build command did not run before its result was checked",
            )
        })
}

fn command_stderr(state: &BuildState) -> Result<String, Error> {
    state
        .command
        .with_ref(|output| String::from_utf8_lossy(&output.stderr).into_owned())
        .ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                "the build command did not run before its error was checked",
            )
        })
}

#[given("a valid Rust module compiled to WebAssembly")]
fn committed_module(state: &BuildState) -> Result<(), Error> {
    let root = tempfile::Builder::new()
        .prefix("luau-rs-cli-bdd")
        .tempdir()?;
    state.input.set(fixture_wasm_path());
    state.output.set(root.path().join("build"));
    state.root.set(root);
    Ok(())
}

#[given("a file that claims to be WebAssembly but contains invalid bytes")]
fn invalid_module(state: &BuildState) -> Result<(), Error> {
    let root = tempfile::Builder::new()
        .prefix("luau-rs-cli-bdd-invalid")
        .tempdir()?;
    let input = root.path().join("invalid.wasm");
    fs_err::write(&input, b"definitely not wasm")?;
    state.input.set(input);
    state.output.set(root.path().join("build"));
    state.root.set(root);
    Ok(())
}

#[when("I build it as a server entrypoint")]
fn build_server(state: &BuildState) -> Result<(), Error> {
    let input = required_input(state)?;
    let output = required_output(state)?;
    let command = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["build"])
        .arg(input)
        .args(["--out"])
        .arg(output)
        .args(["--entrypoint"])
        .output()?;
    state.command.set(command);
    Ok(())
}

#[when("I build it as a client module at \"game/main\"")]
fn build_client(state: &BuildState) -> Result<(), Error> {
    let input = required_input(state)?;
    let output = required_output(state)?;
    let command = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["build"])
        .arg(input)
        .args(["--out"])
        .arg(output)
        .args([
            "--entrypoint",
            "--side",
            "client",
            "--module-path",
            "game/main",
        ])
        .output()?;
    state.command.set(command);
    Ok(())
}

fn artifact_path(state: &BuildState, relative_path: &str) -> Result<PathBuf, Error> {
    state
        .output
        .with_ref(|output| output.join(relative_path))
        .ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                "the build output was not prepared before the artifact was checked",
            )
        })
}

#[then("Roblox receives a server script under ServerScriptService")]
fn server_script_exists(state: &BuildState) -> Result<(), Error> {
    let artifact = artifact_path(state, "ServerScriptService/main.server.luau")?;
    if predicate::path::exists().eval(&artifact) {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!("server script was not written at {}", artifact.display()),
        ))
    }
}

#[then("the server script is strict Luau")]
fn server_script_is_strict(state: &BuildState) -> Result<(), Error> {
    let artifact = artifact_path(state, "ServerScriptService/main.server.luau")?;
    let text = fs_err::read_to_string(&artifact)?;
    if text.starts_with("--!strict") {
        Ok(())
    } else {
        Err(Error::other(format!(
            "server script at {} did not start with --!strict",
            artifact.display()
        )))
    }
}

#[then("callers can instantiate the generated module")]
fn server_script_exposes_factory(state: &BuildState) -> Result<(), Error> {
    let artifact = artifact_path(state, "ServerScriptService/main.server.luau")?;
    let text = fs_err::read_to_string(&artifact)?;
    if text.contains("instantiate") {
        Ok(())
    } else {
        Err(Error::other(format!(
            "server script at {} did not expose instantiate",
            artifact.display()
        )))
    }
}

#[then("the build explains that the module was rejected")]
fn invalid_input_is_rejected(state: &BuildState) -> Result<(), Error> {
    let succeeded = command_succeeded(state)?;
    if succeeded {
        return Err(Error::other(format!(
            "invalid WebAssembly input unexpectedly built: command_succeeded={succeeded}"
        )));
    }
    let stderr = command_stderr(state)?;
    if predicate::str::contains("rejected").eval(&stderr) {
        Ok(())
    } else {
        Err(Error::other(format!(
            "build failure did not explain the rejection: {stderr}"
        )))
    }
}

#[then("Roblox receives the client script at StarterPlayerScripts/game/main")]
fn client_script_exists(state: &BuildState) -> Result<(), Error> {
    let artifact = artifact_path(
        state,
        "StarterPlayer/StarterPlayerScripts/game/main.client.luau",
    )?;
    if predicate::path::exists().eval(&artifact) {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!("client script was not written at {}", artifact.display()),
        ))
    }
}

#[scenario(path = "tests/features/cli_build.feature")]
fn build_server_entrypoint(_state: BuildState) {}

#[scenario(path = "tests/features/cli_build.feature")]
fn reject_invalid_build_input(_state: BuildState) {}

#[scenario(path = "tests/features/cli_build.feature")]
fn build_client_module(_state: BuildState) {}
