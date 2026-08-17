//! Public compiler coverage for file-local typed record aliases, literals, and reads.

mod support;
use support::{official_luau_tool, run_official_luau_tool_required, temporary_luau_file};

use luau_rs::{compile_source, CompilationOutcome, CompilationProblemReason};

#[test]
fn typed_records_compile_diagnose_and_execute_as_strict_luau() {
    let source = r#"struct Profile {
    name: string,
    score: number,
}

fn total(profile: Profile) -> number {
    return profile.score;
}

fn main() {
    let profile: Profile = Profile { name: "Ada", score: 42 };
    if (Profile { name: "Ada", score: 42 }).score == 42 {
        print(total(profile));
    } else {
        print(0);
    }
}
"#;
    let generated_luau_text = compiled_text(source);
    insta::assert_snapshot!(generated_luau_text);

    match full_moon::parse(&generated_luau_text) {
        Ok(_) => {}
        Err(parse_errors) => assert!(false, "Full Moon rejected generated Luau: {parse_errors:?}"),
    }

    let generated_luau_path = temporary_luau_file("luau-rs-records");
    match std::fs::write(generated_luau_path.path(), &generated_luau_text) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write record fixture {}: {write_error}",
                generated_luau_path.path().display()
            );
            return;
        }
    }
    let luau_path = official_luau_tool(("LUAU_BIN", "luau"));
    let luau_analyze_path = official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"));
    let analyzer_output =
        run_official_luau_tool_required((&luau_analyze_path, generated_luau_path.path()));
    assert!(
        analyzer_output.status.success(),
        "official luau-analyze rejected generated records:\n{}",
        String::from_utf8_lossy(&analyzer_output.stderr)
    );
    let runtime_output = run_official_luau_tool_required((&luau_path, generated_luau_path.path()));
    assert!(
        runtime_output.status.success(),
        "official luau rejected generated records:\n{}",
        String::from_utf8_lossy(&runtime_output.stderr)
    );
    let expected_runtime_output: &[u8] = if cfg!(windows) { b"42\r\n" } else { b"42\n" };
    assert_eq!(runtime_output.stdout, expected_runtime_output);
    assert_rejection((
        "struct Profile { score: number, }\nfn main() { let profile: Profile = Unknown { score: 42 }; }\n",
        "Unknown",
        CompilationProblemReason::UnknownRecordType,
    ));
    assert_rejection((
        "struct Profile { score: number, score: number, }\nfn main() {}\n",
        "score",
        CompilationProblemReason::DuplicateRecordField,
    ));
    assert_rejection((
        "struct Profile { score: number, }\nfn main() { let profile: Profile = Profile { name: \"Ada\", score: 42 }; }\n",
        "name",
        CompilationProblemReason::UnknownRecordField,
    ));
    assert_rejection((
        "struct Profile { score: number, }\nfn main() { let profile: Profile = Profile { score: \"Ada\" }; }\n",
        "\"Ada\"",
        CompilationProblemReason::RecordFieldInitializerTypeMismatch,
    ));
    assert_rejection((
        "struct Profile { score: number, name: string, }\nfn main() { let profile: Profile = Profile { score: 42 }; }\n",
        "Profile",
        CompilationProblemReason::MissingRecordField,
    ));
    assert_rejection((
        "struct Profile { score: number, }\nfn main() { let profile: Profile = Profile { score: 42 }; print(profile.name); }\n",
        "name",
        CompilationProblemReason::UnknownRecordAccessField,
    ));
    assert_rejection((
        "fn main() { let score: number = 42; print(score.value); }\n",
        "score",
        CompilationProblemReason::FieldAccessRequiresRecord,
    ));
}

fn compiled_text(source: &str) -> String {
    match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected typed record fixture with {} problems",
                compilation_rejection.problem_count()
            );
            String::new()
        }
    }
}

fn assert_rejection(source_and_range: (&str, &str, CompilationProblemReason)) {
    let (source, ranged_spelling, expected_reason) = source_and_range;
    let compilation_rejection = match compile_source(source) {
        CompilationOutcome::Rejected(compilation_rejection) => compilation_rejection,
        CompilationOutcome::Compiled(generated_luau_text) => {
            assert!(
                false,
                "expected record diagnostic, generated: {}",
                generated_luau_text.into_text()
            );
            return;
        }
    };
    let compilation_problem = compilation_rejection.first_problem();
    assert_eq!(compilation_problem.reason(), &expected_reason);
    let start_byte = source
        .rfind(ranged_spelling)
        .map_or(0, |start_byte| start_byte);
    let end_byte = start_byte + ranged_spelling.len();
    assert_eq!(compilation_problem.source_range().start_byte(), start_byte);
    assert_eq!(compilation_problem.source_range().end_byte(), end_byte);
}
