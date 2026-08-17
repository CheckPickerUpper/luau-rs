//! Public compiler coverage for zero-based homogeneous arrays.

use luau_rs::{compile_source, CompilationOutcome, CompilationProblemReason};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn typed_arrays_lower_to_strict_one_based_luau_and_reject_invalid_access() {
    let source = "fn pick(values: [number], index: number) -> number {\n    return values[index];\n}\n\nfn main() {\n    let mut values: [number] = [20, 0];\n    values[1] = 42;\n    print(pick(values, 1));\n}\n";
    let generated = compiled_text(source);
    assert_eq!(generated, "--!strict\n\nlocal function pick(values: {number}, index: number): number\n    return values[(index) + 1]\nend\n\nlocal function main(): ()\n    local values: {number} = {20, 0}\n    values[(1) + 1] = 42\n    print(pick(values, 1))\nend\n\nmain()\n");
    match full_moon::parse(&generated) {
        Ok(_) => {}
        Err(parse_errors) => assert!(false, "Full Moon rejected arrays: {parse_errors:?}"),
    }
    assert_rejection((
        "fn main() { let values: [number] = [1, true]; }",
        "true",
        CompilationProblemReason::TypesDoNotMatch,
    ));
    assert_rejection((
        "fn main() { let value: number = 1; print(value[0]); }",
        "value",
        CompilationProblemReason::TypesDoNotMatch,
    ));
    assert_rejection((
        "fn main() { let values: [number] = [1]; print(values[true]); }",
        "true",
        CompilationProblemReason::TypesDoNotMatch,
    ));
    assert_rejection((
        "fn main() { let values: [number] = [1]; values[0] = 2; }",
        "values",
        CompilationProblemReason::ImmutableBindingCannotBeAssigned,
    ));
    assert_rejection((
        "fn main() { let mut values: [number] = [1]; values[0] = true; }",
        "true",
        CompilationProblemReason::TypesDoNotMatch,
    ));
    assert_official_tools_execute(&generated);
}

fn assert_official_tools_execute(generated: &str) {
    let path = std::env::temp_dir().join(format!(
        "roblox-rust-typed-arrays-{}.luau",
        std::process::id()
    ));
    assert!(std::fs::write(&path, generated).is_ok());
    let luau = official_tool(("LUAU_BIN", "luau"));
    let analyzer = official_tool(("LUAU_ANALYZE_BIN", "luau-analyze"));
    let runtime = match Command::new(luau).arg(&path).output() {
        Ok(output) => output,
        Err(error) => {
            assert!(false, "official Luau must run: {error}");
            return;
        }
    };
    assert!(
        runtime.status.success(),
        "{}",
        String::from_utf8_lossy(&runtime.stderr)
    );
    assert_eq!(
        runtime.stdout,
        if cfg!(windows) {
            b"42\r\n".as_slice()
        } else {
            b"42\n".as_slice()
        }
    );
    let analysis = match Command::new(analyzer).arg(&path).output() {
        Ok(output) => output,
        Err(error) => {
            assert!(false, "official analyzer must run: {error}");
            return;
        }
    };
    assert!(
        analysis.status.success(),
        "{}",
        String::from_utf8_lossy(&analysis.stderr)
    );
    assert!(std::fs::remove_file(path).is_ok());
}

fn official_tool(parts: (&str, &str)) -> PathBuf {
    let (environment_variable, executable_name) = parts;
    std::env::var_os(environment_variable).map_or_else(
        || {
            let executable = if cfg!(windows) {
                format!("{executable_name}.exe")
            } else {
                executable_name.to_owned()
            };
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("references")
                .join("checkouts")
                .join("luau")
                .join("build")
                .join(executable)
        },
        PathBuf::from,
    )
}

fn compiled_text(source: &str) -> String {
    match compile_source(source) {
        CompilationOutcome::Compiled(generated) => generated.into_text(),
        CompilationOutcome::Rejected(rejection) => {
            let problem = rejection.first_problem();
            assert!(
                false,
                "compiler rejected array fixture with {:?} at {}",
                problem.reason(),
                problem.source_range().start_byte()
            );
            String::new()
        }
    }
}

fn assert_rejection(parts: (&str, &str, CompilationProblemReason)) {
    let (source, spelling, expected_reason) = parts;
    let rejection = match compile_source(source) {
        CompilationOutcome::Rejected(rejection) => rejection,
        CompilationOutcome::Compiled(generated) => {
            assert!(
                false,
                "expected rejection, generated: {}",
                generated.into_text()
            );
            return;
        }
    };
    let problem = rejection.first_problem();
    assert_eq!(problem.reason(), &expected_reason);
    let start = source.rfind(spelling).map_or(0, |offset| offset);
    assert_eq!(problem.source_range().start_byte(), start);
    assert_eq!(problem.source_range().end_byte(), start + spelling.len());
}
