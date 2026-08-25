//! Behaviour-driven round-trip coverage from the committed Rust fixture to Luau.

mod support;

use assert_cmd::Command;
use luau_rs::{
    decode_module, translate_module, DecodeOutcome, MainInvocation, TranslateOptions,
    TranslateOutcome,
};
use rstest::fixture;
use rstest_bdd::Slot;
use rstest_bdd_macros::{given, scenario, then, when, ScenarioState};
use std::io::{Error, ErrorKind};
use std::process::Output;
use support::official_luau_tool;
use tempfile::TempDir;

#[derive(Default, ScenarioState)]
struct RoundTripState {
    wasm_bytes: Slot<Vec<u8>>,
    generated: Slot<String>,
    result: Slot<Output>,
    root: Slot<TempDir>,
}

#[fixture]
fn state() -> RoundTripState {
    RoundTripState::default()
}

fn fixture_wasm_bytes() -> Result<Vec<u8>, Error> {
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/rust-hello/rust_hello.wasm");
    fs_err::read(&fixture_path).map_err(|error| {
        Error::new(
            error.kind(),
            format!("could not read fixture {}: {error}", fixture_path.display()),
        )
    })
}

fn generate_fixture_luau(wasm_bytes: &[u8]) -> Result<String, Error> {
    let decoded = match decode_module(wasm_bytes) {
        DecodeOutcome::Decoded(decoded) => decoded,
        DecodeOutcome::Rejected(rejection) => {
            return Err(Error::other(format!("fixture was rejected: {rejection:?}")))
        }
    };
    let options = TranslateOptions::with_main_invocation(MainInvocation::LeaveIdle);
    match translate_module(&decoded, options) {
        TranslateOutcome::Translated(artifact) => Ok(artifact.into_text()),
        TranslateOutcome::Rejected(rejection) => Err(Error::other(format!(
            "fixture translation was rejected: {rejection:?}"
        ))),
    }
}

fn required_generated(state: &RoundTripState) -> Result<String, Error> {
    state.generated.get().ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            "the fixture was not translated before its Luau was checked",
        )
    })
}

#[given("the compiled Rust hello module")]
fn committed_fixture(state: &RoundTripState) -> Result<(), Error> {
    state.wasm_bytes.set(fixture_wasm_bytes()?);
    Ok(())
}

#[when("I translate it to Luau")]
fn translate_fixture(state: &RoundTripState) -> Result<(), Error> {
    let wasm_bytes = state.wasm_bytes.get().ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            "the WebAssembly fixture was not prepared before translation",
        )
    })?;
    state.generated.set(generate_fixture_luau(&wasm_bytes)?);
    Ok(())
}

#[then("callers can use add, fib, and double_at")]
fn generated_exports_are_present(state: &RoundTripState) -> Result<(), Error> {
    let generated = required_generated(state)?;
    let has_exports =
        generated.contains("add") && generated.contains("fib") && generated.contains("double_at");
    if has_exports {
        Ok(())
    } else {
        Err(Error::other(format!(
            "generated Luau did not expose all fixture functions: has_exports={has_exports}"
        )))
    }
}

#[then("the translated module owns linear memory")]
fn generated_memory_is_present(state: &RoundTripState) -> Result<(), Error> {
    let generated = required_generated(state)?;
    let has_memory = generated.contains("MEMORY");
    if has_memory {
        Ok(())
    } else {
        Err(Error::other(format!(
            "generated Luau did not declare linear memory: has_memory={has_memory}"
        )))
    }
}

#[then("the generated Luau matches the committed output snapshot")]
fn generated_snapshot_matches(state: &RoundTripState) -> Result<(), Error> {
    let generated = required_generated(state)?;
    insta::assert_snapshot!("fixture_generated_luau", generated);
    Ok(())
}

#[when("Luau analysis checks the translated module")]
fn analyze_generated_luau(state: &RoundTripState) -> Result<(), Error> {
    let generated = required_generated(state)?;
    let analyzer = official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"))?;
    let root = tempfile::Builder::new()
        .prefix("luau-rs-round-trip-bdd-analyze")
        .tempdir()?;
    let source_path = root.path().join("fixture.luau");
    fs_err::write(&source_path, generated)?;
    let result = Command::new(analyzer).arg(&source_path).output()?;
    state.result.set(result);
    state.root.set(root);
    Ok(())
}

#[when("I run the translated module with official Luau")]
fn run_generated_luau(state: &RoundTripState) -> Result<(), Error> {
    let generated = required_generated(state)?;
    let luau = official_luau_tool(("LUAU_BIN", "luau"))?;
    let root = tempfile::Builder::new()
        .prefix("luau-rs-round-trip-bdd-run")
        .tempdir()?;
    let source_path = root.path().join("driver.luau");
    let driver = format!(
        "local function make()\n{generated}\nend\n\
         local m = make()({{}})\n\
         assert(m.add(20, 22) == 42, \"add mismatch\")\n\
         assert(m.fib(0) == 0, \"fib(0) mismatch\")\n\
         assert(m.fib(1) == 1, \"fib(1) mismatch\")\n\
         assert(m.fib(2) == 1, \"fib(2) mismatch\")\n\
         assert(m.fib(9) == 34, \"fib(9) mismatch\")\n\
         local mem = m.memory\n\
         buffer.writei32(mem, 0, 7)\n\
         m.double_at(0)\n\
         assert(buffer.readi32(mem, 0) == 14, \"double_at mismatch\")\n"
    );
    fs_err::write(&source_path, &driver)?;
    let result = Command::new(luau).arg(&source_path).output()?;
    state.result.set(result);
    state.root.set(root);
    Ok(())
}

fn command_success(state: &RoundTripState) -> Result<bool, Error> {
    state
        .result
        .with_ref(|output| output.status.success())
        .ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                "the official Luau command did not run before its result was checked",
            )
        })
}

fn command_failure(state: &RoundTripState) -> Result<Error, Error> {
    let stderr = state
        .result
        .with_ref(|output| String::from_utf8_lossy(&output.stderr).into_owned())
        .ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                "the official Luau result disappeared before its failure was reported",
            )
        })?;
    Ok(Error::other(format!(
        "official Luau command failed: stderr={stderr}"
    )))
}

#[then("the generated module passes analysis without errors")]
fn analyzer_accepts_generated_module(state: &RoundTripState) -> Result<(), Error> {
    let success = command_success(state)?;
    if success {
        Ok(())
    } else {
        Err(command_failure(state)?)
    }
}

#[then("add returns 42, fib returns 34, and memory doubles 7 to 14")]
fn exported_results_are_correct(state: &RoundTripState) -> Result<(), Error> {
    let success = command_success(state)?;
    if success {
        Ok(())
    } else {
        Err(command_failure(state)?)
    }
}

#[scenario(path = "tests/features/round_trip.feature")]
fn expose_generated_fixture_surface(_state: RoundTripState) {}

#[scenario(path = "tests/features/round_trip.feature")]
fn keep_generated_fixture_snapshot(_state: RoundTripState) {}

#[scenario(path = "tests/features/round_trip.feature")]
fn analyze_generated_fixture(_state: RoundTripState) {}

#[scenario(path = "tests/features/round_trip.feature")]
fn execute_generated_fixture(_state: RoundTripState) {}
