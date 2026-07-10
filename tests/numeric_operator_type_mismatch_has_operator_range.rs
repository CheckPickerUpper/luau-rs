use roblox_rust::{compile_source, CompilationOutcome, CompilationProblemReason};

#[test]
fn subtraction_rejects_a_string_operand_at_the_operator_range() {
    let source = "fn main() { let computed_value: number = \"x\" - 1; }\n";
    let operator_start_byte = match source.find('-') {
        Some(operator_start_byte) => operator_start_byte,
        None => {
            assert!(false, "subtraction fixture is missing its operator");
            return;
        }
    };
    let operator_end_byte = operator_start_byte + '-'.len_utf8();

    let compilation_rejection = match compile_source(source) {
        CompilationOutcome::Rejected(compilation_rejection) => compilation_rejection,
        CompilationOutcome::Compiled(generated_luau_text) => {
            let generated_text = generated_luau_text.into_text();
            assert!(
                false,
                "expected numeric type rejection, generated: {generated_text}"
            );
            return;
        }
    };
    let compilation_problem = compilation_rejection.first_problem();
    assert_eq!(
        compilation_problem.reason(),
        &CompilationProblemReason::TypesDoNotMatch
    );
    assert_eq!(
        compilation_problem.source_range().start_byte(),
        operator_start_byte
    );
    assert_eq!(
        compilation_problem.source_range().end_byte(),
        operator_end_byte
    );
}
