//! Integration coverage for typed while-loop compilation and execution.

mod support;
use support::{official_luau_tool, run_official_luau_tool_required, temporary_luau_file};

use full_moon::ast::LuaVersion;
use luau_rs::{compile_source, CompilationOutcome, CompilationProblemReason};

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
    insta::assert_snapshot!(generated_luau);
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

    let generated_luau_path = temporary_luau_file("luau-rs-typed-while");
    match std::fs::write(generated_luau_path.path(), generated_luau) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write generated Luau fixture to {}: {write_error}",
                generated_luau_path.path().display()
            );
            return;
        }
    }

    let luau_path = official_luau_tool(("LUAU_BIN", "luau"));
    let luau_analyze_path = official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"));
    let runtime_output = run_official_luau_tool_required((&luau_path, generated_luau_path.path()));
    assert!(
        runtime_output.status.success(),
        "official luau rejected while-loop execution:\n{}",
        String::from_utf8_lossy(&runtime_output.stderr)
    );
    let expected_runtime_output: &[u8] = if cfg!(windows) { b"3\r\n" } else { b"3\n" };
    assert_eq!(runtime_output.stdout, expected_runtime_output);

    let analysis_output =
        run_official_luau_tool_required((&luau_analyze_path, generated_luau_path.path()));
    assert!(
        analysis_output.status.success(),
        "official luau-analyze rejected while-loop Luau:\n{}",
        String::from_utf8_lossy(&analysis_output.stderr)
    );
}
