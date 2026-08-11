use std::{fs, path::Path};

use roblox_rust::{
    ProjectCompilationRequest, ProjectModuleIdentity, ProjectModuleRole, ProjectModuleSource,
};

use super::CliError;

pub(super) struct LoadedProject {
    modules: Vec<LoadedModule>,
}

struct LoadedModule {
    identity: ProjectModuleIdentity,
    role: ProjectModuleRole,
    source_text: String,
}

impl LoadedProject {
    pub(super) fn from_file(project_file: &Path) -> Result<Self, CliError> {
        let project_text = fs::read_to_string(project_file).map_err(|error| {
            CliError::input((
                project_file.to_owned(),
                None,
                format!("cannot read project file: {:?}", error.kind()),
            ))
        })?;
        let project_parent = project_file.parent().unwrap_or_else(|| Path::new("."));
        let mut modules = Vec::new();
        for (line_index, line) in project_text.lines().enumerate() {
            let line_number = line_index + 1;
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
                continue;
            }
            let fields = trimmed_line.split_whitespace().collect::<Vec<_>>();
            if fields.len() != 4 {
                return Err(CliError::input((
                    project_file.to_owned(),
                    Some(line_number),
                    "module lines require side, role, module path, and source file".to_owned(),
                )));
            }
            let identity = parse_identity((fields[0], fields[2])).map_err(|message| {
                CliError::input((project_file.to_owned(), Some(line_number), message))
            })?;
            let role = parse_module_role(fields[1]).map_err(|message| {
                CliError::input((project_file.to_owned(), Some(line_number), message))
            })?;
            let source_path = project_parent.join(fields[3]);
            let source_text = fs::read_to_string(&source_path).map_err(|error| {
                CliError::input((
                    source_path.clone(),
                    Some(line_number),
                    format!("cannot read source file: {:?}", error.kind()),
                ))
            })?;
            modules.push(LoadedModule {
                identity,
                role,
                source_text,
            });
        }
        Ok(Self { modules })
    }

    pub(super) fn request(&self) -> ProjectCompilationRequest {
        ProjectCompilationRequest::from_source_modules(
            self.modules
                .iter()
                .map(|module| {
                    ProjectModuleSource::from_source_parts((
                        module.identity.clone(),
                        module.role,
                        module.source_text.clone(),
                    ))
                })
                .collect(),
        )
    }
}

fn parse_identity(identity_parts: (&str, &str)) -> Result<ProjectModuleIdentity, String> {
    let (side_name, module_path) = identity_parts;
    if module_path.is_empty() {
        return Err("module path cannot be empty".to_owned());
    }
    match side_name {
        "server" => Ok(ProjectModuleIdentity::Server {
            module_path: module_path.to_owned(),
        }),
        "client" => Ok(ProjectModuleIdentity::Client {
            module_path: module_path.to_owned(),
        }),
        "shared" => Ok(ProjectModuleIdentity::Shared {
            module_path: module_path.to_owned(),
        }),
        _ => Err(format!(
            "unknown execution side `{side_name}`; expected server, client, or shared"
        )),
    }
}

fn parse_module_role(role_name: &str) -> Result<ProjectModuleRole, String> {
    match role_name {
        "entrypoint" => Ok(ProjectModuleRole::Entrypoint),
        "library" => Ok(ProjectModuleRole::Library),
        _ => Err(format!(
            "unknown module role `{role_name}`; expected entrypoint or library"
        )),
    }
}
