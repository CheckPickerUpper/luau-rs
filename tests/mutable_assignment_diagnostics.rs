//! Verifies that mutable-binding diagnostics identify the rejected source intent.

use luau_rs::{compile_source, CompilationOutcome, CompilationProblemReason};

#[test]
fn an_immutable_local_write_points_to_the_assigned_name() {
    let source = "fn main() {\n    let total: number = 1;\n    total = 2;\n}\n";
    let Some(assigned_name_start) = source.rfind("total") else {
        assert!(false, "fixture must contain the assigned local name");
        return;
    };

    let compilation_rejection = match compile_source(source) {
        CompilationOutcome::Rejected(compilation_rejection) => compilation_rejection,
        CompilationOutcome::Compiled(generated_luau_text) => {
            assert!(
                false,
                "expected rejection, generated: {}",
                generated_luau_text.into_text()
            );
            return;
        }
    };
    let compilation_problem = compilation_rejection.first_problem();
    assert_eq!(
        compilation_problem.reason(),
        &CompilationProblemReason::ImmutableBindingCannotBeAssigned
    );
    let source_range = compilation_problem.source_range();
    assert_eq!(source_range.start_byte(), assigned_name_start);
    assert_eq!(source_range.end_byte(), assigned_name_start + "total".len());
}

#[test]
fn a_mutable_local_type_mismatch_points_to_the_assigned_value() {
    let source = "fn main() {\n    let mut count: number = 1;\n    count = \"wrong\";\n}\n";
    let Some(assigned_value_start) = source.find("\"wrong\"") else {
        assert!(false, "fixture must contain the mismatched value");
        return;
    };

    let compilation_rejection = match compile_source(source) {
        CompilationOutcome::Rejected(compilation_rejection) => compilation_rejection,
        CompilationOutcome::Compiled(generated_luau_text) => {
            assert!(
                false,
                "expected rejection, generated: {}",
                generated_luau_text.into_text()
            );
            return;
        }
    };
    let compilation_problem = compilation_rejection.first_problem();
    assert_eq!(
        compilation_problem.reason(),
        &CompilationProblemReason::TypesDoNotMatch
    );
    let source_range = compilation_problem.source_range();
    assert_eq!(source_range.start_byte(), assigned_value_start);
    assert_eq!(
        source_range.end_byte(),
        assigned_value_start + "\"wrong\"".len()
    );
}
