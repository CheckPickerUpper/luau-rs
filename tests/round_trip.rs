//! Behavior scenarios for keeping Rust module behavior after translation to Luau.

mod support;

use assert_cmd::Command;
use luau_rs::{
    decode_module, translate_module, DecodeOutcome, MainInvocation, TranslateOptions,
    TranslateOutcome,
};
use rstest::rstest;
use std::io::Error;
use std::thread;
use support::official_luau_tool;

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
            return Err(Error::other(format!(
                "fixture was rejected: rejection={rejection:?}"
            )))
        }
    };
    let options = TranslateOptions::with_main_invocation(MainInvocation::LeaveIdle);
    match translate_module(&decoded, options) {
        TranslateOutcome::Translated(artifact) => Ok(artifact.into_text()),
        TranslateOutcome::Rejected(rejection) => Err(Error::other(format!(
            "fixture translation was rejected: rejection={rejection:?}"
        ))),
    }
}

#[rstest]
fn given_compiled_module_when_translated_then_exports_and_memory_survive() -> Result<(), Error> {
    // Given the compiled Rust hello module.
    let wasm_bytes = fixture_wasm_bytes()?;

    // When the module is translated to Luau.
    let generated = generate_fixture_luau(&wasm_bytes)?;

    // Then callers can use add, fib, and double_at, and the module owns linear memory.
    let has_exports =
        generated.contains("add") && generated.contains("fib") && generated.contains("double_at");
    if !has_exports {
        return Err(Error::other(format!(
            "translated module lost an export: has_exports={has_exports}"
        )));
    }
    let has_memory = generated.contains("MEMORY");
    if has_memory {
        Ok(())
    } else {
        Err(Error::other(format!(
            "translated module lost linear memory: has_memory={has_memory}"
        )))
    }
}

#[rstest]
fn given_compiled_module_when_translated_then_output_matches_snapshot() -> Result<(), Error> {
    // Given the compiled Rust hello module.
    let wasm_bytes = fixture_wasm_bytes()?;

    // When the module is translated to Luau.
    let generated = generate_fixture_luau(&wasm_bytes)?;

    // Then the generated Luau matches the committed output snapshot.
    insta::assert_snapshot!("fixture_generated_luau", generated);
    Ok(())
}

#[rstest]
fn given_translated_module_when_analyzed_then_luau_accepts_it() -> Result<(), Error> {
    // Given the compiled Rust hello module translated into Luau.
    let generated = generate_fixture_luau(&fixture_wasm_bytes()?)?;
    let analyzer = official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"))?;
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-round-trip-bdd-analyze")
        .tempdir()?;
    let source_path = temp_dir.path().join("fixture.luau");
    fs_err::write(&source_path, &generated)?;

    // When official Luau analysis checks the translated module.
    let result = Command::new(analyzer).arg(&source_path).output()?;

    // Then the generated module passes analysis without errors.
    let success = result.status.success();
    if success {
        Ok(())
    } else {
        Err(Error::other(format!(
            "translated Luau failed analysis: success={success}, stderr={}",
            String::from_utf8_lossy(&result.stderr)
        )))
    }
}

#[rstest]
fn given_translated_module_when_checked_by_full_moon_then_luau_parses_it() -> Result<(), Error> {
    // Given the compiled Rust hello module translated into Luau.
    let generated = generate_fixture_luau(&fixture_wasm_bytes()?)?;

    // When Full Moon parses the generated Luau using its Luau grammar.
    let parse_result = thread::Builder::new()
        .name("full-moon-contract".to_owned())
        .stack_size(32 * 1024 * 1024)
        .spawn(move || match full_moon::parse(&generated) {
            Ok(_) => Ok(()),
            Err(errors) => Err(format!("errors={errors:?}")),
        })
        .map_err(|error| Error::other(format!("could not start Full Moon parser: {error}")))?
        .join()
        .map_err(|_| Error::other("Full Moon parser thread panicked"))?;

    parse_result.map_err(|errors| {
        Error::other(format!("generated Luau failed Full Moon parsing: {errors}"))
    })
}

#[rstest]
fn given_translated_module_when_compiled_then_official_luau_accepts_it() -> Result<(), Error> {
    // Given the compiled Rust hello module translated into Luau.
    let generated = generate_fixture_luau(&fixture_wasm_bytes()?)?;
    let compiler = official_luau_tool(("LUAU_COMPILE_BIN", "luau-compile"))?;
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-round-trip-compile")
        .tempdir()?;
    let source_path = temp_dir.path().join("fixture.luau");
    fs_err::write(&source_path, &generated)?;

    // When the pinned official Luau compiler compiles the generated module.
    let result = Command::new(compiler)
        .arg("--text")
        .arg(&source_path)
        .output()?;

    // Then the generated module is accepted by the official compiler.
    let success = result.status.success();
    if success {
        Ok(())
    } else {
        Err(Error::other(format!(
            "generated Luau failed official compilation: success={success}, stderr={}",
            String::from_utf8_lossy(&result.stderr)
        )))
    }
}

#[rstest]
fn given_translated_module_when_executed_then_exports_and_memory_match() -> Result<(), Error> {
    // Given the compiled Rust hello module translated into Luau.
    let generated = generate_fixture_luau(&fixture_wasm_bytes()?)?;
    let luau = official_luau_tool(("LUAU_BIN", "luau"))?;
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-round-trip-bdd-run")
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

    // When official Luau runs the translated module.
    let result = Command::new(luau).arg(&source_path).output()?;

    // Then add returns 42, fib returns 34, and memory doubles 7 to 14.
    let success = result.status.success();
    if success {
        Ok(())
    } else {
        Err(Error::other(format!(
            "translated behavior failed: success={success}, stderr={}",
            String::from_utf8_lossy(&result.stderr)
        )))
    }
}
