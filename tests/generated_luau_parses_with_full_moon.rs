//! Integration coverage for parsing generated Luau with `full_moon`.

use full_moon::ast::{LuaVersion, Stmt};
use luau_rs::{compile_source, CompilationOutcome};

#[test]
fn generated_luau_has_two_functions_and_an_entry_call() {
    let source = r"fn add(left: number, right: number) -> number {
    return left + right;
}

fn main() {
    let total: number = add(20, 22);
    print(total);
}
";

    let generated_luau_text = match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected validation fixture with {} problems",
                compilation_rejection.problem_count()
            );
            return;
        }
    };

    let generated_luau_ast =
        match full_moon::parse_fallible(&generated_luau_text, LuaVersion::luau()).into_result() {
            Ok(generated_luau_ast) => generated_luau_ast,
            Err(parse_errors) => {
                assert!(
                    false,
                    "Full Moon rejected generated Luau: {parse_errors:#?}"
                );
                return;
            }
        };

    let local_function_count = generated_luau_ast
        .nodes()
        .stmts()
        .filter(|statement| matches!(statement, Stmt::LocalFunction(_)))
        .count();
    let function_call_count = generated_luau_ast
        .nodes()
        .stmts()
        .filter(|statement| matches!(statement, Stmt::FunctionCall(_)))
        .count();

    assert_eq!(local_function_count, 2);
    assert_eq!(function_call_count, 1);
}
