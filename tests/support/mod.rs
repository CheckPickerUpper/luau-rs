//! Shared test helpers: official Luau tool resolution and wasm builders.

use std::path::PathBuf;
use std::process::Output;

/// Resolves one official Luau tool, failing the suite when it is absent.
///
/// The integration suite deliberately fails when the official Luau tools are
/// not available. From a fresh checkout, run:
/// `python scripts/build_pinned_luau.py`
pub fn official_luau_tool(tool_name: (&str, &str)) -> PathBuf {
    let (environment_name, binary_name) = tool_name;
    if let Some(path) = std::env::var_os(environment_name) {
        return PathBuf::from(path);
    }
    let checked_out = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("references/checkouts/luau");
    let candidate = checked_out.join(binary_name);
    if candidate.exists() {
        return candidate;
    }
    assert!(
        false,
        "official Luau tool {binary_name} not found; run `python scripts/build_pinned_luau.py` or set {environment_name}"
    );
    PathBuf::new()
}

/// Runs one official Luau executable against one source file.
pub fn run_official_luau_tool(tool_and_source: (&PathBuf, &PathBuf)) -> Output {
    let (tool_path, source_path) = tool_and_source;
    match std::process::Command::new(tool_path)
        .arg(source_path)
        .output()
    {
        Ok(tool_output) => tool_output,
        Err(execution_error) => {
            assert!(
                false,
                "could not execute {}: {execution_error}",
                tool_path.display()
            );
            std::process::exit(1)
        }
    }
}

/// Creates a temporary directory for generated-file tests.
pub fn temporary_directory(prefix: &str) -> tempfile::TempDir {
    match tempfile::Builder::new().prefix(prefix).tempdir() {
        Ok(directory) => directory,
        Err(error) => {
            assert!(false, "could not create temporary directory: {error}");
            std::process::exit(1)
        }
    }
}
