//! Behaviour-driven coverage for decoder rejection boundaries.

use luau_rs::{decode_module, DecodeOutcome, WasmDecodeProblemReason};
use rstest::fixture;
use rstest_bdd::Slot;
use rstest_bdd_macros::{given, scenario, then, when, ScenarioState};
use std::io::{Error, ErrorKind};
use walrus::{ImportKind, Module};

#[derive(Default, ScenarioState)]
struct DecodeState {
    wasm_bytes: Slot<Vec<u8>>,
    outcome: Slot<DecodeOutcome>,
}

#[fixture]
fn state() -> DecodeState {
    DecodeState::default()
}

fn required_outcome(state: &DecodeState) -> Result<bool, Error> {
    state.outcome.is_filled().then_some(true).ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            "decoding did not produce an outcome before it was checked",
        )
    })
}

#[given("input bytes that are not a WebAssembly module")]
fn malformed_bytes(state: &DecodeState) {
    state.wasm_bytes.set(b"not wasm at all, sorry".to_vec());
}

#[given("a WebAssembly module that asks the host to provide memory")]
fn imported_memory(state: &DecodeState) {
    let mut module = Module::default();
    let memory = module.memories.add_local(false, false, 1, None, None);
    let _import_id = module.imports.add("env", "mem", ImportKind::Memory(memory));
    state.wasm_bytes.set(module.emit_wasm());
}

#[given("an empty WebAssembly module")]
fn empty_module(state: &DecodeState) {
    state.wasm_bytes.set(Module::default().emit_wasm());
}

#[when("I ask the compiler to read the WebAssembly bytes")]
fn decode_bytes(state: &DecodeState) -> Result<(), Error> {
    let outcome = state
        .wasm_bytes
        .with_ref(|wasm_bytes| decode_module(wasm_bytes))
        .ok_or_else(|| {
            Error::new(
                ErrorKind::NotFound,
                "WebAssembly bytes were not prepared before decoding",
            )
        })?;
    state.outcome.set(outcome);
    Ok(())
}

#[then("it identifies a malformed module")]
fn malformed_module_is_reported(state: &DecodeState) -> Result<(), Error> {
    let _outcome_exists = required_outcome(state)?;
    let malformed = state
        .outcome
        .with_ref(|outcome| match outcome {
            DecodeOutcome::Rejected(rejection) => rejection
                .problems()
                .iter()
                .any(|problem| matches!(problem, WasmDecodeProblemReason::MalformedModule(_))),
            DecodeOutcome::Decoded(_) => false,
        })
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "decoder outcome disappeared"))?;
    if malformed {
        Ok(())
    } else {
        Err(Error::other(format!(
            "malformed bytes were not classified as malformed: malformed={malformed}"
        )))
    }
}

#[then("it explains that imported memory is unsupported")]
fn memory_import_is_reported(state: &DecodeState) -> Result<(), Error> {
    let _outcome_exists = required_outcome(state)?;
    let unsupported_memory = state
        .outcome
        .with_ref(|outcome| match outcome {
            DecodeOutcome::Rejected(rejection) => rejection.problems().iter().any(|problem| {
                matches!(
                    problem,
                    WasmDecodeProblemReason::UnsupportedImportKind { kind, .. }
                        if *kind == "memory"
                )
            }),
            DecodeOutcome::Decoded(_) => false,
        })
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "decoder outcome disappeared"))?;
    if unsupported_memory {
        Ok(())
    } else {
        Err(Error::other(format!(
            "memory import was not rejected by name: unsupported_memory={unsupported_memory}"
        )))
    }
}

#[then("it returns no functions and no exports")]
fn empty_surface_is_reported(state: &DecodeState) -> Result<(), Error> {
    let _outcome_exists = required_outcome(state)?;
    let empty_surface = state
        .outcome
        .with_ref(|outcome| match outcome {
            DecodeOutcome::Decoded(decoded) => {
                decoded.functions().is_empty() && decoded.exports().is_empty()
            }
            DecodeOutcome::Rejected(_) => false,
        })
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "decoder outcome disappeared"))?;
    if empty_surface {
        Ok(())
    } else {
        Err(Error::other(format!(
            "empty module did not produce an empty surface: empty_surface={empty_surface}"
        )))
    }
}

#[scenario(path = "tests/features/decode.feature")]
fn reject_malformed_bytes(_state: DecodeState) {}

#[scenario(path = "tests/features/decode.feature")]
fn reject_imported_memory(_state: DecodeState) {}

#[scenario(path = "tests/features/decode.feature")]
fn decode_empty_module(_state: DecodeState) {}
