//! Deterministic discovery from a Roblox-shaped project tree.

use super::{
    ProjectCompilationRequest, ProjectManifest, ProjectModuleIdentity, ProjectModuleSource,
    ProjectSourceAsset, RobloxService,
};
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

/// Explains why a Roblox-shaped source tree could not become a compilation request.
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
    /// A retained Luau or Roblox model asset could not be read.
    #[error("could not read project asset {path}: {source}")]
    ReadAsset {
        /// The asset that was discovered.
        path: PathBuf,
        /// The filesystem failure returned by the operating system.
        #[source]
        source: std::io::Error,
    },
    /// A source path was not below one recognized Roblox service.
    #[error("project path {path} uses unknown Roblox service {service:?}")]
    UnknownRobloxService {
        /// The offending source path.
        path: PathBuf,
        /// The unrecognized first path component.
        service: String,
    },
    /// A recognized service was used without its required child container.
    #[error("project path {path} is not a legal path beneath a Roblox service")]
    InvalidServicePath {
        /// The offending source path.
        path: PathBuf,
    },
    /// A project file did not have enough path components to identify a module.
    #[error("project path {path} must name a file beneath a recognized Roblox service")]
    InvalidModulePath {
        /// The path relative to the configured source root.
        path: PathBuf,
    },
    /// No supported source module or retained asset was found.
    #[error("project source root {root} contains no supported Roblox project files")]
    EmptySourceRoot {
        /// The configured source root.
        root: PathBuf,
    },
}

/// Discovers wasm modules and retained assets in deterministic Roblox service order.
///
/// # Errors
///
/// Returns a typed problem when a service path is unknown, a supported file
/// cannot be read, or the source root contains no supported project files.
pub fn discover_project_request(
    manifest: &ProjectManifest,
) -> Result<ProjectCompilationRequest, ProjectDiscoveryProblem> {
    let source_root = manifest.source_root();
    let mut source_modules = Vec::new();
    let mut source_assets = Vec::new();
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
        let relative_path = match entry.path().strip_prefix(source_root) {
            Ok(relative_path) => relative_path.to_path_buf(),
            Err(_) => {
                return Err(ProjectDiscoveryProblem::InvalidModulePath {
                    path: entry.path().to_path_buf(),
                });
            }
        };
        if entry.depth() == 1 && entry.file_type().is_dir() {
            validate_service_root(&relative_path)?;
            continue;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let (service, content_start) = parse_service_layout(&relative_path)?;
        if is_wasm_path(entry.path()) {
            let module_path = module_path_text(&relative_path, content_start)?;
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
                ProjectModuleIdentity::RobloxService {
                    service,
                    module_path,
                },
                service.module_role(),
                wasm_bytes,
            )));
        } else if is_project_asset_path(entry.path()) {
            let bytes = match fs_err::read(entry.path()) {
                Ok(bytes) => bytes,
                Err(source) => {
                    return Err(ProjectDiscoveryProblem::ReadAsset {
                        path: entry.path().to_path_buf(),
                        source,
                    });
                }
            };
            source_assets.push(ProjectSourceAsset {
                relative_path,
                bytes,
            });
        } else if content_start == 0 {
            return Err(ProjectDiscoveryProblem::InvalidModulePath {
                path: relative_path,
            });
        }
    }

    if source_modules.is_empty() && source_assets.is_empty() {
        return Err(ProjectDiscoveryProblem::EmptySourceRoot {
            root: source_root.to_path_buf(),
        });
    }
    Ok(ProjectCompilationRequest::from_discovered(
        source_modules,
        source_assets,
    ))
}

fn validate_service_root(relative_path: &Path) -> Result<(), ProjectDiscoveryProblem> {
    let components = path_components(relative_path)?;
    match components.first().map(String::as_str) {
        Some(
            "ServerScriptService"
            | "ServerStorage"
            | "ReplicatedStorage"
            | "Workspace"
            | "ReplicatedFirst"
            | "StarterGui"
            | "StarterPlayer",
        ) => Ok(()),
        Some(service) => Err(ProjectDiscoveryProblem::UnknownRobloxService {
            path: relative_path.to_path_buf(),
            service: service.to_owned(),
        }),
        None => Err(ProjectDiscoveryProblem::InvalidModulePath {
            path: relative_path.to_path_buf(),
        }),
    }
}

fn parse_service_layout(
    relative_path: &Path,
) -> Result<(RobloxService, usize), ProjectDiscoveryProblem> {
    let components = path_components(relative_path)?;
    let Some(root) = components.first().map(String::as_str) else {
        return Err(ProjectDiscoveryProblem::InvalidModulePath {
            path: relative_path.to_path_buf(),
        });
    };
    match root {
        "ServerScriptService" => Ok((RobloxService::ServerScriptService, 1)),
        "ServerStorage" => Ok((RobloxService::ServerStorage, 1)),
        "ReplicatedStorage" => Ok((RobloxService::ReplicatedStorage, 1)),
        "Workspace" => Ok((RobloxService::Workspace, 1)),
        "ReplicatedFirst" => Ok((RobloxService::ReplicatedFirst, 1)),
        "StarterGui" => Ok((RobloxService::StarterGui, 1)),
        "StarterPlayer" => match components.get(1).map(String::as_str) {
            Some("StarterPlayerScripts") => Ok((RobloxService::StarterPlayerScripts, 2)),
            Some("StarterCharacterScripts") => Ok((RobloxService::StarterCharacterScripts, 2)),
            Some(_) | None => Err(ProjectDiscoveryProblem::InvalidServicePath {
                path: relative_path.to_path_buf(),
            }),
        },
        service => Err(ProjectDiscoveryProblem::UnknownRobloxService {
            path: relative_path.to_path_buf(),
            service: service.to_owned(),
        }),
    }
}

fn path_components(path: &Path) -> Result<Vec<String>, ProjectDiscoveryProblem> {
    path.iter()
        .map(|component| {
            component.to_str().map_or_else(
                || {
                    Err(ProjectDiscoveryProblem::InvalidModulePath {
                        path: path.to_path_buf(),
                    })
                },
                |component| Ok(component.to_owned()),
            )
        })
        .collect()
}

fn module_path_text(
    relative_path: &Path,
    content_start: usize,
) -> Result<String, ProjectDiscoveryProblem> {
    let components = path_components(relative_path)?;
    if components.len() <= content_start {
        return Err(ProjectDiscoveryProblem::InvalidModulePath {
            path: relative_path.to_path_buf(),
        });
    }
    let Some(last_component) = components.last() else {
        return Err(ProjectDiscoveryProblem::InvalidModulePath {
            path: relative_path.to_path_buf(),
        });
    };
    let module_name = match Path::new(last_component).file_stem() {
        Some(module_name) if !module_name.is_empty() => match module_name.to_str() {
            Some(module_name) => module_name.to_owned(),
            None => {
                return Err(ProjectDiscoveryProblem::InvalidModulePath {
                    path: relative_path.to_path_buf(),
                });
            }
        },
        Some(_) | None => {
            return Err(ProjectDiscoveryProblem::InvalidModulePath {
                path: relative_path.to_path_buf(),
            });
        }
    };
    let mut module_segments = components[content_start..components.len() - 1].to_vec();
    module_segments.push(module_name);
    Ok(module_segments.join("/"))
}

fn is_wasm_path(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("wasm")
}

fn is_project_asset_path(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map_or("", |file_name| file_name);
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "luau" | "rbxm" | "rbxmx" | "rbxl" | "rbxlx"))
        || file_name.ends_with(".model.json")
}
