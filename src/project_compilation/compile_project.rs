use crate::{
    checked_program::{
        check_project_entrypoint, check_project_library, CheckedValueType,
        ImportedFunctionSignature,
    },
    generated_luau::{generate_luau_library, generate_luau_program, write_luau_text},
    source_language::{
        parse_source_program, split_source_into_tokens, ParsedFunction, ParsedFunctionVisibility,
        ParsedProgram,
    },
    CompiledProject, GeneratedLuauText, GeneratedProjectModule, ModuleExecutionSide,
    ProjectCompilationOutcome, ProjectCompilationProblem, ProjectCompilationRejection,
    ProjectCompilationRequest, ProjectModuleIdentity, ProjectModuleRole, ProjectModuleSource,
    ProjectOutputPath,
};

/// Compiles a complete project after resolving all module dependencies before any output is accepted.
#[must_use]
pub fn compile_project(project_request: ProjectCompilationRequest) -> ProjectCompilationOutcome {
    let mut source_modules = project_request.into_source_modules();
    source_modules.sort_by(|left_module, right_module| {
        left_module
            .module_identity()
            .cmp(right_module.module_identity())
    });

    match validate_project_modules(&source_modules) {
        Ok(()) => {}
        Err(project_problem) => return rejected(project_problem),
    }
    let parsed_modules = match parse_project_modules(source_modules) {
        Ok(parsed_modules) => parsed_modules,
        Err(project_problem) => return rejected(project_problem),
    };
    let resolved_modules = match resolve_project_imports(&parsed_modules) {
        Ok(resolved_modules) => resolved_modules,
        Err(project_problem) => return rejected(project_problem),
    };
    match reject_import_cycles(&resolved_modules) {
        Ok(()) => {}
        Err(project_problem) => return rejected(project_problem),
    }
    let generated_modules = match check_and_lower_modules(resolved_modules) {
        Ok(generated_modules) => generated_modules,
        Err(project_problem) => return rejected(project_problem),
    };
    ProjectCompilationOutcome::Compiled(CompiledProject::from_generated_modules(generated_modules))
}

fn validate_project_modules(
    source_modules: &[ProjectModuleSource],
) -> Result<(), ProjectCompilationProblem> {
    match project_has_entrypoint(source_modules) {
        ProjectEntrypointPresence::Present => {}
        ProjectEntrypointPresence::Absent => {
            return Err(ProjectCompilationProblem::MissingEntrypointModule)
        }
    }
    let mut previous_module_identity = None;
    for source_module in source_modules {
        let module_identity = source_module.module_identity();
        match validate_module_identity(module_identity) {
            ModuleIdentityValidity::Valid => {}
            ModuleIdentityValidity::Invalid => {
                return Err(ProjectCompilationProblem::InvalidModuleIdentity {
                    module_identity: module_identity.clone(),
                });
            }
        }
        match &previous_module_identity {
            Some(previous_identity) if previous_identity == module_identity => {
                return Err(ProjectCompilationProblem::DuplicateModuleIdentity {
                    module_identity: module_identity.clone(),
                });
            }
            Some(_) | None => {}
        }
        if module_identity
            .output_path_text(source_module.module_role())
            .is_none()
        {
            return Err(ProjectCompilationProblem::SharedModuleCannotBeEntrypoint {
                module_identity: module_identity.clone(),
            });
        }
        previous_module_identity = Some(module_identity.clone());
    }
    Ok(())
}

fn parse_project_modules(
    source_modules: Vec<ProjectModuleSource>,
) -> Result<Vec<ParsedProjectModule>, ProjectCompilationProblem> {
    let mut parsed_modules = Vec::new();
    for source_module in source_modules {
        let module_identity = source_module.module_identity().clone();
        let source_tokens = match split_source_into_tokens(source_module.source_text()) {
            Ok(source_tokens) => source_tokens,
            Err(compilation_problem) => {
                return Err(ProjectCompilationProblem::SourceModuleRejected {
                    module_identity,
                    compilation_rejection: crate::CompilationRejection::from_problem(
                        compilation_problem,
                    ),
                });
            }
        };
        let parsed_program = match parse_source_program(source_tokens) {
            Ok(parsed_program) => parsed_program,
            Err(compilation_problem) => {
                return Err(ProjectCompilationProblem::SourceModuleRejected {
                    module_identity,
                    compilation_rejection: crate::CompilationRejection::from_problem(
                        compilation_problem,
                    ),
                });
            }
        };
        parsed_modules.push(ParsedProjectModule {
            source_module,
            parsed_program,
        });
    }
    Ok(parsed_modules)
}

fn resolve_project_imports(
    parsed_modules: &[ParsedProjectModule],
) -> Result<Vec<ResolvedProjectModule<'_>>, ProjectCompilationProblem> {
    let mut resolved_modules = Vec::new();
    for importing_module in parsed_modules {
        let importing_identity = importing_module.source_module.module_identity();
        let mut resolved_imports = Vec::new();
        for parsed_import in importing_module.parsed_program.parsed_imports() {
            let Some(target_module) = parsed_modules.iter().find(|candidate_module| {
                candidate_module.source_module.module_identity()
                    == parsed_import.target_module_identity()
            }) else {
                return Err(ProjectCompilationProblem::ImportedModuleNotFound {
                    importing_module_identity: importing_identity.clone(),
                    target_module_identity: parsed_import.target_module_identity().clone(),
                    source_range: parsed_import.import_range(),
                });
            };
            if target_module.source_module.module_role() != ProjectModuleRole::Library {
                return Err(ProjectCompilationProblem::ImportedModuleIsEntrypoint {
                    importing_module_identity: importing_identity.clone(),
                    target_module_identity: parsed_import.target_module_identity().clone(),
                    source_range: parsed_import.import_range(),
                });
            }
            if !import_is_legal_for_side((
                importing_identity.execution_side(),
                parsed_import.target_module_identity().execution_side(),
            )) {
                return Err(ProjectCompilationProblem::ImportExecutionSideNotAllowed {
                    importing_module_identity: importing_identity.clone(),
                    target_module_identity: parsed_import.target_module_identity().clone(),
                    source_range: parsed_import.import_range(),
                });
            }
            let Some(imported_function) = target_module
                .parsed_program
                .parsed_functions()
                .iter()
                .find(|function| {
                    function.function_name() == parsed_import.imported_function_name()
                })
            else {
                return Err(ProjectCompilationProblem::ImportedFunctionNotFound {
                    importing_module_identity: importing_identity.clone(),
                    target_module_identity: parsed_import.target_module_identity().clone(),
                    function_name: parsed_import.imported_function_name().to_owned(),
                    source_range: parsed_import.imported_function_range(),
                });
            };
            if imported_function.visibility() != ParsedFunctionVisibility::Public {
                return Err(ProjectCompilationProblem::ImportedFunctionIsPrivate {
                    importing_module_identity: importing_identity.clone(),
                    target_module_identity: parsed_import.target_module_identity().clone(),
                    function_name: parsed_import.imported_function_name().to_owned(),
                    source_range: parsed_import.imported_function_range(),
                });
            }
            if importing_module
                .parsed_program
                .parsed_functions()
                .iter()
                .any(|function| function.function_name() == parsed_import.imported_function_name())
                || resolved_imports
                    .iter()
                    .any(|resolved_import: &ResolvedProjectImport| {
                        resolved_import.function_name == parsed_import.imported_function_name()
                    })
            {
                return Err(
                    ProjectCompilationProblem::ImportNameCollidesWithLocalDeclaration {
                        importing_module_identity: importing_identity.clone(),
                        function_name: parsed_import.imported_function_name().to_owned(),
                        source_range: parsed_import.imported_function_range(),
                    },
                );
            }
            resolved_imports.push(ResolvedProjectImport {
                target_module_identity: parsed_import.target_module_identity().clone(),
                function_name: parsed_import.imported_function_name().to_owned(),
                signature: signature_from_function(imported_function),
            });
        }
        resolved_modules.push(ResolvedProjectModule {
            parsed_module: importing_module,
            resolved_imports,
        });
    }
    Ok(resolved_modules)
}

fn reject_import_cycles(
    resolved_modules: &[ResolvedProjectModule<'_>],
) -> Result<(), ProjectCompilationProblem> {
    let mut visited_module_identities = Vec::new();
    let mut active_module_path = Vec::new();
    for resolved_module in resolved_modules {
        let module_identity = resolved_module
            .parsed_module
            .source_module
            .module_identity();
        if visited_module_identities.contains(module_identity) {
            continue;
        }
        match visit_module_for_cycle((
            module_identity,
            resolved_modules,
            &mut visited_module_identities,
            &mut active_module_path,
        )) {
            CycleSearchOutcome::NoCycle => {}
            CycleSearchOutcome::Cycle(cycle_path) => {
                return Err(ProjectCompilationProblem::ImportCycle { cycle_path });
            }
        }
    }
    Ok(())
}

fn visit_module_for_cycle(
    cycle_search: (
        &ProjectModuleIdentity,
        &[ResolvedProjectModule<'_>],
        &mut Vec<ProjectModuleIdentity>,
        &mut Vec<ProjectModuleIdentity>,
    ),
) -> CycleSearchOutcome {
    let (module_identity, resolved_modules, visited_module_identities, active_module_path) =
        cycle_search;
    if let Some(cycle_start) = active_module_path
        .iter()
        .position(|active_identity| active_identity == module_identity)
    {
        let mut cycle_path = active_module_path[cycle_start..].to_vec();
        cycle_path.push(module_identity.clone());
        return CycleSearchOutcome::Cycle(cycle_path);
    }
    if visited_module_identities.contains(module_identity) {
        return CycleSearchOutcome::NoCycle;
    }
    active_module_path.push(module_identity.clone());
    let Some(resolved_module) = resolved_modules.iter().find(|candidate_module| {
        candidate_module
            .parsed_module
            .source_module
            .module_identity()
            == module_identity
    }) else {
        return CycleSearchOutcome::NoCycle;
    };
    let mut dependency_identities: Vec<_> = resolved_module
        .resolved_imports
        .iter()
        .map(|resolved_import| resolved_import.target_module_identity.clone())
        .collect();
    dependency_identities.sort();
    dependency_identities.dedup();
    for dependency_identity in dependency_identities {
        match visit_module_for_cycle((
            &dependency_identity,
            resolved_modules,
            visited_module_identities,
            active_module_path,
        )) {
            CycleSearchOutcome::NoCycle => {}
            CycleSearchOutcome::Cycle(cycle_path) => return CycleSearchOutcome::Cycle(cycle_path),
        }
    }
    active_module_path.pop();
    visited_module_identities.push(module_identity.clone());
    CycleSearchOutcome::NoCycle
}

fn check_and_lower_modules(
    resolved_modules: Vec<ResolvedProjectModule<'_>>,
) -> Result<Vec<GeneratedProjectModule>, ProjectCompilationProblem> {
    let mut generated_modules = Vec::new();
    for resolved_module in resolved_modules {
        let source_module = &resolved_module.parsed_module.source_module;
        let imported_signatures: Vec<_> = resolved_module
            .resolved_imports
            .iter()
            .map(|resolved_import| resolved_import.signature.clone())
            .collect();
        let checked_program = match source_module.module_role() {
            ProjectModuleRole::Entrypoint => check_project_entrypoint((
                &resolved_module.parsed_module.parsed_program,
                &imported_signatures,
                source_module.module_identity().execution_side(),
            )),
            ProjectModuleRole::Library => check_project_library((
                &resolved_module.parsed_module.parsed_program,
                &imported_signatures,
                source_module.module_identity().execution_side(),
            )),
        };
        let checked_program = match checked_program {
            Ok(checked_program) => checked_program,
            Err(compilation_problem) => {
                return Err(ProjectCompilationProblem::SourceModuleRejected {
                    module_identity: source_module.module_identity().clone(),
                    compilation_rejection: crate::CompilationRejection::from_problem(
                        compilation_problem,
                    ),
                });
            }
        };
        let generated_luau_text = match source_module.module_role() {
            ProjectModuleRole::Entrypoint => {
                write_luau_text(&generate_luau_program(&checked_program))
            }
            ProjectModuleRole::Library => write_luau_text(&generate_luau_library(&checked_program)),
        };
        let generated_luau_text = compose_project_module_text((
            generated_luau_text,
            source_module.module_role(),
            &resolved_module.parsed_module.parsed_program,
            &resolved_module.resolved_imports,
        ));
        let Some(output_path_text) = source_module
            .module_identity()
            .output_path_text(source_module.module_role())
        else {
            return Err(ProjectCompilationProblem::SharedModuleCannotBeEntrypoint {
                module_identity: source_module.module_identity().clone(),
            });
        };
        generated_modules.push(GeneratedProjectModule::from_generated_parts((
            source_module.module_identity().clone(),
            ProjectOutputPath::from_path_text(output_path_text),
            generated_luau_text,
        )));
    }
    Ok(generated_modules)
}

fn compose_project_module_text(
    module_text_parts: (
        GeneratedLuauText,
        ProjectModuleRole,
        &ParsedProgram,
        &[ResolvedProjectImport],
    ),
) -> GeneratedLuauText {
    let (generated_luau_text, module_role, parsed_program, resolved_imports) = module_text_parts;
    let mut module_text = generated_luau_text.into_text();
    let import_text = write_import_text((parsed_program, resolved_imports));
    if !import_text.is_empty() {
        let generated_body = module_text
            .strip_prefix("--!strict\n\n")
            .map_or(module_text.as_str(), |generated_body| generated_body);
        module_text = format!("--!strict\n\n{import_text}\n{generated_body}");
    }
    if module_role == ProjectModuleRole::Library {
        module_text.push_str(&write_library_exports(parsed_program));
    }
    GeneratedLuauText::from_text(module_text)
}

fn write_import_text(import_text_parts: (&ParsedProgram, &[ResolvedProjectImport])) -> String {
    let (parsed_program, resolved_imports) = import_text_parts;
    if resolved_imports.is_empty() {
        return String::new();
    }
    let mut target_module_identities: Vec<_> = resolved_imports
        .iter()
        .map(|resolved_import| resolved_import.target_module_identity.clone())
        .collect();
    target_module_identities.sort();
    target_module_identities.dedup();
    let mut occupied_names: Vec<_> = parsed_program
        .parsed_functions()
        .iter()
        .map(|function| function.function_name().to_owned())
        .collect();
    occupied_names.extend(
        resolved_imports
            .iter()
            .map(|resolved_import| resolved_import.function_name.clone()),
    );
    let mut target_bindings = Vec::new();
    let mut import_text = String::new();
    for target_module_identity in target_module_identities {
        let binding_name = unique_import_binding_name((&target_module_identity, &occupied_names));
        occupied_names.push(binding_name.clone());
        import_text.push_str("local ");
        import_text.push_str(&binding_name);
        import_text.push_str(" = require(");
        import_text.push_str(&require_target_expression(&target_module_identity));
        import_text.push_str(")\n");
        target_bindings.push((target_module_identity, binding_name));
    }
    for resolved_import in resolved_imports {
        let Some((_, target_binding)) = target_bindings.iter().find(|(target_identity, _)| {
            target_identity == &resolved_import.target_module_identity
        }) else {
            continue;
        };
        import_text.push_str("local ");
        import_text.push_str(&resolved_import.function_name);
        import_text.push_str(" = ");
        import_text.push_str(target_binding);
        import_text.push('.');
        import_text.push_str(&resolved_import.function_name);
        import_text.push('\n');
    }
    import_text
}

fn unique_import_binding_name(binding_parts: (&ProjectModuleIdentity, &[String])) -> String {
    let (target_module_identity, occupied_names) = binding_parts;
    let side_name = match target_module_identity.execution_side() {
        ModuleExecutionSide::Server => "server",
        ModuleExecutionSide::Client => "client",
        ModuleExecutionSide::Shared => "shared",
    };
    let mut binding_name = format!(
        "__roblox_rust_import_{side_name}_{}",
        target_module_identity.module_path().replace('/', "__")
    );
    while occupied_names
        .iter()
        .any(|occupied_name| occupied_name == &binding_name)
    {
        binding_name.push('_');
    }
    binding_name
}

fn require_target_expression(target_module_identity: &ProjectModuleIdentity) -> String {
    let mut target_expression = match target_module_identity.execution_side() {
        ModuleExecutionSide::Server => "game:GetService(\"ServerScriptService\")".to_owned(),
        ModuleExecutionSide::Client => {
            "game:GetService(\"Players\").LocalPlayer:WaitForChild(\"PlayerScripts\")".to_owned()
        }
        ModuleExecutionSide::Shared => "game:GetService(\"ReplicatedStorage\")".to_owned(),
    };
    for path_segment in target_module_identity.module_path().split('/') {
        target_expression.push_str(":WaitForChild(\"");
        target_expression.push_str(path_segment);
        target_expression.push_str("\")");
    }
    target_expression
}

fn write_library_exports(parsed_program: &ParsedProgram) -> String {
    let public_functions: Vec<_> = parsed_program
        .parsed_functions()
        .iter()
        .filter(|function| function.visibility() == ParsedFunctionVisibility::Public)
        .collect();
    let mut export_text = String::from("return {\n");
    for public_function in public_functions {
        export_text.push_str("    ");
        export_text.push_str(public_function.function_name());
        export_text.push_str(" = ");
        export_text.push_str(public_function.function_name());
        export_text.push_str(",\n");
    }
    export_text.push_str("}\n");
    export_text
}

fn signature_from_function(parsed_function: &ParsedFunction) -> ImportedFunctionSignature {
    ImportedFunctionSignature::from_parts((
        parsed_function.function_name().to_owned(),
        parsed_function
            .function_parameters()
            .iter()
            .map(|parameter| checked_value_type(parameter.value_type()))
            .collect(),
        checked_value_type(parsed_function.returned_value_type()),
    ))
}

fn checked_value_type(
    parsed_value_type: crate::source_language::ParsedValueType,
) -> CheckedValueType {
    match parsed_value_type {
        crate::source_language::ParsedValueType::Number => CheckedValueType::Number,
        crate::source_language::ParsedValueType::String => CheckedValueType::String,
        crate::source_language::ParsedValueType::Boolean => CheckedValueType::Boolean,
        crate::source_language::ParsedValueType::Array(element_type) => {
            CheckedValueType::Array(Box::new(checked_value_type(*element_type)))
        }
        crate::source_language::ParsedValueType::NoReturnedValues => {
            CheckedValueType::NoReturnedValues
        }
        crate::source_language::ParsedValueType::NamedRecord { record_name, .. } => {
            CheckedValueType::NamedRecord(record_name)
        }
    }
}

const fn import_is_legal_for_side(side_pair: (ModuleExecutionSide, ModuleExecutionSide)) -> bool {
    match side_pair {
        (
            ModuleExecutionSide::Server,
            ModuleExecutionSide::Server | ModuleExecutionSide::Shared,
        )
        | (
            ModuleExecutionSide::Client,
            ModuleExecutionSide::Client | ModuleExecutionSide::Shared,
        )
        | (ModuleExecutionSide::Shared, ModuleExecutionSide::Shared) => true,
        (ModuleExecutionSide::Server, ModuleExecutionSide::Client)
        | (ModuleExecutionSide::Client, ModuleExecutionSide::Server)
        | (
            ModuleExecutionSide::Shared,
            ModuleExecutionSide::Server | ModuleExecutionSide::Client,
        ) => false,
    }
}

fn project_has_entrypoint(source_modules: &[ProjectModuleSource]) -> ProjectEntrypointPresence {
    match source_modules
        .iter()
        .find(|source_module| source_module.module_role() == ProjectModuleRole::Entrypoint)
    {
        Some(_) => ProjectEntrypointPresence::Present,
        None => ProjectEntrypointPresence::Absent,
    }
}

fn validate_module_identity(module_identity: &ProjectModuleIdentity) -> ModuleIdentityValidity {
    if module_identity
        .module_path()
        .split('/')
        .all(module_path_segment_is_valid)
    {
        ModuleIdentityValidity::Valid
    } else {
        ModuleIdentityValidity::Invalid
    }
}

fn module_path_segment_is_valid(module_path_segment: &str) -> bool {
    !module_path_segment.is_empty()
        && module_path_segment
            .chars()
            .all(|path_character| path_character.is_ascii_alphanumeric() || path_character == '_')
}

const fn rejected(project_problem: ProjectCompilationProblem) -> ProjectCompilationOutcome {
    ProjectCompilationOutcome::Rejected(ProjectCompilationRejection::from_problem(project_problem))
}

struct ParsedProjectModule {
    source_module: ProjectModuleSource,
    parsed_program: ParsedProgram,
}

struct ResolvedProjectModule<'source> {
    parsed_module: &'source ParsedProjectModule,
    resolved_imports: Vec<ResolvedProjectImport>,
}

struct ResolvedProjectImport {
    target_module_identity: ProjectModuleIdentity,
    function_name: String,
    signature: ImportedFunctionSignature,
}

enum CycleSearchOutcome {
    NoCycle,
    Cycle(Vec<ProjectModuleIdentity>),
}

enum ProjectEntrypointPresence {
    Present,
    Absent,
}

enum ModuleIdentityValidity {
    Valid,
    Invalid,
}
