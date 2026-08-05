//! Integration coverage for typed while-loop compilation and execution.

use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use full_moon::ast::LuaVersion;
use roblox_rust::{compile_source, CompilationOutcome, CompilationProblemReason};

const NUMBER_CONDITION_START_BYTE: usize = 18;
const NUMBER_CONDITION_END_BYTE: usize = 19;
const RETURNING_LOOP_FUNCTION_NAME_START_BYTE: usize = 3;
const RETURNING_LOOP_FUNCTION_NAME_END_BYTE: usize = 13;

#[test]
fn typed_while_loops_enforce_conditions_scope_and_function_returns_then_execute_in_luau() {
    let non_boolean_condition_source = "fn main() { while 1 { print(3); } }\n";
    let non_boolean_condition_rejection = match compile_source(non_boolean_condition_source) {
        CompilationOutcome::Rejected(compilation_rejection) => compilation_rejection,
        CompilationOutcome::Compiled(generated_luau_text) => {
            assert!(
                false,
                "expected a non-boolean loop condition rejection, generated: {}",
                generated_luau_text.into_text()
            );
            return;
        }
    };
    let non_boolean_condition_problem = non_boolean_condition_rejection.first_problem();
    assert_eq!(
        non_boolean_condition_problem.reason(),
        &CompilationProblemReason::TypesDoNotMatch
    );
    assert_eq!(
        non_boolean_condition_problem.source_range().start_byte(),
        NUMBER_CONDITION_START_BYTE
    );
    assert_eq!(
        non_boolean_condition_problem.source_range().end_byte(),
        NUMBER_CONDITION_END_BYTE
    );

    let loop_local_scope_source = r"fn main() {
    while true {
        let loop_number: number = 3;
        print(loop_number);
    }
    print(loop_number);
}
";
    let loop_local_scope_rejection = match compile_source(loop_local_scope_source) {
        CompilationOutcome::Rejected(compilation_rejection) => compilation_rejection,
        CompilationOutcome::Compiled(generated_luau_text) => {
            assert!(
                false,
                "expected a loop-scope rejection, generated: {}",
                generated_luau_text.into_text()
            );
            return;
        }
    };
    assert_eq!(
        loop_local_scope_rejection.first_problem().reason(),
        &CompilationProblemReason::UnknownName
    );

    let returning_loop_source = r"fn loop_value() -> number {
    while true {
        return 3;
    }
}

fn main() {
    print(loop_value());
}
";
    let returning_loop_rejection = match compile_source(returning_loop_source) {
        CompilationOutcome::Rejected(compilation_rejection) => compilation_rejection,
        CompilationOutcome::Compiled(generated_luau_text) => {
            assert!(
                false,
                "expected a missing-return rejection, generated: {}",
                generated_luau_text.into_text()
            );
            return;
        }
    };
    let returning_loop_problem = returning_loop_rejection.first_problem();
    assert_eq!(
        returning_loop_problem.reason(),
        &CompilationProblemReason::MissingReturn
    );
    assert_eq!(
        returning_loop_problem.source_range().start_byte(),
        RETURNING_LOOP_FUNCTION_NAME_START_BYTE
    );
    assert_eq!(
        returning_loop_problem.source_range().end_byte(),
        RETURNING_LOOP_FUNCTION_NAME_END_BYTE
    );

    let executable_loop_source = r"fn main() {
    let mut count: number = 0;
    while count < 3 {
        count = count + 1;
    }
    print(count);
}
";
    let expected_luau = r"--!strict

local function main(): ()
    local count: number = 0
    while count < 3 do
        count = count + 1
    end
    print(count)
end

main()
";
    let generated_luau = match compile_source(executable_loop_source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected mutable loop fixture with {} problems",
                compilation_rejection.problem_count()
            );
            return;
        }
    };
    assert_eq!(generated_luau, expected_luau);
    match full_moon::parse_fallible(&generated_luau, LuaVersion::luau()).into_result() {
        Ok(_) => {}
        Err(parse_errors) => {
            assert!(
                false,
                "Full Moon rejected while-loop Luau: {parse_errors:?}"
            );
            return;
        }
    }

    let generated_luau_path = std::env::temp_dir().join(format!(
        "roblox-rust-typed-while-loops-{}.luau",
        std::process::id()
    ));
    match std::fs::write(&generated_luau_path, generated_luau) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write generated Luau fixture to {}: {write_error}",
                generated_luau_path.display()
            );
            return;
        }
    }

    let Some(luau_path) = resolve_official_luau_tool(("LUAU_BIN", "luau")) else {
        fail_missing_official_luau_tools();
        return;
    };
    let Some(luau_analyze_path) = resolve_official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"))
    else {
        fail_missing_official_luau_tools();
        return;
    };
    let Some(runtime_output) = run_official_luau_tool((&luau_path, &generated_luau_path)) else {
        return;
    };
    assert!(
        runtime_output.status.success(),
        "official luau rejected while-loop execution:\n{}",
        String::from_utf8_lossy(&runtime_output.stderr)
    );
    let expected_runtime_output: &[u8] = if cfg!(windows) { b"3\r\n" } else { b"3\n" };
    assert_eq!(runtime_output.stdout, expected_runtime_output);

    let Some(analysis_output) = run_official_luau_tool((&luau_analyze_path, &generated_luau_path))
    else {
        return;
    };
    assert!(
        analysis_output.status.success(),
        "official luau-analyze rejected while-loop Luau:\n{}",
        String::from_utf8_lossy(&analysis_output.stderr)
    );

    match std::fs::remove_file(&generated_luau_path) {
        Ok(()) => {}
        Err(remove_error) => assert!(
            false,
            "could not remove generated Luau fixture {}: {remove_error}",
            generated_luau_path.display()
        ),
    }
}

fn resolve_official_luau_tool(tool_name: (&str, &str)) -> Option<PathBuf> {
    let (environment_variable, executable_name) = tool_name;
    std::env::var_os(environment_variable).map_or_else(
        || {
            let executable_filename = if cfg!(windows) {
                format!("{executable_name}.exe")
            } else {
                executable_name.to_owned()
            };
            let checkout_build_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("references")
                .join("checkouts")
                .join("luau")
                .join("build")
                .join("release")
                .join(executable_filename);
            if checkout_build_path.is_file() {
                Some(checkout_build_path)
            } else {
                None
            }
        },
        |configured_path| Some(PathBuf::from(configured_path)),
    )
}

fn run_official_luau_tool(tool_and_source: (&Path, &Path)) -> Option<Output> {
    let (tool_path, generated_luau_path) = tool_and_source;
    match Command::new(tool_path).arg(generated_luau_path).output() {
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

fn fail_missing_official_luau_tools() {
    assert!(
        false,
        "official Luau tools are required; set LUAU_BIN and LUAU_ANALYZE_BIN to executable paths or build them in references/checkouts/luau"
    );
}
