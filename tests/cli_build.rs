//! Behavior scenarios for building individual WebAssembly modules.

use predicates::prelude::*;
use rstest::rstest;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

fn fixture_wasm_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/rust-hello/rust_hello.wasm")
}

#[rstest]
fn a_valid_rust_module_becomes_a_strict_server_script() -> Result<(), Error> {
    // Given a valid Rust module compiled to WebAssembly and a temporary Roblox output root.
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-cli-bdd-server")
        .tempdir()?;
    let output_root = temp_dir.path().join("build");

    // When the module is built as a server entrypoint.
    let command = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["build"])
        .arg(fixture_wasm_path())
        .args(["--out"])
        .arg(&output_root)
        .args(["--entrypoint"])
        .output()?;

    // Then Roblox receives a strict server script with an instantiate factory.
    let success = command.status.success();
    if !success {
        return Err(Error::other(format!(
            "server build failed: success={success}, stderr={}",
            String::from_utf8_lossy(&command.stderr)
        )));
    }
    let artifact = output_root.join("ServerScriptService/main.server.luau");
    let exists = predicate::path::exists().eval(&artifact);
    if !exists {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "server script was not written: exists={exists}, path={}",
                artifact.display()
            ),
        ));
    }
    let text = fs_err::read_to_string(&artifact)?;
    let strict = text.starts_with("--!strict");
    if !strict {
        return Err(Error::other(format!(
            "server script is not strict Luau: strict={strict}"
        )));
    }
    let has_factory = text.contains("instantiate");
    if !has_factory {
        return Err(Error::other(format!(
            "server script has no instantiate factory: has_factory={has_factory}"
        )));
    }
    Ok(())
}

#[rstest]
fn corrupt_module_input_is_rejected_without_building_a_script() -> Result<(), Error> {
    // Given a file that claims to be WebAssembly but contains invalid bytes.
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-cli-bdd-invalid")
        .tempdir()?;
    let input = temp_dir.path().join("invalid.wasm");
    fs_err::write(&input, b"definitely not wasm")?;
    let output_root = temp_dir.path().join("build");

    // When the corrupt input is built as a server entrypoint.
    let command = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["build"])
        .arg(&input)
        .args(["--out"])
        .arg(&output_root)
        .args(["--entrypoint"])
        .output()?;

    // Then the build explains the rejection and does not report success.
    let success = command.status.success();
    if success {
        return Err(Error::other(format!(
            "corrupt input unexpectedly built: success={success}"
        )));
    }
    let stderr = String::from_utf8_lossy(&command.stderr);
    let mentions_rejection = predicate::str::contains("rejected").eval(&stderr);
    if mentions_rejection {
        Ok(())
    } else {
        Err(Error::other(format!(
            "corrupt-input failure did not explain rejection: mentions_rejection={mentions_rejection}, stderr={stderr}"
        )))
    }
}

#[rstest]
fn a_client_module_lands_in_its_roblox_client_container() -> Result<(), Error> {
    // Given a valid Rust module compiled to WebAssembly and a temporary Roblox output root.
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-cli-bdd-client")
        .tempdir()?;
    let output_root = temp_dir.path().join("build");

    // When the module is built as a client entrypoint at game/main.
    let command = assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["build"])
        .arg(fixture_wasm_path())
        .args(["--out"])
        .arg(&output_root)
        .args([
            "--entrypoint",
            "--side",
            "client",
            "--module-path",
            "game/main",
        ])
        .output()?;

    // Then Roblox receives the client script at StarterPlayerScripts/game/main.
    let success = command.status.success();
    if !success {
        return Err(Error::other(format!(
            "client build failed: success={success}, stderr={}",
            String::from_utf8_lossy(&command.stderr)
        )));
    }
    let artifact = output_root.join("StarterPlayer/StarterPlayerScripts/game/main.client.luau");
    let exists = predicate::path::exists().eval(&artifact);
    if exists {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!(
                "client script was not written: exists={exists}, path={}",
                artifact.display()
            ),
        ))
    }
}
