//! Decode-stage rejections: modules using unsupported features fail loudly
//! with a typed reason instead of translating into wrong Luau.

use luau_rs::{decode_module, DecodeOutcome, WasmDecodeProblemReason};
use walrus::{ImportKind, Module};

/// Random bytes are not a wasm module.
#[test]
fn malformed_bytes_are_rejected() {
    let outcome = decode_module(b"not wasm at all, sorry");
    match outcome {
        DecodeOutcome::Rejected(rejection) => {
            assert!(
                rejection
                    .problems()
                    .iter()
                    .any(|problem| matches!(problem, WasmDecodeProblemReason::MalformedModule(_))),
                "expected a malformed-module rejection, got {rejection:?}"
            );
        }
        DecodeOutcome::Decoded(_) => {
            assert!(false, "garbage bytes must not decode");
        }
    }
}

/// Memory imports have no Luau representation and must be rejected by name.
#[test]
fn imported_memory_is_rejected() {
    let mut module = Module::default();
    let memory = module.memories.add_local(false, false, 1, None, None);
    let import_id = module.imports.add("env", "mem", ImportKind::Memory(memory));
    let import = module.imports.get(import_id);
    assert_eq!(import.module, "env");
    let wasm_bytes = module.emit_wasm();

    match decode_module(&wasm_bytes) {
        DecodeOutcome::Rejected(rejection) => {
            assert!(
                rejection.problems().iter().any(|problem| {
                    matches!(
                        problem,
                        WasmDecodeProblemReason::UnsupportedImportKind { kind, .. }
                            if *kind == "memory"
                    )
                }),
                "expected an unsupported-import rejection, got {rejection:?}"
            );
        }
        DecodeOutcome::Decoded(_) => {
            assert!(false, "memory imports must not decode");
        }
    }
}

/// A module with no functions at all still decodes to a valid empty surface.
#[test]
fn empty_module_decodes() {
    let mut module = Module::default();
    let wasm_bytes = module.emit_wasm();
    match decode_module(&wasm_bytes) {
        DecodeOutcome::Decoded(decoded) => {
            assert!(decoded.functions().is_empty());
            assert!(decoded.exports().is_empty());
        }
        DecodeOutcome::Rejected(rejection) => {
            assert!(false, "empty module must decode, got {rejection:?}");
        }
    }
}
