//! Verifies project compilation preserves deterministic Luau module placement.

use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use full_moon::ast::LuaVersion;
use roblox_rust::{
    compile_project, ProjectCompilationOutcome, ProjectCompilationProblem,
    ProjectCompilationRequest, ProjectModuleIdentity, ProjectModuleRole, ProjectModuleSource,
};

#[test]
fn a_project_emits_sorted_strict_modules_at_their_roblox_locations() {
    let first_project_outcome = compile_project(project_request());
    let second_project_outcome = compile_project(project_request());

    let first_compiled_project = match first_project_outcome {
        ProjectCompilationOutcome::Compiled(compiled_project) => compiled_project,
        ProjectCompilationOutcome::Rejected(project_rejection) => {
            assert!(false, "project rejected: {project_rejection:#?}");
            return;
        }
    };
    let second_compiled_project = match second_project_outcome {
        ProjectCompilationOutcome::Compiled(compiled_project) => compiled_project,
        ProjectCompilationOutcome::Rejected(project_rejection) => {
            assert!(false, "project rejected: {project_rejection:#?}");
            return;
        }
    };

    let first_output_paths: Vec<_> = first_compiled_project
        .generated_modules()
        .iter()
        .map(|generated_module| generated_module.output_path().as_str())
        .collect();
    let second_output_paths: Vec<_> = second_compiled_project
        .generated_modules()
        .iter()
        .map(|generated_module| generated_module.output_path().as_str())
        .collect();
    assert_eq!(
        first_output_paths,
        [
            "ServerScriptService/data/cache.luau",
            "ServerScriptService/start.server.luau",
            "StarterPlayer/StarterPlayerScripts/hud.client.luau",
            "StarterPlayer/StarterPlayerScripts/ui/theme.luau",
            "ReplicatedStorage/math.luau",
        ]
    );
    assert_eq!(first_output_paths, second_output_paths);

    let Some(luau_path) = resolve_official_luau_tool(("LUAU_BIN", "luau")) else {
        fail_missing_official_luau_tool("LUAU_BIN", "luau");
        return;
    };
    let Some(luau_analyze_path) = resolve_official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"))
    else {
        fail_missing_official_luau_tool("LUAU_ANALYZE_BIN", "luau-analyze");
        return;
    };

    for (module_index, generated_module) in first_compiled_project
        .generated_modules()
        .iter()
        .enumerate()
    {
        validate_generated_module((
            generated_module.output_path().as_str(),
            generated_module.generated_luau_text().as_text(),
            module_index,
            &luau_path,
            &luau_analyze_path,
        ));
    }
}

#[test]
fn a_project_rejects_an_invalid_module_identity_before_accepting_output() {
    let project_outcome = compile_project(ProjectCompilationRequest::from_source_modules(vec![
        ProjectModuleSource::from_source_parts((
            ProjectModuleIdentity::Server {
                module_path: "start".to_owned(),
            },
            ProjectModuleRole::Entrypoint,
            "fn main() {}".to_owned(),
        )),
        ProjectModuleSource::from_source_parts((
            ProjectModuleIdentity::Shared {
                module_path: "bad-name".to_owned(),
            },
            ProjectModuleRole::Library,
            "fn helper() {}".to_owned(),
        )),
    ]));

    match project_outcome {
        ProjectCompilationOutcome::Compiled(_) => {
            assert!(false, "project accepted an invalid source module identity");
        }
        ProjectCompilationOutcome::Rejected(project_rejection) => {
            assert!(matches!(
                project_rejection.first_problem(),
                ProjectCompilationProblem::InvalidModuleIdentity { module_identity }
                    if module_identity.module_path() == "bad-name"
            ));
        }
    }
}

#[test]
fn a_project_rejects_a_shared_entrypoint_before_emitting_output() {
    let project_outcome = compile_project(ProjectCompilationRequest::from_source_modules(vec![
        ProjectModuleSource::from_source_parts((
            ProjectModuleIdentity::Server {
                module_path: "start".to_owned(),
            },
            ProjectModuleRole::Entrypoint,
            "fn main() {}".to_owned(),
        )),
        ProjectModuleSource::from_source_parts((
            ProjectModuleIdentity::Shared {
                module_path: "bootstrap".to_owned(),
            },
            ProjectModuleRole::Entrypoint,
            "fn main() {}".to_owned(),
        )),
    ]));

    match project_outcome {
        ProjectCompilationOutcome::Compiled(_) => {
            assert!(false, "project emitted a shared entrypoint");
        }
        ProjectCompilationOutcome::Rejected(project_rejection) => {
            assert!(matches!(
                project_rejection.first_problem(),
                ProjectCompilationProblem::SharedModuleCannotBeEntrypoint { module_identity }
                    if module_identity.module_path() == "bootstrap"
            ));
        }
    }
}

fn project_request() -> ProjectCompilationRequest {
    ProjectCompilationRequest::from_source_modules(vec![
        ProjectModuleSource::from_source_parts((
            ProjectModuleIdentity::Shared {
                module_path: "math".to_owned(),
            },
            ProjectModuleRole::Library,
            "fn double(value: number) -> number { return value * 2; }".to_owned(),
        )),
        ProjectModuleSource::from_source_parts((
            ProjectModuleIdentity::Client {
                module_path: "ui/theme".to_owned(),
            },
            ProjectModuleRole::Library,
            "fn theme_name() -> string { return \"dark\"; }".to_owned(),
        )),
        ProjectModuleSource::from_source_parts((
            ProjectModuleIdentity::Client {
                module_path: "hud".to_owned(),
            },
            ProjectModuleRole::Entrypoint,
            "fn main() { print(\"ready\"); }".to_owned(),
        )),
        ProjectModuleSource::from_source_parts((
            ProjectModuleIdentity::Server {
                module_path: "start".to_owned(),
            },
            ProjectModuleRole::Entrypoint,
            "fn main() { print(\"started\"); }".to_owned(),
        )),
        ProjectModuleSource::from_source_parts((
            ProjectModuleIdentity::Server {
                module_path: "data/cache".to_owned(),
            },
            ProjectModuleRole::Library,
            "fn cache_size() -> number { return 0; }".to_owned(),
        )),
    ])
}

fn validate_generated_module(generated_module_parts: (&str, &str, usize, &Path, &Path)) {
    let (output_path, generated_luau, module_index, luau_path, luau_analyze_path) =
        generated_module_parts;
    assert!(generated_luau.starts_with("--!strict\n"));
    assert!(
        full_moon::parse_fallible(generated_luau, LuaVersion::luau())
            .into_result()
            .is_ok(),
        "Full Moon rejected {output_path}"
    );

    let generated_luau_path = std::env::temp_dir().join(format!(
        "roblox-rust-project-layout-{}-{module_index}.luau",
        std::process::id()
    ));
    match std::fs::write(&generated_luau_path, generated_luau) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write {output_path} for official Luau validation: {write_error}"
            );
            return;
        }
    }

    let Some(analysis_output) = run_official_luau_tool((luau_analyze_path, &generated_luau_path))
    else {
        return;
    };
    assert!(
        analysis_output.status.success(),
        "official luau-analyze rejected {output_path}:\n{}",
        String::from_utf8_lossy(&analysis_output.stderr)
    );

    if output_path.ends_with(".server.luau") || output_path.ends_with(".client.luau") {
        let Some(runtime_output) = run_official_luau_tool((luau_path, &generated_luau_path)) else {
            return;
        };
        assert!(
            runtime_output.status.success(),
            "official luau could not execute {output_path}:\n{}",
            String::from_utf8_lossy(&runtime_output.stderr)
        );
    }

    match std::fs::remove_file(&generated_luau_path) {
        Ok(()) => {}
        Err(remove_error) => assert!(
            false,
            "could not remove generated Luau validation artifact {}: {remove_error}",
            generated_luau_path.display()
        ),
    }
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
            let checkout_build_directory = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("references")
                .join("checkouts")
                .join("luau")
                .join("build");
            let cmake_binary = checkout_build_directory.join(&executable_filename);
            if cmake_binary.is_file() {
                Some(cmake_binary)
            } else {
                let release_binary = checkout_build_directory
                    .join("release")
                    .join(executable_filename);
                if release_binary.is_file() {
                    Some(release_binary)
                } else {
                    None
                }
            }
        },
        |configured_path| Some(PathBuf::from(configured_path)),
    )
}

fn run_official_luau_tool(tool_and_source: (&Path, &Path)) -> Option<Output> {
    let (tool_path, generated_luau_path) = tool_and_source;
    match Command::new(tool_path).arg(generated_luau_path).output() {
        Ok(tool_output) => Some(tool_output),
        Err(execution_error) => {
            assert!(
                false,
                "could not invoke official Luau tool {}: {execution_error}",
                tool_path.display()
            );
            None
        }
    }
}

fn fail_missing_official_luau_tool(environment_variable: &str, executable_name: &str) {
    assert!(
        false,
        "official {executable_name} is required; set {environment_variable} or build it in references/checkouts/luau/build"
    );
}
