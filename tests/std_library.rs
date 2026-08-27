//! End-to-end behavior for a standard-library Rust crate.

mod support;

use luau_rs::{
    decode_module, translate_module, DecodeOutcome, MainInvocation, TranslateOptions,
    TranslateOutcome,
};
use rstest::rstest;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default)]
struct ConstantHasher;

impl Hasher for ConstantHasher {
    fn finish(&self) -> u64 {
        0
    }

    fn write(&mut self, _bytes: &[u8]) {}
}
use std::io::Error;
use std::path::PathBuf;
use std::process::Command;
use support::official_luau_tool;

fn fixture_wasm_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/rust-std/rust_std.wasm")
}

fn native_standard_library_score() -> i32 {
    let values = [3, 5, 7];
    let name = String::from("luau");
    let formatted = format!("{name}:{}", values.len());
    let key = formatted.len() as i32;
    let mut scores: HashMap<i32, i32, BuildHasherDefault<ConstantHasher>> =
        HashMap::with_capacity_and_hasher(1, BuildHasherDefault::default());
    scores.insert(key, 29);
    let lookup = scores.get(&key).copied().unwrap_or_default();
    lookup + values.iter().sum::<i32>()
}

fn generated_fixture_luau() -> Result<String, Error> {
    let wasm = fs_err::read(fixture_wasm_path())?;
    let decoded = match decode_module(&wasm) {
        DecodeOutcome::Decoded(decoded) => decoded,
        DecodeOutcome::Rejected(rejection) => {
            return Err(Error::other(format!(
                "standard-library fixture was rejected: {rejection:?}"
            )))
        }
    };
    let options = TranslateOptions::with_main_invocation(MainInvocation::LeaveIdle);
    match translate_module(&decoded, options) {
        TranslateOutcome::Translated(artifact) => Ok(artifact.into_text()),
        TranslateOutcome::Rejected(rejection) => Err(Error::other(format!(
            "standard-library translation was rejected: {rejection:?}"
        ))),
    }
}

#[rstest]
fn given_std_crate_when_run_in_official_luau_then_matches_native_output() -> Result<(), Error> {
    let generated = generated_fixture_luau()?;
    let expected = native_standard_library_score();
    let temp_dir = tempfile::Builder::new()
        .prefix("luau-rs-standard-library-bdd")
        .tempdir()?;
    let source_path = temp_dir.path().join("driver.luau");
    let module_path = temp_dir.path().join("module.luau");
    let driver = format!(
        "local function make()\n{generated}\nend\n\
         local m = make()({{}})\n\
         assert(m.standard_library_score() == {expected}, \"standard-library result mismatch\")\n",
    );
    fs_err::write(&source_path, &driver)?;
    fs_err::write(&module_path, &generated)?;

    let analyzer = official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"))?;
    let analysis = Command::new(analyzer).arg(&module_path).output()?;
    if !analysis.status.success() {
        return Err(Error::other(format!(
            "standard-library Luau analysis failed: stderr={}",
            String::from_utf8_lossy(&analysis.stderr)
        )));
    }

    let luau = official_luau_tool(("LUAU_BIN", "luau"))?;
    let execution = Command::new(luau).arg(&source_path).output()?;
    if execution.status.success() {
        Ok(())
    } else {
        Err(Error::other(format!(
            "standard-library Luau execution failed: stderr={}",
            String::from_utf8_lossy(&execution.stderr)
        )))
    }
}
