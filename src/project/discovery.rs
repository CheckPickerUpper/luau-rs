//! Convention-based recursive discovery of wasm project modules.

use super::{
    ProjectCompilationRequest, ProjectManifest, ProjectModuleIdentity, ProjectModuleRole,
    ProjectModuleSource,
};
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

const MODULE_SIDE_COMPONENT_INDEX: usize = 0;
const MODULE_ROLE_COMPONENT_INDEX: usize = 1;
const MODULE_PATH_START_COMPONENT: usize = 2;
const MINIMUM_MODULE_PATH_COMPONENTS: usize = MODULE_PATH_START_COMPONENT + 1;

/// Explains why a manifest source tree could not become a compilation request.
#[derive(Debug, Error)]
pub enum ProjectDiscoveryProblem {
    /// A directory entry could not be inspected while walking the source root.
    #[error("could not walk project source root {root}: {source}")]
    Walk {
        /// The configured source root.
        root: PathBuf,
        /// The traversal failure and its source path.
        #[source]
        source: walkdir::Error,
    },
    /// A discovered wasm file could not be read.
    #[error("could not read wasm module {path}: {source}")]
    ReadModule {
        /// The file that was discovered.
        path: PathBuf,
        /// The filesystem failure returned by the operating system.
        #[source]
        source: std::io::Error,
    },
    /// A wasm file did not follow the documented source-tree convention.
    #[error("wasm module {path} must be below <server|client|shared>/<entrypoint|library>/; found {component_count} path components")]
    InvalidModulePath {
        /// The path relative to the configured source root.
        path: PathBuf,
        /// The number of components that established the invalid layout.
        component_count: usize,
    },
    /// A source-tree directory used an unsupported execution side.
    #[error(
        "wasm module {path} uses an unknown execution side; expected server, client, or shared"
    )]
    UnknownExecutionSide {
        /// The path relative to the configured source root.
        path: PathBuf,
    },
    /// A source-tree directory used an unsupported module role.
    #[error("wasm module {path} uses an unknown module role; expected entrypoint or library")]
    UnknownModuleRole {
        /// The path relative to the configured source root.
        path: PathBuf,
    },
    /// No wasm input was found beneath the configured source root.
    #[error("project source root {root} contains no .wasm modules (found {module_count} modules)")]
    EmptySourceRoot {
        /// The configured source root.
        root: PathBuf,
        /// The discovered module count that established the empty result.
        module_count: usize,
    },
}

/// Discovers convention-named wasm files in deterministic filesystem order.
///
/// Every module must be below `<side>/<role>/` beneath the manifest's source
/// root. The remaining path becomes the slash-separated project module path;
/// for example, `server/entrypoint/game/main.wasm` becomes the server
/// entrypoint `game/main`.
///
/// # Errors
///
/// Returns a typed discovery problem when traversal, input reading, or path
/// interpretation fails.
pub fn discover_project_request(
    manifest: &ProjectManifest,
) -> Result<ProjectCompilationRequest, ProjectDiscoveryProblem> {
    let source_root = manifest.source_root();
    let mut source_modules = Vec::new();
    for entry_result in WalkDir::new(source_root).min_depth(1).sort_by_file_name() {
        let entry = match entry_result {
            Ok(entry) => entry,
            Err(source) => {
                return Err(ProjectDiscoveryProblem::Walk {
                    root: source_root.to_path_buf(),
                    source,
                });
            }
        };
        if !entry.file_type().is_file() || !is_wasm_path(entry.path()) {
            continue;
        }
        let relative_path = match entry.path().strip_prefix(source_root) {
            Ok(relative_path) => relative_path.to_path_buf(),
            Err(_) => {
                return Err(ProjectDiscoveryProblem::InvalidModulePath {
                    path: entry.path().to_path_buf(),
                    component_count: 0,
                });
            }
        };
        let (module_identity, module_role) = parse_module_layout(&relative_path)?;
        let wasm_bytes = match fs_err::read(entry.path()) {
            Ok(wasm_bytes) => wasm_bytes,
            Err(source) => {
                return Err(ProjectDiscoveryProblem::ReadModule {
                    path: entry.path().to_path_buf(),
                    source,
                });
            }
        };
        source_modules.push(ProjectModuleSource::from_wasm_parts((
            module_identity,
            module_role,
            wasm_bytes,
        )));
    }

    if source_modules.is_empty() {
        return Err(ProjectDiscoveryProblem::EmptySourceRoot {
            root: source_root.to_path_buf(),
            module_count: source_modules.len(),
        });
    }
    Ok(ProjectCompilationRequest::from_source_modules(
        source_modules,
    ))
}

fn is_wasm_path(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("wasm")
}

fn parse_module_layout(
    relative_path: &Path,
) -> Result<(ProjectModuleIdentity, ProjectModuleRole), ProjectDiscoveryProblem> {
    let path_text = relative_path.to_path_buf();
    let components = relative_path
        .iter()
        .map(|component| {
            component.to_str().map_or_else(
                || {
                    Err(ProjectDiscoveryProblem::InvalidModulePath {
                        path: path_text.clone(),
                        component_count: relative_path.components().count(),
                    })
                },
                |component| Ok(component.to_owned()),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    if components.len() < MINIMUM_MODULE_PATH_COMPONENTS {
        return Err(ProjectDiscoveryProblem::InvalidModulePath {
            path: path_text,
            component_count: components.len(),
        });
    }

    let side = match components
        .get(MODULE_SIDE_COMPONENT_INDEX)
        .map(String::as_str)
    {
        Some("server") => ProjectModuleIdentity::Server {
            module_path: module_path_text(&components[MODULE_PATH_START_COMPONENT..])?,
        },
        Some("client") => ProjectModuleIdentity::Client {
            module_path: module_path_text(&components[MODULE_PATH_START_COMPONENT..])?,
        },
        Some("shared") => ProjectModuleIdentity::Shared {
            module_path: module_path_text(&components[MODULE_PATH_START_COMPONENT..])?,
        },
        Some(_) => {
            return Err(ProjectDiscoveryProblem::UnknownExecutionSide { path: path_text });
        }
        None => {
            return Err(ProjectDiscoveryProblem::InvalidModulePath {
                path: path_text,
                component_count: components.len(),
            })
        }
    };
    let role = match components
        .get(MODULE_ROLE_COMPONENT_INDEX)
        .map(String::as_str)
    {
        Some("entrypoint") => ProjectModuleRole::Entrypoint,
        Some("library") => ProjectModuleRole::Library,
        Some(_) => {
            return Err(ProjectDiscoveryProblem::UnknownModuleRole {
                path: relative_path.to_path_buf(),
            });
        }
        None => {
            return Err(ProjectDiscoveryProblem::InvalidModulePath {
                path: relative_path.to_path_buf(),
                component_count: components.len(),
            });
        }
    };
    Ok((side, role))
}

fn module_path_text(components: &[String]) -> Result<String, ProjectDiscoveryProblem> {
    let Some(last_component) = components.last() else {
        return Err(ProjectDiscoveryProblem::InvalidModulePath {
            path: PathBuf::new(),
            component_count: components.len(),
        });
    };
    let module_name = match Path::new(last_component).file_stem() {
        Some(module_name) => match module_name.to_str() {
            Some(module_name) if !module_name.is_empty() => module_name.to_owned(),
            Some(_) | None => {
                return Err(ProjectDiscoveryProblem::InvalidModulePath {
                    path: PathBuf::from(last_component),
                    component_count: components.len(),
                });
            }
        },
        None => {
            return Err(ProjectDiscoveryProblem::InvalidModulePath {
                path: PathBuf::from(last_component),
                component_count: components.len(),
            });
        }
    };
    let last_component_index = components.len() - 1;
    let mut module_segments = components[..last_component_index].to_vec();
    module_segments.push(module_name);
    Ok(module_segments.join("/"))
}
