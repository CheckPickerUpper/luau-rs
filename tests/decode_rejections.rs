//! Behavior scenarios for deciding which WebAssembly modules can become Luau.

use luau_rs::{decode_module, DecodeOutcome, WasmDecodeProblemReason};
use rstest::rstest;
use std::io::{Error, ErrorKind};
use walrus::{ImportKind, Module};

#[rstest]
fn given_random_bytes_when_decoded_then_malformed_module_is_reported() -> Result<(), Error> {
    // Given input bytes that are not a WebAssembly module.
    let input = b"not wasm at all, sorry";

    // When the compiler reads the WebAssembly bytes.
    let outcome = decode_module(input);

    // Then it identifies a malformed module.
    match outcome {
        DecodeOutcome::Rejected(rejection) => {
            let malformed = rejection
                .problems()
                .iter()
                .any(|problem| matches!(problem, WasmDecodeProblemReason::MalformedModule(_)));
            if malformed {
                Ok(())
            } else {
                Err(Error::other(format!(
                    "malformed input had the wrong rejection: malformed={malformed}, rejection={rejection:?}"
                )))
            }
        }
        DecodeOutcome::Decoded(_) => Err(Error::other(
            "input bytes unexpectedly decoded: decoded=true",
        )),
    }
}

#[rstest]
fn given_imported_memory_when_decoded_then_unsupported_host_request_is_reported(
) -> Result<(), Error> {
    // Given a WebAssembly module that asks the host to provide memory.
    let mut module = Module::default();
    let memory = module.memories.add_local(false, false, 1, None, None);
    let _import_id = module.imports.add("env", "mem", ImportKind::Memory(memory));
    let input = module.emit_wasm();

    // When the compiler reads the WebAssembly bytes.
    let outcome = decode_module(&input);

    // Then it explains that imported memory is unsupported.
    match outcome {
        DecodeOutcome::Rejected(rejection) => {
            let unsupported_memory = rejection.problems().iter().any(|problem| {
                matches!(
                    problem,
                    WasmDecodeProblemReason::UnsupportedImportKind { kind, .. }
                        if *kind == "memory"
                )
            });
            if unsupported_memory {
                Ok(())
            } else {
                Err(Error::other(format!(
                    "memory import had the wrong rejection: unsupported_memory={unsupported_memory}, rejection={rejection:?}"
                )))
            }
        }
        DecodeOutcome::Decoded(_) => Err(Error::other(
            "memory import unexpectedly decoded: decoded=true",
        )),
    }
}

#[rstest]
fn given_empty_module_when_decoded_then_public_surface_is_empty() -> Result<(), Error> {
    // Given an empty WebAssembly module.
    let input = Module::default().emit_wasm();

    // When the compiler reads the WebAssembly bytes.
    let outcome = decode_module(&input);

    // Then it returns no functions and no exports.
    match outcome {
        DecodeOutcome::Decoded(decoded) => {
            let functions_empty = decoded.functions().is_empty();
            let exports_empty = decoded.exports().is_empty();
            if functions_empty && exports_empty {
                Ok(())
            } else {
                Err(Error::other(format!(
                    "empty module gained a public surface: functions_empty={functions_empty}, exports_empty={exports_empty}"
                )))
            }
        }
        DecodeOutcome::Rejected(rejection) => Err(Error::new(
            ErrorKind::InvalidData,
            format!("empty module was rejected: rejection={rejection:?}"),
        )),
    }
}
