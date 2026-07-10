use roblox_rust::{compile_source, CompilationOutcome, CompilationProblemReason};

const COLLIDING_LOCAL_START_BYTE: usize = 28;
const COLLIDING_LOCAL_END_BYTE: usize = 31;

#[test]
fn a_local_cannot_reuse_a_visible_function_name() {
    let source = "fn add() {}\nfn main() { let add: number = 1; add(); }\n";

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
        &CompilationProblemReason::NameAlreadyDefined
    );
    let source_range = compilation_problem.source_range();
    assert_eq!(source_range.start_byte(), COLLIDING_LOCAL_START_BYTE);
    assert_eq!(source_range.end_byte(), COLLIDING_LOCAL_END_BYTE);
}
