//! Shared integration-test adapters for the pinned official Luau toolchain.

use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

/// Resolves an official Luau executable from its explicit environment override or local checkout.
pub fn resolve_official_luau_tool(tool_name: (&str, &str)) -> Option<PathBuf> {
    let (environment_variable, executable_name) = tool_name;
    std::env::var_os(environment_variable).map_or_else(
        || {
            let executable_filename = if cfg!(windows) {
                format!("{executable_name}.exe")
            } else {
                executable_name.to_owned()
            };
            let checkout_root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("references")
                .join("checkouts")
                .join("luau");
            [
                checkout_root
                    .join("build")
                    .join("release")
                    .join(&executable_filename),
                checkout_root.join("build").join(&executable_filename),
            ]
            .into_iter()
            .find(|candidate| candidate.is_file())
        },
        |configured_path| Some(PathBuf::from(configured_path)),
    )
}

/// Returns a resolved official Luau executable or fails loudly when the oracle is unavailable.
pub fn official_luau_tool(tool_name: (&str, &str)) -> PathBuf {
    resolve_official_luau_tool(tool_name).unwrap_or_else(|| {
        fail_missing_official_luau_tools();
        PathBuf::new()
    })
}

/// Runs one official Luau executable against one source file.
pub fn run_official_luau_tool(tool_and_source: (&Path, &Path)) -> Option<Output> {
    let (tool_path, source_path) = tool_and_source;
    match Command::new(tool_path).arg(source_path).output() {
        Ok(tool_output) => Some(tool_output),
        Err(execution_error) => {
            assert!(
                false,
                "could not invoke official Luau tool {}: {execution_error}",
                tool_path.display()
            );
            None
        }
    }
}

/// Runs one official Luau executable and returns its completed process output.
pub fn run_official_luau_tool_required(tool_and_source: (&Path, &Path)) -> Output {
    run_official_luau_tool(tool_and_source)
        .unwrap_or_else(|| panic!("official Luau tool invocation failed"))
}

/// Creates an automatically cleaned-up `.luau` scratch file for one integration test.
pub fn temporary_luau_file(label: &str) -> tempfile::NamedTempFile {
    let safe_label = label.replace(['\\', '/'], "-");
    match tempfile::Builder::new()
        .prefix(&safe_label)
        .suffix(".luau")
        .tempfile()
    {
        Ok(file) => file,
        Err(error) => panic!("could not create Luau scratch file: {error}"),
    }
}

/// Keeps missing official tools as a deliberate test failure instead of a skipped test.
pub fn fail_missing_official_luau_tools() {
    assert!(
        false,
        "official Luau tools are required; set LUAU_BIN, LUAU_ANALYZE_BIN, and LUAU_COMPILE_BIN to executable paths or run `python scripts/build_pinned_luau.py`"
    );
}
