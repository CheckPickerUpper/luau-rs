//! Public compiler coverage for file-local typed record aliases, literals, and reads.

use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

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
    assert!(generated_luau_text.starts_with("--!strict\n\ntype Profile = {\n"));
    assert!(generated_luau_text.contains("    name: string,\n    score: number,\n}"));
    assert!(generated_luau_text.contains("profile: Profile"));
    assert!(generated_luau_text.contains("{name = \"Ada\", score = 42}"));
    assert!(generated_luau_text.contains("return profile.score"));

    match full_moon::parse(&generated_luau_text) {
        Ok(_) => {}
        Err(parse_errors) => assert!(false, "Full Moon rejected generated Luau: {parse_errors:?}"),
    }

    let generated_luau_path = temporary_luau_path();
    match std::fs::write(&generated_luau_path, &generated_luau_text) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write record fixture {}: {write_error}",
                generated_luau_path.display()
            );
            return;
        }
    }
    let Some(luau_path) = resolve_official_luau_tool(("LUAU_BIN", "luau")) else {
        assert!(
            false,
            "official luau is required for typed record runtime coverage"
        );
        return;
    };
    let Some(luau_analyze_path) = resolve_official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"))
    else {
        assert!(
            false,
            "official luau-analyze is required for typed record analysis coverage"
        );
        return;
    };
    let analyzer_output = run_official_luau_tool((&luau_analyze_path, &generated_luau_path));
    assert!(
        analyzer_output.status.success(),
        "official luau-analyze rejected generated records:\n{}",
        String::from_utf8_lossy(&analyzer_output.stderr)
    );
    let runtime_output = run_official_luau_tool((&luau_path, &generated_luau_path));
    assert!(
        runtime_output.status.success(),
        "official luau rejected generated records:\n{}",
        String::from_utf8_lossy(&runtime_output.stderr)
    );
    let expected_runtime_output: &[u8] = if cfg!(windows) { b"42\r\n" } else { b"42\n" };
    assert_eq!(runtime_output.stdout, expected_runtime_output);
    match std::fs::remove_file(&generated_luau_path) {
        Ok(()) => {}
        Err(remove_error) => assert!(
            false,
            "could not remove record fixture {}: {remove_error}",
            generated_luau_path.display()
        ),
    }

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

fn temporary_luau_path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "roblox-rust-typed-records-{}.luau",
        std::process::id()
    ))
}

fn resolve_official_luau_tool(tool_name: (&str, &str)) -> Option<PathBuf> {
    let (environment_variable, executable_name) = tool_name;
    std::env::var_os(environment_variable).map_or_else(
        || {
            let executable_filename = if cfg!(windows) {
                format!("{executable_name}.exe")
            } else {
                executable_name.to_owned()
            };
            let checkout_build_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("references")
                .join("checkouts")
                .join("luau")
                .join("build")
                .join("release")
                .join(executable_filename);
            if checkout_build_path.is_file() {
                Some(checkout_build_path)
            } else {
                None
            }
        },
        |configured_path| Some(PathBuf::from(configured_path)),
    )
}

fn run_official_luau_tool(tool_and_source: (&Path, &Path)) -> Output {
    let (tool_path, generated_luau_path) = tool_and_source;
    match Command::new(tool_path).arg(generated_luau_path).output() {
        Ok(tool_output) => tool_output,
        Err(execution_error) => {
            assert!(
                false,
                "could not invoke official Luau tool {}: {execution_error}",
                tool_path.display()
            );
            std::process::exit(1);
        }
    }
}
