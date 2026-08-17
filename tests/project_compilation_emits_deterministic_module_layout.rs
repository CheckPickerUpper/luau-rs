//! Verifies project compilation preserves deterministic Luau module placement.

mod support;
use std::path::Path;
use support::{official_luau_tool, run_official_luau_tool_required, temporary_luau_file};

use full_moon::ast::LuaVersion;
use luau_rs::{
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

    let luau_path = official_luau_tool(("LUAU_BIN", "luau"));
    let luau_analyze_path = official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"));

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
    insta::assert_snapshot!(format!("module-{module_index}"), generated_luau);
    assert!(generated_luau.starts_with("--!strict\n"));
    assert!(
        full_moon::parse_fallible(generated_luau, LuaVersion::luau())
            .into_result()
            .is_ok(),
        "Full Moon rejected {output_path}"
    );

    let generated_luau_path =
        temporary_luau_file(&format!("luau-rs-project-layout-{module_index}"));
    match std::fs::write(generated_luau_path.path(), generated_luau) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write {output_path} for official Luau validation: {write_error}"
            );
            return;
        }
    }

    let analysis_output =
        run_official_luau_tool_required((luau_analyze_path, generated_luau_path.path()));
    assert!(
        analysis_output.status.success(),
        "official luau-analyze rejected {output_path}:\n{}",
        String::from_utf8_lossy(&analysis_output.stderr)
    );

    if output_path.ends_with(".server.luau") || output_path.ends_with(".client.luau") {
        let runtime_output =
            run_official_luau_tool_required((luau_path, generated_luau_path.path()));
        assert!(
            runtime_output.status.success(),
            "official luau could not execute {output_path}:\n{}",
            String::from_utf8_lossy(&runtime_output.stderr)
        );
    }
}
