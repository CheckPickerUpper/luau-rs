//! End-to-end round trip: the committed Rust fixture (compiled to wasm32)
//! decodes, translates, and passes the official Luau tools.

mod support;

use assert_cmd::Command;
use luau_rs::{
    decode_module, translate_module, DecodeOutcome, MainInvocation, TranslateOptions,
    TranslateOutcome,
};
use support::official_luau_tool;

/// Reads the committed wasm fixture built from `fixtures/rust-hello`.
fn fixture_wasm_bytes() -> Vec<u8> {
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/rust-hello/rust_hello.wasm");
    match fs_err::read(&fixture_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            assert!(
                false,
                "could not read fixture {}: {error}",
                fixture_path.display()
            );
            Vec::new()
        }
    }
}

/// Decodes and translates the fixture, returning the generated Luau text.
fn generate_fixture_luau() -> String {
    let wasm_bytes = fixture_wasm_bytes();
    let decoded = match decode_module(&wasm_bytes) {
        DecodeOutcome::Decoded(decoded) => decoded,
        DecodeOutcome::Rejected(rejection) => {
            assert!(false, "fixture rejected: {rejection:?}");
            return String::new();
        }
    };
    let options = TranslateOptions::with_main_invocation(MainInvocation::LeaveIdle);
    match translate_module(&decoded, options) {
        TranslateOutcome::Translated(artifact) => artifact.into_text(),
        TranslateOutcome::Rejected(rejection) => {
            assert!(false, "fixture translation rejected: {rejection:?}");
            String::new()
        }
    }
}

/// The generated module must name the exported surface it declares.
#[test]
fn generated_luau_declares_expected_exports() {
    let generated = generate_fixture_luau();
    assert!(
        generated.contains("add"),
        "generated Luau lacks the add export"
    );
    assert!(
        generated.contains("fib"),
        "generated Luau lacks the fib export"
    );
    assert!(
        generated.contains("double_at"),
        "generated Luau lacks the double_at export"
    );
    assert!(generated.contains("MEMORY"), "generated Luau lacks memory");
}

/// The generated output must be byte-for-byte stable (guards against drift).
#[test]
fn generated_luau_matches_snapshot() {
    let generated = generate_fixture_luau();
    insta::assert_snapshot!("fixture_generated_luau", generated);
}

/// The official Luau tools must accept the generated module.
#[test]
fn official_luau_analyze_accepts_generated_fixture() -> std::result::Result<(), std::io::Error> {
    let generated = generate_fixture_luau();
    let luau_analyze_path = official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"));

    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-analyze")
        .tempdir()?;
    let source_path = temp_dir.path().join("fixture.luau");
    fs_err::write(&source_path, &generated)?;

    Command::new(luau_analyze_path)
        .arg(&source_path)
        .assert()
        .success();
    Ok(())
}

/// The official Luau runtime must execute the generated exports correctly.
#[test]
fn official_luau_executes_fixture_with_expected_results() -> std::result::Result<(), std::io::Error>
{
    let generated = generate_fixture_luau();
    let luau_path = official_luau_tool(("LUAU_BIN", "luau"));

    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-execute")
        .tempdir()?;
    let source_path = temp_dir.path().join("driver.luau");
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

    Command::new(luau_path).arg(&source_path).assert().success();
    Ok(())
}
