//! Verifies record field updates remain typed, strict-Luau compatible, and executable.

mod support;
use support::{official_luau_tool, run_official_luau_tool_required, temporary_luau_file};

use full_moon::ast::LuaVersion;
use luau_rs::{compile_source, CompilationOutcome, CompilationProblemReason};

#[test]
fn mutable_nested_record_fields_update_in_strict_luau() {
    let source = r"struct Level {
    value: number,
}

struct Profile {
    level: Level,
}

fn main() {
    let mut profile: Profile = Profile { level: Level { value: 1 } };
    profile.level.value = profile.level.value + 41;
    print(profile.level.value);
}
";
    let generated_luau = compiled_text(source);
    insta::assert_snapshot!(generated_luau);

    match full_moon::parse_fallible(&generated_luau, LuaVersion::luau()).into_result() {
        Ok(_) => {}
        Err(parse_errors) => {
            assert!(
                false,
                "Full Moon rejected mutable record field Luau: {parse_errors:?}"
            );
            return;
        }
    }

    let generated_luau_path = temporary_luau_file("luau-rs-mutable-record-fields");
    match std::fs::write(generated_luau_path.path(), generated_luau) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write generated Luau fixture to {}: {write_error}",
                generated_luau_path.path().display()
            );
            return;
        }
    }
    let luau_path = official_luau_tool(("LUAU_BIN", "luau"));
    let luau_analyze_path = official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"));
    let runtime_output = run_official_luau_tool_required((&luau_path, generated_luau_path.path()));
    assert!(
        runtime_output.status.success(),
        "official luau rejected mutable record field execution:\n{}",
        String::from_utf8_lossy(&runtime_output.stderr)
    );
    let expected_runtime_output: &[u8] = if cfg!(windows) { b"42\r\n" } else { b"42\n" };
    assert_eq!(runtime_output.stdout, expected_runtime_output);
    let analyzer_output =
        run_official_luau_tool_required((&luau_analyze_path, generated_luau_path.path()));
    assert!(
        analyzer_output.status.success(),
        "official luau-analyze rejected mutable record fields:\n{}",
        String::from_utf8_lossy(&analyzer_output.stderr)
    );
    assert_rejection((
        "struct Level { value: number, }\nstruct Profile { level: Level, }\nfn main() { let profile: Profile = Profile { level: Level { value: 1 } }; profile.level.value = 42; }\n",
        "profile",
        CompilationProblemReason::ImmutableBindingCannotBeAssigned,
    ));
    assert_rejection((
        "struct Level { value: number, }\nstruct Profile { level: Level, }\nfn main() { let mut profile: Profile = Profile { level: Level { value: 1 } }; profile.level.unknown = 42; }\n",
        "unknown",
        CompilationProblemReason::UnknownRecordAccessField,
    ));
    assert_rejection((
        "struct Level { value: number, }\nstruct Profile { level: Level, }\nfn main() { let mut profile: Profile = Profile { level: Level { value: 1 } }; profile.level.value = \"wrong\"; }\n",
        "\"wrong\"",
        CompilationProblemReason::TypesDoNotMatch,
    ));
}

fn compiled_text(source: &str) -> String {
    match compile_source(source) {
        CompilationOutcome::Compiled(generated_luau_text) => generated_luau_text.into_text(),
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(
                false,
                "compiler rejected mutable record field fixture with {} problems",
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
                "expected field-assignment diagnostic, generated: {}",
                generated_luau_text.into_text()
            );
            return;
        }
    };
    let compilation_problem = compilation_rejection.first_problem();
    assert_eq!(compilation_problem.reason(), &expected_reason);
    let Some(start_byte) = source.rfind(ranged_spelling) else {
        assert!(false, "fixture must contain the ranged spelling");
        return;
    };
    assert_eq!(compilation_problem.source_range().start_byte(), start_byte);
    assert_eq!(
        compilation_problem.source_range().end_byte(),
        start_byte + ranged_spelling.len()
    );
}
