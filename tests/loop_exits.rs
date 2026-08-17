//! Integration coverage for loop-exit parsing, checking, lowering, and Luau execution.

use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use full_moon::ast::LuaVersion;
use luau_rs::{compile_source, CompilationOutcome, CompilationProblemReason};

#[test]
fn loop_exits_are_scoped_checked_and_lowered_to_luau() {
    assert_outside_loop_exit_is_rejected("break");
    assert_outside_loop_exit_is_rejected("continue");

    let direct_exit_source = r"fn main() {
    while true {
        break;
        print(1);
    }
}
";
    assert_unreachable_statement_is_rejected(direct_exit_source, "print(1)");

    let branching_exit_source = r"fn main() {
    while true {
        if true {
            break;
        } else {
            continue;
        }
        print(1);
    }
}
";
    assert_unreachable_statement_is_rejected(branching_exit_source, "print(1)");

    let incomplete_return_source = r"fn loop_value() -> number {
    while true {
        break;
    }
}

fn main() {}
";
    let incomplete_return_rejection = rejected_compilation(incomplete_return_source);
    let incomplete_return_problem = incomplete_return_rejection.first_problem();
    assert_eq!(
        incomplete_return_problem.reason(),
        &CompilationProblemReason::MissingReturn
    );
    let Some(function_name_start) = incomplete_return_source.find("loop_value") else {
        assert!(false, "fixture must contain the value function name");
        return;
    };
    assert_eq!(
        incomplete_return_problem.source_range().start_byte(),
        function_name_start
    );
    assert_eq!(
        incomplete_return_problem.source_range().end_byte(),
        function_name_start + "loop_value".len()
    );

    let nested_loop_source = r"fn main() {
    let mut outer: number = 0;
    let mut inner: number = 0;
    while outer < 3 {
        outer = outer + 1;
        inner = 0;
        while true {
            inner = inner + 1;
            if inner == 2 {
                break;
            } else {
                continue;
            }
        }
    }
    print(outer);
    print(inner);
}
";
    let generated_luau = compiled_luau(nested_loop_source);
    assert!(generated_luau.contains("                break\n"));
    assert!(generated_luau.contains("                continue\n"));
    match full_moon::parse_fallible(&generated_luau, LuaVersion::luau()).into_result() {
        Ok(_) => {}
        Err(parse_errors) => {
            assert!(false, "Full Moon rejected loop-exit Luau: {parse_errors:?}");
            return;
        }
    }

    let generated_luau_path = std::env::temp_dir().join(format!(
        "roblox-rust-loop-exits-{}.luau",
        std::process::id()
    ));
    match std::fs::write(&generated_luau_path, generated_luau) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write generated loop-exit fixture to {}: {write_error}",
                generated_luau_path.display()
            );
            return;
        }
    }

    let luau_path = official_luau_tool(("LUAU_BIN", "luau"));
    let luau_analyze_path = official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"));
    let runtime_output = run_official_luau_tool((&luau_path, &generated_luau_path));
    assert!(
        runtime_output.status.success(),
        "official luau rejected loop-exit execution:\n{}",
        String::from_utf8_lossy(&runtime_output.stderr)
    );
    let expected_runtime_output: &[u8] = if cfg!(windows) {
        b"3\r\n2\r\n"
    } else {
        b"3\n2\n"
    };
    assert_eq!(runtime_output.stdout, expected_runtime_output);

    let analysis_output = run_official_luau_tool((&luau_analyze_path, &generated_luau_path));
    assert!(
        analysis_output.status.success(),
        "official luau-analyze rejected loop-exit Luau:\n{}",
        String::from_utf8_lossy(&analysis_output.stderr)
    );

    match std::fs::remove_file(&generated_luau_path) {
        Ok(()) => {}
        Err(remove_error) => assert!(
            false,
            "could not remove generated loop-exit fixture {}: {remove_error}",
            generated_luau_path.display()
        ),
    }
}

fn assert_outside_loop_exit_is_rejected(exit_keyword: &str) {
    let source = format!("fn main() {{\n    {exit_keyword};\n}}\n");
    let rejection = rejected_compilation(&source);
    let problem = rejection.first_problem();
    assert_eq!(
        problem.reason(),
        &CompilationProblemReason::SourceDoesNotFollowLanguageRules
    );
    let Some(keyword_start) = source.find(exit_keyword) else {
        assert!(false, "fixture must contain the loop-exit keyword");
        return;
    };
    assert_eq!(problem.source_range().start_byte(), keyword_start);
    assert_eq!(
        problem.source_range().end_byte(),
        keyword_start + exit_keyword.len()
    );
}

fn assert_unreachable_statement_is_rejected(source: &str, statement_name: &str) {
    let rejection = rejected_compilation(source);
    let problem = rejection.first_problem();
    assert_eq!(
        problem.reason(),
        &CompilationProblemReason::SourceDoesNotFollowLanguageRules
    );
    let Some(statement_start) = source.find(statement_name) else {
        assert!(false, "fixture must contain the unreachable statement");
        return;
    };
    assert_eq!(problem.source_range().start_byte(), statement_start);
    assert_eq!(
        problem.source_range().end_byte(),
        statement_start + statement_name.len()
    );
}

fn rejected_compilation(source: &str) -> luau_rs::CompilationRejection {
    match compile_source(source) {
        CompilationOutcome::Rejected(compilation_rejection) => compilation_rejection,
        CompilationOutcome::Compiled(generated_luau_text) => {
            assert!(
                false,
                "expected a compiler rejection, generated: {}",
                generated_luau_text.into_text()
            );
            unreachable!();
        }
    }
}

fn compiled_luau(source: &str) -> String {
    match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected loop-exit fixture with {} problems",
                compilation_rejection.problem_count()
            );
            unreachable!();
        }
    }
}

fn official_luau_tool(tool_name: (&str, &str)) -> PathBuf {
    let (environment_variable, executable_name) = tool_name;
    if let Some(configured_path) = std::env::var_os(environment_variable) {
        return PathBuf::from(configured_path);
    }
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
        .join(executable_filename);
    assert!(
        checkout_build_path.is_file(),
        "official Luau tool is required; set {environment_variable} or build it in references/checkouts/luau/build"
    );
    checkout_build_path
}

fn run_official_luau_tool(tool_and_source: (&Path, &Path)) -> Output {
    let (tool_path, generated_luau_path) = tool_and_source;
    match Command::new(tool_path).arg(generated_luau_path).output() {
        Ok(tool_output) => tool_output,
        Err(execution_error) => {
            assert!(
                false,
                "could not invoke official Luau tool {}: {execution_error}",
                tool_path.display()
            );
            unreachable!();
        }
    }
}
