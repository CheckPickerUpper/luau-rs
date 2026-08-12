//! Verifies project imports resolve across Roblox module boundaries before lowering output.

mod support;
use support::{official_luau_tool, run_official_luau_tool_required, temporary_luau_file};

use full_moon::ast::LuaVersion;
use luau_rs::{
    compile_project, compile_source, CompilationOutcome, CompilationProblemReason,
    ProjectCompilationOutcome, ProjectCompilationProblem, ProjectCompilationRequest,
    ProjectModuleIdentity, ProjectModuleRole, ProjectModuleSource,
};

#[test]
fn a_project_lowers_shared_and_server_imports_once_and_exports_public_library_functions() {
    let project_outcome = compile_project(ProjectCompilationRequest::from_source_modules(vec![
        source_module((
            shared_identity("math"),
            ProjectModuleRole::Library,
            "pub fn double(value: number) -> number { return value * 2; }".to_owned(),
        )),
        source_module((
            server_identity("logic"),
            ProjectModuleRole::Library,
            "use crate::shared::math::double;\npub fn answer() -> number { return double(21); }"
                .to_owned(),
        )),
        source_module((
            server_identity("start"),
            ProjectModuleRole::Entrypoint,
            "use crate::server::logic::answer;\nfn main() { print(answer()); }".to_owned(),
        )),
    ]));
    let compiled_project = match project_outcome {
        ProjectCompilationOutcome::Compiled(compiled_project) => compiled_project,
        ProjectCompilationOutcome::Rejected(project_rejection) => {
            assert!(false, "project rejected: {project_rejection:#?}");
            return;
        }
    };
    let generated_modules = compiled_project.generated_modules();
    let Some(shared_math_module) = generated_modules
        .iter()
        .find(|generated_module| generated_module.module_identity() == &shared_identity("math"))
    else {
        assert!(false, "shared math module was not emitted");
        return;
    };
    assert!(shared_math_module
        .generated_luau_text()
        .as_text()
        .contains("return {\n    double = double,\n}\n"));
    let Some(server_logic_module) = generated_modules
        .iter()
        .find(|generated_module| generated_module.module_identity() == &server_identity("logic"))
    else {
        assert!(false, "server logic module was not emitted");
        return;
    };
    let server_logic_text = server_logic_module.generated_luau_text().as_text();
    assert_eq!(
        server_logic_text.matches("require(").count(),
        1,
        "each imported library should load once per importing module"
    );
    assert!(server_logic_text
        .contains("require(game:GetService(\"ReplicatedStorage\"):WaitForChild(\"math\"))"));
    assert!(server_logic_text.contains("local double = __roblox_rust_import_shared_math.double"));
    validate_every_generated_module(generated_modules);
}

#[test]
fn project_import_diagnostics_preserve_function_segment_ranges_and_execution_boundaries() {
    let private_import_source = "use crate::shared::vault::secret;\nfn main() {}";
    let private_project = compile_project(ProjectCompilationRequest::from_source_modules(vec![
        source_module((
            shared_identity("vault"),
            ProjectModuleRole::Library,
            "fn secret() {}".to_owned(),
        )),
        source_module((
            server_identity("start"),
            ProjectModuleRole::Entrypoint,
            private_import_source.to_owned(),
        )),
    ]));
    let Some(private_function_start) = private_import_source.find("secret") else {
        assert!(false, "private import fixture lost its function segment");
        return;
    };
    match private_project {
        ProjectCompilationOutcome::Compiled(_) => {
            assert!(false, "private function import compiled");
        }
        ProjectCompilationOutcome::Rejected(project_rejection) => {
            assert!(matches!(
                project_rejection.first_problem(),
                ProjectCompilationProblem::ImportedFunctionIsPrivate {
                    importing_module_identity: ProjectModuleIdentity::Server { module_path },
                    source_range,
                    ..
                } if module_path == "start"
                    && source_range.start_byte() == private_function_start
                    && source_range.end_byte() == private_function_start + "secret".len()
            ));
        }
    }

    let missing_import_source = "use crate::shared::vault::missing;\nfn main() {}";
    let missing_project = compile_project(ProjectCompilationRequest::from_source_modules(vec![
        source_module((
            shared_identity("vault"),
            ProjectModuleRole::Library,
            "pub fn present() {}".to_owned(),
        )),
        source_module((
            server_identity("start"),
            ProjectModuleRole::Entrypoint,
            missing_import_source.to_owned(),
        )),
    ]));
    let Some(missing_function_start) = missing_import_source.find("missing") else {
        assert!(false, "missing import fixture lost its function segment");
        return;
    };
    match missing_project {
        ProjectCompilationOutcome::Compiled(_) => {
            assert!(false, "missing function import compiled");
        }
        ProjectCompilationOutcome::Rejected(project_rejection) => {
            assert!(matches!(
                project_rejection.first_problem(),
                ProjectCompilationProblem::ImportedFunctionNotFound { source_range, .. }
                    if source_range.start_byte() == missing_function_start
                        && source_range.end_byte() == missing_function_start + "missing".len()
            ));
        }
    }

    let forbidden_side_project =
        compile_project(ProjectCompilationRequest::from_source_modules(vec![
            source_module((
                client_identity("hud"),
                ProjectModuleRole::Library,
                "pub fn title() -> string { return \"HUD\"; }".to_owned(),
            )),
            source_module((
                server_identity("start"),
                ProjectModuleRole::Entrypoint,
                "use crate::client::hud::title;\nfn main() {}".to_owned(),
            )),
        ]));
    match forbidden_side_project {
        ProjectCompilationOutcome::Compiled(_) => assert!(false, "server imported a client module"),
        ProjectCompilationOutcome::Rejected(project_rejection) => {
            assert!(matches!(
                project_rejection.first_problem(),
                ProjectCompilationProblem::ImportExecutionSideNotAllowed {
                    importing_module_identity: ProjectModuleIdentity::Server { module_path },
                    target_module_identity: ProjectModuleIdentity::Client { .. },
                    ..
                } if module_path == "start"
            ));
        }
    }
}

#[test]
fn project_import_cycles_are_closed_deterministically_before_any_module_is_emitted() {
    let project_outcome = compile_project(ProjectCompilationRequest::from_source_modules(vec![
        source_module((
            server_identity("start"),
            ProjectModuleRole::Entrypoint,
            "fn main() {}".to_owned(),
        )),
        source_module((
            shared_identity("a"),
            ProjectModuleRole::Library,
            "use crate::shared::b::b;\npub fn a() { b(); }".to_owned(),
        )),
        source_module((
            shared_identity("b"),
            ProjectModuleRole::Library,
            "use crate::shared::a::a;\npub fn b() { a(); }".to_owned(),
        )),
    ]));
    match project_outcome {
        ProjectCompilationOutcome::Compiled(_) => assert!(false, "cyclic imports compiled"),
        ProjectCompilationOutcome::Rejected(project_rejection) => {
            assert!(matches!(
                project_rejection.first_problem(),
                ProjectCompilationProblem::ImportCycle { cycle_path }
                    if cycle_path == &vec![shared_identity("a"), shared_identity("b"), shared_identity("a")]
            ));
        }
    }
}

#[test]
fn standalone_source_compilation_rejects_project_imports() {
    let compilation_outcome = compile_source("use crate::shared::math::double;\nfn main() {}");
    match compilation_outcome {
        CompilationOutcome::Compiled(_) => {
            assert!(false, "standalone source accepted a project import");
        }
        CompilationOutcome::Rejected(compilation_rejection) => {
            assert!(matches!(
                compilation_rejection.first_problem().reason(),
                CompilationProblemReason::ProjectImportRequiresProjectCompilation
            ));
        }
    }
}

fn source_module(
    source_parts: (ProjectModuleIdentity, ProjectModuleRole, String),
) -> ProjectModuleSource {
    ProjectModuleSource::from_source_parts(source_parts)
}

fn server_identity(module_path: &str) -> ProjectModuleIdentity {
    ProjectModuleIdentity::Server {
        module_path: module_path.to_owned(),
    }
}

fn client_identity(module_path: &str) -> ProjectModuleIdentity {
    ProjectModuleIdentity::Client {
        module_path: module_path.to_owned(),
    }
}

fn shared_identity(module_path: &str) -> ProjectModuleIdentity {
    ProjectModuleIdentity::Shared {
        module_path: module_path.to_owned(),
    }
}

fn validate_every_generated_module(generated_modules: &[luau_rs::GeneratedProjectModule]) {
    let luau_analyze_path = official_luau_tool(("LUAU_ANALYZE_BIN", "luau-analyze"));
    for (module_index, generated_module) in generated_modules.iter().enumerate() {
        let generated_luau = generated_module.generated_luau_text().as_text();
        insta::assert_snapshot!(format!("module-{module_index}"), generated_luau);
        assert!(
            full_moon::parse_fallible(generated_luau, LuaVersion::luau())
                .into_result()
                .is_ok(),
            "Full Moon rejected {}",
            generated_module.output_path().as_str()
        );
        let generated_luau_path =
            temporary_luau_file(&format!("luau-rs-project-import-{module_index}"));
        let analyzer_harness = format!(
            "--!strict\nlocal game: any = {{}}\nlocal require: any = function(...) return nil end\n{generated_luau}"
        );
        match std::fs::write(generated_luau_path.path(), analyzer_harness) {
            Ok(()) => {}
            Err(write_error) => {
                assert!(false, "could not write generated module: {write_error}");
                return;
            }
        }
        let analyzer_output =
            run_official_luau_tool_required((&luau_analyze_path, generated_luau_path.path()));
        assert!(
            analyzer_output.status.success(),
            "luau-analyze rejected {}:\n{}",
            generated_module.output_path().as_str(),
            String::from_utf8_lossy(&analyzer_output.stderr)
        );
    }
}
