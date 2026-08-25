//! CLI tests: `luau-rs build` compiles the fixture into a Roblox layout.

use predicates::prelude::*;

/// The fixture wasm committed from `fixtures/rust-hello`.
fn fixture_wasm_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/rust-hello/rust_hello.wasm")
}

/// `luau-rs build` writes the entrypoint script to the Roblox layout path.

#[test]
fn build_writes_entrypoint_script() -> std::result::Result<(), std::io::Error> {
    let temp_dir = tempfile::Builder::new().prefix("luau-rs-cli").tempdir()?;
    let output_root = temp_dir.path().join("build");

    assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["build"])
        .arg(fixture_wasm_path())
        .args(["--out"])
        .arg(&output_root)
        .args(["--entrypoint"])
        .assert()
        .success();

    let artifact = output_root.join("ServerScriptService/main.server.luau");
    assert!(
        artifact.exists(),
        "entrypoint artifact missing at {}",
        artifact.display()
    );
    let text = fs_err::read_to_string(&artifact)?;
    assert!(
        text.starts_with("--!strict"),
        "artifact must be strict Luau"
    );
    assert!(
        text.contains("instantiate"),
        "artifact must export a factory"
    );
    Ok(())
}

/// `luau-rs build` rejects a non-wasm file with a typed error.
#[test]
fn build_rejects_garbage_input() -> std::result::Result<(), std::io::Error> {
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-cli-garbage")
        .tempdir()?;
    let garbage_path = temp_dir.path().join("garbage.wasm");
    fs_err::write(&garbage_path, b"definitely not wasm")?;
    let output_root = temp_dir.path().join("build");

    assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["build"])
        .arg(&garbage_path)
        .args(["--out"])
        .arg(&output_root)
        .args(["--entrypoint"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("rejected"));
    Ok(())
}

/// `luau-rs build` with `--side client` places the script under
/// `StarterPlayerScripts`.
#[test]
fn build_client_side_uses_starter_player_path() -> std::result::Result<(), std::io::Error> {
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-cli-client")
        .tempdir()?;
    let output_root = temp_dir.path().join("build");

    assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
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
        .assert()
        .success();

    let artifact = output_root.join("StarterPlayer/StarterPlayerScripts/game/main.client.luau");
    assert!(
        artifact.exists(),
        "client artifact missing at {}",
        artifact.display()
    );
    Ok(())
}
