use roblox_rust::{compile_source, CompilationOutcome, CompilationProblemReason};

const FORWARD_CALL_START_BYTE: usize = 12;
const FORWARD_CALL_END_BYTE: usize = 17;

#[test]
fn a_function_cannot_call_a_later_declaration() {
    let source = "fn main() { later(); }\nfn later() {}\n";

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
        &CompilationProblemReason::NameUsedBeforeDeclaration
    );
    let source_range = compilation_problem.source_range();
    assert_eq!(source_range.start_byte(), FORWARD_CALL_START_BYTE);
    assert_eq!(source_range.end_byte(), FORWARD_CALL_END_BYTE);
}
