use roblox_rust::{compile_source, CompilationOutcome, CompilationProblemReason};

const RESERVED_NAME_START_BYTE: usize = 3;
const RESERVED_NAME_END_BYTE: usize = 6;

#[test]
fn a_luau_reserved_word_cannot_name_a_function() {
    let source = "fn end() {}\nfn main() {}\n";

    let compilation_rejection = match compile_source(source) {
        CompilationOutcome::Rejected(compilation_rejection) => compilation_rejection,
        CompilationOutcome::Compiled(generated_luau_text) => {
            let generated_text = generated_luau_text.into_text();
            assert!(false, "expected rejection, compiled: {generated_text}");
            return;
        }
    };
    let compilation_problem = compilation_rejection.first_problem();
    assert_eq!(
        compilation_problem.reason(),
        &CompilationProblemReason::NameNotAllowedInLuau
    );
    let source_range = compilation_problem.source_range();
    assert_eq!(source_range.start_byte(), RESERVED_NAME_START_BYTE);
    assert_eq!(source_range.end_byte(), RESERVED_NAME_END_BYTE);
}
