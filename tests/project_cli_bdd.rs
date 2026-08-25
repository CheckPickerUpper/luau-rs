//! Behaviour-driven coverage for manifest-backed project compilation.

use predicates::prelude::*;
use rstest::fixture;
use rstest_bdd::Slot;
use rstest_bdd_macros::{given, scenario, then, when, ScenarioState};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[derive(Default, ScenarioState)]
struct ProjectState {
    root: Slot<TempDir>,
    manifest: Slot<PathBuf>,
}

#[fixture]
fn state() -> ProjectState {
    ProjectState::default()
}

fn fixture_wasm_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/rust-hello/rust_hello.wasm")
}

fn stage_fixture_module(root: &Path, relative_path: &str) -> std::result::Result<(), Error> {
    let destination = root.join(relative_path);
    if let Some(parent_directory) = destination.parent() {
        fs_err::create_dir_all(parent_directory)?;
    }
    fs_err::copy(fixture_wasm_path(), destination)?;
    Ok(())
}

#[given("a manifest project with nested wasm modules")]
fn manifest_project(state: &ProjectState) -> std::result::Result<(), Error> {
    let root = tempfile::Builder::new()
        .prefix("luau-rs-project-bdd")
        .tempdir()?;
    stage_fixture_module(root.path(), "wasm/shared/library/math/core.wasm")?;
    stage_fixture_module(root.path(), "wasm/server/entrypoint/game/main.wasm")?;
    let manifest = root.path().join("luau-rs.toml");
    fs_err::write(
        &manifest,
        "[project]\nsource_root = \"wasm\"\noutput_root = \"build\"\n",
    )?;
    state.manifest.set(manifest);
    state.root.set(root);
    Ok(())
}

#[when("I compile the manifest project")]
fn compile_manifest_project(state: &ProjectState) -> std::result::Result<(), Error> {
    let manifest = state.manifest.get().ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            "manifest project was not created before compilation",
        )
    })?;
    assert_cmd::cargo::cargo_bin_cmd!("luau-rs")
        .args(["compile", "--manifest-path"])
        .arg(manifest)
        .assert()
        .success();
    Ok(())
}

fn published_path(
    state: &ProjectState,
    relative_path: &str,
) -> std::result::Result<PathBuf, Error> {
    state
        .root
        .with_ref(|root| root.path().join(relative_path))
        .ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                "manifest project was not created before publication",
            )
        })
}

#[then("the shared module is published under ReplicatedStorage")]
fn shared_module_is_published(state: &ProjectState) -> std::result::Result<(), Error> {
    let path = published_path(state, "build/ReplicatedStorage/math/core.luau")?;
    if predicate::path::exists().eval(&path) {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!("shared module was not published at {}", path.display()),
        ))
    }
}

#[then("the server entrypoint is published under ServerScriptService")]
fn server_entrypoint_is_published(state: &ProjectState) -> std::result::Result<(), Error> {
    let path = published_path(state, "build/ServerScriptService/game/main.server.luau")?;
    if predicate::path::exists().eval(&path) {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            format!("server entrypoint was not published at {}", path.display()),
        ))
    }
}

#[scenario(path = "tests/features/project_cli.feature")]
fn compile_nested_project(_state: ProjectState) {}
