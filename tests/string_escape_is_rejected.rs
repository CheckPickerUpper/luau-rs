use roblox_rust::{compile_source, CompilationOutcome, CompilationProblemReason};

const BACKSLASH_START_BYTE: usize = 36;
const BACKSLASH_END_BYTE: usize = 37;

#[test]
fn a_string_escape_is_rejected_at_the_backslash() {
    let source = "fn main() { let text: string = \"line\\nbreak\"; }\n";

    let compilation_rejection = match compile_source(source) {
        CompilationOutcome::Rejected(compilation_rejection) => compilation_rejection,
        CompilationOutcome::Compiled(generated_luau_text) => {
            let generated_text = generated_luau_text.into_text();
            assert!(false, "expected rejection, generated: {generated_text}");
            return;
        }
    };
    let compilation_problem = compilation_rejection.first_problem();
    assert_eq!(
        compilation_problem.reason(),
        &CompilationProblemReason::UnsupportedCharacter('\\')
    );
    let source_range = compilation_problem.source_range();
    assert_eq!(source_range.start_byte(), BACKSLASH_START_BYTE);
    assert_eq!(source_range.end_byte(), BACKSLASH_END_BYTE);
}
