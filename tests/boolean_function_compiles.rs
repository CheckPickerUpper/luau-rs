//! Integration coverage for compiling boolean functions.

use roblox_rust::{compile_source, CompilationOutcome};

#[test]
fn both_boolean_literals_flow_through_a_function_and_print() {
    let source = r"fn identity(flag: boolean) -> boolean {
    return flag;
}

fn main() {
    let enabled: boolean = identity(true);
    let disabled: boolean = false;
    print(enabled);
    print(disabled);
}
";
    let expected_luau_text = r"--!strict

local function identity(flag: boolean): boolean
    return flag
end

local function main(): ()
    const enabled: boolean = identity(true)
    const disabled: boolean = false
    print(enabled)
    print(disabled)
end

main()
";

    let generated_luau_text = match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected boolean fixture with {} problems",
                compilation_rejection.problem_count()
            );
            return;
        }
    };

    assert_eq!(generated_luau_text, expected_luau_text);
}
