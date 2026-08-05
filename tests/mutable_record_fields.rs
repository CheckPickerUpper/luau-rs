//! Verifies record field updates remain typed, strict-Luau compatible, and executable.

use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use full_moon::ast::LuaVersion;
use roblox_rust::{compile_source, CompilationOutcome, CompilationProblemReason};

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
    let expected_luau = r"--!strict

type Level = {
    value: number,
}

type Profile = {
    level: Level,
}

local function main(): ()
    local profile: Profile = {level = {value = 1}}
    profile.level.value = profile.level.value + 41
    print(profile.level.value)
end

main()
";
    let generated_luau = compiled_text(source);
    assert_eq!(generated_luau, expected_luau);

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

    let generated_luau_path = std::env::temp_dir().join(format!(
        "roblox-rust-mutable-record-fields-{}.luau",
        std::process::id()
    ));
    match std::fs::write(&generated_luau_path, generated_luau) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write generated Luau fixture to {}: {write_error}",
                generated_luau_path.display()
            );
            return;
        }
    }
    let Some(luau_path) = resolve_official_luau_tool(("LUAU_BIN", "luau")) else {
        fail_missing_official_luau_tools();
        return;
    };
    let Some(luau_analyze_path) = resolve_official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"))
    else {
        fail_missing_official_luau_tools();
        return;
    };
    let runtime_output = run_official_luau_tool((&luau_path, &generated_luau_path));
    assert!(
        runtime_output.status.success(),
        "official luau rejected mutable record field execution:\n{}",
        String::from_utf8_lossy(&runtime_output.stderr)
    );
    let expected_runtime_output: &[u8] = if cfg!(windows) { b"42\r\n" } else { b"42\n" };
    assert_eq!(runtime_output.stdout, expected_runtime_output);
    let analyzer_output = run_official_luau_tool((&luau_analyze_path, &generated_luau_path));
    assert!(
        analyzer_output.status.success(),
        "official luau-analyze rejected mutable record fields:\n{}",
        String::from_utf8_lossy(&analyzer_output.stderr)
    );
    match std::fs::remove_file(&generated_luau_path) {
        Ok(()) => {}
        Err(remove_error) => assert!(
            false,
            "could not remove generated Luau fixture {}: {remove_error}",
            generated_luau_path.display()
        ),
    }

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

fn fail_missing_official_luau_tools() {
    assert!(
        false,
        "official Luau tools are required; set LUAU_BIN and LUAU_ANALYZE_BIN to executable paths or build them in references/checkouts/luau"
    );
}
