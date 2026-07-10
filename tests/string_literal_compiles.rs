use roblox_rust::{compile_source, CompilationOutcome};

#[test]
fn a_quoted_string_initializes_a_string_local() {
    let source = "fn main() { let greeting: string = \"hello\"; }\n";
    let expected_luau_text = r#"--!strict

local function main(): ()
    const greeting: string = "hello"
end

main()
"#;

    let generated_luau_text = match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected string fixture with {} problems",
                compilation_rejection.problem_count()
            );
            return;
        }
    };

    assert_eq!(generated_luau_text, expected_luau_text);
}
