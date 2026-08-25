//! Shared test helpers: official Luau tool resolution.

use std::path::PathBuf;

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
