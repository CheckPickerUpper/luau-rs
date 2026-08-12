//! Exercises the bounded V1 Roblox service intrinsic through the public compiler APIs.

use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

use full_moon::ast::LuaVersion;
use luau_rs::{
    compile_project, compile_source, CompilationOutcome, CompilationProblemReason,
    ModuleExecutionSide, ProjectCompilationOutcome, ProjectCompilationProblem,
    ProjectCompilationRequest, ProjectModuleIdentity, ProjectModuleRole, ProjectModuleSource,
};

#[test]
fn project_service_acquisition_is_side_checked_and_lowers_to_recorded_get_service_calls() {
    let project_outcome = compile_project(accepted_service_project());
    let compiled_project = match project_outcome {
        ProjectCompilationOutcome::Compiled(compiled_project) => compiled_project,
        ProjectCompilationOutcome::Rejected(project_rejection) => {
            assert!(
                false,
                "accepted service project was rejected: {project_rejection:#?}"
            );
            return;
        }
    };

    let Some(luau_path) = official_luau_tool(("LUAU_BIN", "luau")) else {
        assert!(false, "the installed CMake Luau runtime is required");
        return;
    };
    let Some(luau_analyze_path) = official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze")) else {
        assert!(false, "the installed CMake Luau analyzer is required");
        return;
    };

    for generated_module in compiled_project.generated_modules() {
        let generated_luau = generated_module.generated_luau_text().as_text();
        assert!(
            full_moon::parse_fallible(generated_luau, LuaVersion::luau())
                .into_result()
                .is_ok(),
            "Full Moon rejected {}",
            generated_module.output_path().as_str()
        );
        run_luau_analyzer((
            &luau_analyze_path,
            generated_luau,
            generated_module.output_path().as_str(),
        ));
    }

    let server_module =
        generated_module_at((&compiled_project, "ServerScriptService/start.server.luau"));
    assert_get_service_calls((
        server_module,
        &[
            "Players",
            "ReplicatedStorage",
            "DataStoreService",
            "ServerScriptService",
        ],
    ));
    run_service_runtime_harness((
        &luau_path,
        server_module,
        &[
            "Players",
            "ReplicatedStorage",
            "DataStoreService",
            "ServerScriptService",
        ],
        "server",
    ));

    let client_module = generated_module_at((
        &compiled_project,
        "StarterPlayer/StarterPlayerScripts/start.client.luau",
    ));
    assert_get_service_calls((
        client_module,
        &["Players", "ReplicatedStorage", "UserInputService"],
    ));
    run_service_runtime_harness((
        &luau_path,
        client_module,
        &["Players", "ReplicatedStorage", "UserInputService"],
        "client",
    ));

    let shared_module = generated_module_at((&compiled_project, "ReplicatedStorage/shared.luau"));
    assert_get_service_calls((shared_module, &["Players", "ReplicatedStorage"]));

    assert_rejected_service_project((
        ModuleExecutionSide::Server,
        "UserInputService",
        CompilationProblemReason::RobloxServiceUnavailableOnModuleExecutionSide,
    ));
    assert_rejected_service_project((
        ModuleExecutionSide::Client,
        "DataStoreService",
        CompilationProblemReason::RobloxServiceUnavailableOnModuleExecutionSide,
    ));
    assert_rejected_service_project((
        ModuleExecutionSide::Shared,
        "ServerScriptService",
        CompilationProblemReason::RobloxServiceUnavailableOnModuleExecutionSide,
    ));
    assert_unknown_service_has_the_service_name_range();
    assert_annotation_mismatch_has_the_acquisition_range();
    assert_non_intrinsic_service_forms_are_rejected();
    assert_standalone_compilation_rejects_service_acquisition();
    assert_service_types_cannot_escape_local_acquisition();
    assert_service_members_are_rejected();
}

fn accepted_service_project() -> ProjectCompilationRequest {
    ProjectCompilationRequest::from_source_modules(vec![
        project_module((
            ModuleExecutionSide::Server,
            ProjectModuleRole::Entrypoint,
            "start",
            service_main_source(&[
                ("players", "Players"),
                ("storage", "ReplicatedStorage"),
                ("data_stores", "DataStoreService"),
                ("scripts", "ServerScriptService"),
            ]),
        )),
        project_module((
            ModuleExecutionSide::Client,
            ProjectModuleRole::Entrypoint,
            "start",
            service_main_source(&[
                ("players", "Players"),
                ("storage", "ReplicatedStorage"),
                ("input", "UserInputService"),
            ]),
        )),
        project_module((
            ModuleExecutionSide::Shared,
            ProjectModuleRole::Library,
            "shared",
            "fn acquire_shared_services() {\n    let players: Players = roblox::service::<Players>();\n    let storage: ReplicatedStorage = roblox::service::<ReplicatedStorage>();\n}\n".to_owned(),
        )),
    ])
}

fn service_main_source(service_bindings: &[(&str, &str)]) -> String {
    let mut source = String::from("fn main() {\n");
    for (local_name, service_type) in service_bindings {
        source.push_str("    let ");
        source.push_str(local_name);
        source.push_str(": ");
        source.push_str(service_type);
        source.push_str(" = roblox::service::<");
        source.push_str(service_type);
        source.push_str(">();\n");
    }
    source.push_str("}\n");
    source
}

fn project_module(
    module_parts: (ModuleExecutionSide, ProjectModuleRole, &str, String),
) -> ProjectModuleSource {
    let (execution_side, module_role, module_path, source_text) = module_parts;
    let module_identity = match execution_side {
        ModuleExecutionSide::Server => ProjectModuleIdentity::Server {
            module_path: module_path.to_owned(),
        },
        ModuleExecutionSide::Client => ProjectModuleIdentity::Client {
            module_path: module_path.to_owned(),
        },
        ModuleExecutionSide::Shared => ProjectModuleIdentity::Shared {
            module_path: module_path.to_owned(),
        },
    };
    ProjectModuleSource::from_source_parts((module_identity, module_role, source_text))
}

fn generated_module_at<'project>(
    module_lookup: (&'project luau_rs::CompiledProject, &str),
) -> &'project str {
    let (compiled_project, output_path) = module_lookup;
    let generated_module = compiled_project
        .generated_modules()
        .iter()
        .find(|module| module.output_path().as_str() == output_path);
    let Some(generated_module) = generated_module else {
        assert!(false, "project did not generate {output_path}");
        return "";
    };
    generated_module.generated_luau_text().as_text()
}

fn assert_get_service_calls(service_call_text: (&str, &[&str])) {
    let (generated_luau, expected_service_names) = service_call_text;
    for service_name in expected_service_names {
        let expected_call = format!("game:GetService(\"{service_name}\")");
        assert!(
            generated_luau.contains(&expected_call),
            "generated Luau omitted {expected_call}:\n{generated_luau}"
        );
    }
}

fn run_luau_analyzer(analyzer_run: (&Path, &str, &str)) {
    let (luau_analyze_path, generated_luau, module_name) = analyzer_run;
    let generated_luau_path = temporary_luau_path(module_name);
    let analyzer_source = analyzer_harness_source(generated_luau);
    match std::fs::write(&generated_luau_path, analyzer_source) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(false, "could not write {module_name}: {write_error}");
            return;
        }
    }
    let Some(analysis_output) = run_luau_tool((luau_analyze_path, &generated_luau_path)) else {
        return;
    };
    assert!(
        analysis_output.status.success(),
        "luau-analyze rejected {module_name}:\n{}",
        String::from_utf8_lossy(&analysis_output.stderr)
    );
    remove_temporary_luau(&generated_luau_path);
}

fn analyzer_harness_source(generated_luau: &str) -> String {
    let mut analyzer_source = String::from("--!strict\n--!nolint LocalUnused\ntype Players = {}\ntype ReplicatedStorage = {}\ntype UserInputService = {}\ntype DataStoreService = {}\ntype ServerScriptService = {}\nlocal game = {\n    GetService = function(_: any, _: string): any\n        return {}\n    end,\n}\n");
    match generated_luau.strip_prefix("--!strict\n") {
        Some(generated_body) => analyzer_source.push_str(generated_body),
        None => analyzer_source.push_str(generated_luau),
    }
    analyzer_source
}

fn run_service_runtime_harness(runtime_harness: (&Path, &str, &[&str], &str)) {
    let (luau_path, generated_luau, expected_service_names, runtime_name) = runtime_harness;
    let mut harness_source = String::from("--!strict\nlocal requested_services: {string} = {}\nlocal game = {}\nfunction game:GetService(service_name: string): {}\n    table.insert(requested_services, service_name)\n    return {}\nend\n");
    match generated_luau.strip_prefix("--!strict\n") {
        Some(generated_body) => harness_source.push_str(generated_body),
        None => harness_source.push_str(generated_luau),
    }
    harness_source.push_str("for index, expected_name in {\n");
    for service_name in expected_service_names {
        harness_source.push_str("    \"");
        harness_source.push_str(service_name);
        harness_source.push_str("\",\n");
    }
    harness_source.push_str("} do\n    assert(requested_services[index] == expected_name)\nend\nassert(#requested_services == ");
    harness_source.push_str(&expected_service_names.len().to_string());
    harness_source.push_str(")\n");
    let harness_path = temporary_luau_path(runtime_name);
    match std::fs::write(&harness_path, harness_source) {
        Ok(()) => {}
        Err(write_error) => {
            assert!(
                false,
                "could not write {runtime_name} harness: {write_error}"
            );
            return;
        }
    }
    let Some(runtime_output) = run_luau_tool((luau_path, &harness_path)) else {
        return;
    };
    assert!(
        runtime_output.status.success(),
        "fake game runtime rejected {runtime_name}:\n{}",
        String::from_utf8_lossy(&runtime_output.stderr)
    );
    remove_temporary_luau(&harness_path);
}

fn assert_rejected_service_project(
    rejected_service: (ModuleExecutionSide, &str, CompilationProblemReason),
) {
    let (execution_side, service_name, expected_reason) = rejected_service;
    let source = service_main_source(&[("service", service_name)]);
    let source_modules = match execution_side {
        ModuleExecutionSide::Shared => vec![
            project_module((
                ModuleExecutionSide::Server,
                ProjectModuleRole::Entrypoint,
                "start",
                "fn main() {}".to_owned(),
            )),
            project_module((
                ModuleExecutionSide::Shared,
                ProjectModuleRole::Library,
                "shared",
                source.replacen("fn main", "fn acquire", 1),
            )),
        ],
        ModuleExecutionSide::Server | ModuleExecutionSide::Client => vec![project_module((
            execution_side,
            ProjectModuleRole::Entrypoint,
            "start",
            source,
        ))],
    };
    let project_outcome = compile_project(ProjectCompilationRequest::from_source_modules(
        source_modules,
    ));
    let ProjectCompilationOutcome::Rejected(project_rejection) = project_outcome else {
        assert!(false, "project accepted unavailable {service_name}");
        return;
    };
    assert!(matches!(
        project_rejection.first_problem(),
        ProjectCompilationProblem::SourceModuleRejected { compilation_rejection, .. }
            if compilation_rejection.first_problem().reason() == &expected_reason
    ));
}

fn assert_unknown_service_has_the_service_name_range() {
    let source =
        "fn main() {\n    let unknown: Players = roblox::service::<UnknownService>();\n}\n"
            .to_owned();
    let project_outcome = compile_project(ProjectCompilationRequest::from_source_modules(vec![
        project_module((
            ModuleExecutionSide::Server,
            ProjectModuleRole::Entrypoint,
            "start",
            source.clone(),
        )),
    ]));
    let ProjectCompilationOutcome::Rejected(project_rejection) = project_outcome else {
        assert!(false, "project accepted an unknown service");
        return;
    };
    let ProjectCompilationProblem::SourceModuleRejected {
        compilation_rejection,
        ..
    } = project_rejection.first_problem()
    else {
        assert!(
            false,
            "unknown service produced the wrong project diagnostic"
        );
        return;
    };
    let Some(service_start) = source.rfind("UnknownService") else {
        assert!(false, "unknown-service fixture lost its service name");
        return;
    };
    assert_eq!(
        compilation_rejection.first_problem().reason(),
        &CompilationProblemReason::UnknownRobloxService
    );
    assert_eq!(
        compilation_rejection
            .first_problem()
            .source_range()
            .start_byte(),
        service_start
    );
    assert_eq!(
        compilation_rejection
            .first_problem()
            .source_range()
            .end_byte(),
        service_start + "UnknownService".len()
    );
}

fn assert_annotation_mismatch_has_the_acquisition_range() {
    let source =
        "fn main() {\n    let players: Players = roblox::service::<ReplicatedStorage>();\n}\n";
    let project_outcome = compile_project(ProjectCompilationRequest::from_source_modules(vec![
        project_module((
            ModuleExecutionSide::Server,
            ProjectModuleRole::Entrypoint,
            "start",
            source.to_owned(),
        )),
    ]));
    let ProjectCompilationOutcome::Rejected(project_rejection) = project_outcome else {
        assert!(false, "project accepted a mismatched service annotation");
        return;
    };
    let ProjectCompilationProblem::SourceModuleRejected {
        compilation_rejection,
        ..
    } = project_rejection.first_problem()
    else {
        assert!(
            false,
            "annotation mismatch produced the wrong project diagnostic"
        );
        return;
    };
    let Some(acquisition_start) = source.find("roblox::service") else {
        assert!(false, "annotation-mismatch fixture lost its intrinsic");
        return;
    };
    assert_eq!(
        compilation_rejection.first_problem().reason(),
        &CompilationProblemReason::TypesDoNotMatch
    );
    assert_eq!(
        compilation_rejection
            .first_problem()
            .source_range()
            .start_byte(),
        acquisition_start
    );
}

fn assert_non_intrinsic_service_forms_are_rejected() {
    for source in [
        "fn main() { let players: Players = roblox::service(\"Players\"); }",
        "fn main() { let players: Players = roblox::service::<Players, ReplicatedStorage>(); }",
        "fn main() { let players: Players = roblox::service<Players>(); }",
    ] {
        let project_outcome =
            compile_project(ProjectCompilationRequest::from_source_modules(vec![
                project_module((
                    ModuleExecutionSide::Server,
                    ProjectModuleRole::Entrypoint,
                    "start",
                    source.to_owned(),
                )),
            ]));
        assert!(matches!(
            project_outcome,
            ProjectCompilationOutcome::Rejected(_)
        ));
    }
}

fn assert_standalone_compilation_rejects_service_acquisition() {
    let source = "fn main() { let players: Players = roblox::service::<Players>(); }";
    let CompilationOutcome::Rejected(compilation_rejection) = compile_source(source) else {
        assert!(
            false,
            "standalone compilation accepted Roblox service acquisition"
        );
        return;
    };
    assert_eq!(
        compilation_rejection.first_problem().reason(),
        &CompilationProblemReason::RobloxServiceAcquisitionRequiresProjectCompilation
    );
}

fn assert_service_types_cannot_escape_local_acquisition() {
    for source in [
        "struct Holder { players: Players } fn main() {}",
        "fn receive(players: Players) {} fn main() {}",
        "fn acquire() -> Players { let players: Players = roblox::service::<Players>(); return players; } fn main() {}",
    ] {
        let CompilationOutcome::Rejected(compilation_rejection) = compile_source(source) else {
            assert!(false, "standalone compilation accepted an escaped service type");
            return;
        };
        assert_eq!(
            compilation_rejection.first_problem().reason(),
            &CompilationProblemReason::RobloxServiceTypeMayOnlyBeUsedForLocalAcquisition
        );
    }
}

fn assert_service_members_are_rejected() {
    let source = "fn main() { let players: Players = roblox::service::<Players>(); print(players.LocalPlayer); }";
    let project_outcome = compile_project(ProjectCompilationRequest::from_source_modules(vec![
        project_module((
            ModuleExecutionSide::Server,
            ProjectModuleRole::Entrypoint,
            "start",
            source.to_owned(),
        )),
    ]));
    let ProjectCompilationOutcome::Rejected(project_rejection) = project_outcome else {
        assert!(false, "project accepted service member access");
        return;
    };
    assert!(matches!(
        project_rejection.first_problem(),
        ProjectCompilationProblem::SourceModuleRejected { compilation_rejection, .. }
            if compilation_rejection.first_problem().reason()
                == &CompilationProblemReason::FieldAccessRequiresRecord
    ));
}

fn official_luau_tool(tool_name: (&str, &str)) -> Option<PathBuf> {
    let (environment_variable, executable_name) = tool_name;
    std::env::var_os(environment_variable).map_or_else(
        || {
            let executable_filename = if cfg!(windows) {
                format!("{executable_name}.exe")
            } else {
                executable_name.to_owned()
            };
            let build_directory = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("references")
                .join("checkouts")
                .join("luau")
                .join("build");
            [
                build_directory.join(&executable_filename),
                build_directory.join("release").join(executable_filename),
            ]
            .into_iter()
            .find(|candidate_path| candidate_path.is_file())
        },
        |configured_path| Some(PathBuf::from(configured_path)),
    )
}

fn run_luau_tool(tool_run: (&Path, &Path)) -> Option<Output> {
    let (tool_path, source_path) = tool_run;
    match Command::new(tool_path).arg(source_path).output() {
        Ok(tool_output) => Some(tool_output),
        Err(execution_error) => {
            assert!(
                false,
                "could not invoke {}: {execution_error}",
                tool_path.display()
            );
            None
        }
    }
}

fn temporary_luau_path(label: &str) -> PathBuf {
    let filename = label.replace('/', "-");
    std::env::temp_dir().join(format!(
        "roblox-rust-service-{filename}-{}.luau",
        std::process::id()
    ))
}

fn remove_temporary_luau(path: &Path) {
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(remove_error) => assert!(false, "could not remove {}: {remove_error}", path.display()),
    }
}
