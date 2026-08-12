//! Integration coverage for strict Luau program generation.

use luau_rs::{compile_source, CompilationOutcome};

#[test]
fn a_program_with_functions_and_a_call_compiles_to_strict_luau() {
    let source = r"fn add(left: number, right: number) -> number {
    return left + right;
}

fn main() {
    let total: number = add(20, 22);
    print(total);
}
";
    let expected_luau_text = r"--!strict

local function add(left: number, right: number): number
    return left + right
end

local function main(): ()
    const total: number = add(20, 22)
    print(total)
end

main()
";

    let actual_luau_text = match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert_eq!(compilation_rejection.problem_count(), 0);
            String::new()
        }
    };

    assert_eq!(actual_luau_text, expected_luau_text);
}
