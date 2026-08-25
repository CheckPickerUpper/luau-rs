//! Diagnostics contract tests for source-aware wasm rejection output.

use luau_rs::{decode_module, DecodeOutcome, WasmDecodeProblemReason};
use std::error::Error;
use std::path::PathBuf;

fn diagnostics_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("fixtures/rust-diagnostics/{name}"))
}

#[test]
fn given_debug_wasm_when_decoder_rejects_simd_then_source_line_is_reported(
) -> Result<(), Box<dyn Error>> {
    let wasm = fs_err::read(diagnostics_fixture("diagnostics.wasm"))?;
    let DecodeOutcome::Rejected(rejection) = decode_module(&wasm) else {
        panic!("SIMD fixture must be rejected");
    };
    assert!(rejection
        .problems()
        .contains(&WasmDecodeProblemReason::UnsupportedVectorType));

    let [diagnostic] = rejection.diagnostics().diagnostics() else {
        panic!("the SIMD rejection should have one structured diagnostic");
    };
    assert_eq!(diagnostic.code, "unsupported_vector_type");
    assert!(diagnostic
        .location
        .file
        .as_deref()
        .is_some_and(|file| file.ends_with("src/lib.rs")));
    assert_eq!(diagnostic.location.line, Some(15));
    assert!(diagnostic.location.source_available);
    assert!(diagnostic.location.function.is_some());
    Ok(())
}

#[test]
fn given_stripped_wasm_when_decoder_rejects_simd_then_function_offset_is_reported(
) -> Result<(), Box<dyn Error>> {
    let wasm = fs_err::read(diagnostics_fixture("diagnostics-stripped.wasm"))?;
    let DecodeOutcome::Rejected(rejection) = decode_module(&wasm) else {
        panic!("SIMD fixture must be rejected");
    };
    let [diagnostic] = rejection.diagnostics().diagnostics() else {
        panic!("the SIMD rejection should have one structured diagnostic");
    };
    assert_eq!(diagnostic.code, "unsupported_vector_type");
    assert_eq!(
        diagnostic.location.function.as_deref(),
        Some("unsupported_vector")
    );
    assert!(diagnostic.location.wasm_offset.is_some());
    assert!(!diagnostic.location.source_available);
    assert!(diagnostic
        .location
        .hint
        .as_deref()
        .is_some_and(|hint| hint.contains("debug information")));
    Ok(())
}

#[test]
fn given_no_metadata_when_rejection_is_constructed_then_fallback_is_actionable() {
    let rejection = luau_rs::WasmDecodeRejection::from_problems(vec![
        WasmDecodeProblemReason::UnsupportedInstruction {
            instruction: "v128.load".into(),
        },
    ]);
    let [diagnostic] = rejection.diagnostics().diagnostics() else {
        panic!("one structured diagnostic is expected");
    };
    assert!(!diagnostic.location.source_available);
    assert!(diagnostic
        .location
        .hint
        .as_deref()
        .is_some_and(|hint| hint.contains("debug information")));
}
