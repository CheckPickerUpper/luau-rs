//! Loading the `luau-rs.toml` project manifest.

use serde::Deserialize;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Identifies the manifest file and the two roots used by project commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectManifest {
    manifest_path: PathBuf,
    project: ManifestProject,
}

/// Explains why a project manifest could not be loaded.
#[derive(Debug, Error)]
pub enum ProjectManifestProblem {
    /// The manifest file could not be read from disk.
    #[error("could not read project manifest {path}: {source}")]
    Read {
        /// The manifest path that was requested.
        path: PathBuf,
        /// The filesystem failure returned by the operating system.
        #[source]
        source: std::io::Error,
    },
    /// The manifest did not match the supported TOML document shape.
    #[error("could not parse project manifest {path}: {source}")]
    Parse {
        /// The manifest path that contained invalid configuration.
        path: PathBuf,
        /// The TOML parser's typed error and source location.
        #[source]
        source: toml::de::Error,
    },
    /// The source root was empty, which would make discovery ambiguous.
    #[error("project.source_root {path:?} must not be empty")]
    EmptySourceRoot {
        /// The configured source-root value that failed validation.
        path: PathBuf,
    },
    /// The output root was empty, which would make publication ambiguous.
    #[error("project.output_root {path:?} must not be empty")]
    EmptyOutputRoot {
        /// The configured output-root value that failed validation.
        path: PathBuf,
    },
    /// The compiler cannot safely publish one project into the other project's source tree.
    #[error("project source_root {source_root:?} overlaps output_root {output_root:?}")]
    OverlappingRoots {
        /// The configured source root.
        source_root: PathBuf,
        /// The configured output root.
        output_root: PathBuf,
    },
}

impl ProjectManifest {
    /// Reads a manifest and resolves its roots relative to the manifest file.
    ///
    /// # Errors
    ///
    /// Returns a typed problem when the file cannot be read, the TOML shape is
    /// invalid, or either configured root is empty.
    pub fn load_from_path(manifest_path: &Path) -> Result<Self, ProjectManifestProblem> {
        let manifest_text = match fs_err::read_to_string(manifest_path) {
            Ok(manifest_text) => manifest_text,
            Err(source) => {
                return Err(ProjectManifestProblem::Read {
                    path: manifest_path.to_path_buf(),
                    source,
                });
            }
        };
        let document = match toml::from_str::<ManifestDocument>(&manifest_text) {
            Ok(document) => document,
            Err(source) => {
                return Err(ProjectManifestProblem::Parse {
                    path: manifest_path.to_path_buf(),
                    source,
                });
            }
        };
        if document.project.source_root.as_os_str().is_empty() {
            return Err(ProjectManifestProblem::EmptySourceRoot {
                path: document.project.source_root,
            });
        }
        if document.project.output_root.as_os_str().is_empty() {
            return Err(ProjectManifestProblem::EmptyOutputRoot {
                path: document.project.output_root,
            });
        }

        let manifest_directory = match manifest_path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
            Some(_) | None => PathBuf::from("."),
        };
        let source_root = resolve_manifest_path(&manifest_directory, &document.project.source_root);
        let output_root = resolve_manifest_path(&manifest_directory, &document.project.output_root);
        if source_root.starts_with(&output_root) || output_root.starts_with(&source_root) {
            return Err(ProjectManifestProblem::OverlappingRoots {
                source_root,
                output_root,
            });
        }
        Ok(Self {
            manifest_path: manifest_path.to_path_buf(),
            project: ManifestProject {
                source_root,
                output_root,
            },
        })
    }

    /// Returns the manifest path used to resolve project-relative paths.
    #[must_use]
    pub fn manifest_path(&self) -> &Path {
        self.manifest_path.as_path()
    }

    /// Returns the recursive root containing convention-named wasm modules.
    #[must_use]
    pub fn source_root(&self) -> &Path {
        self.project.source_root.as_path()
    }

    /// Returns the directory that receives the next successful project build.
    #[must_use]
    pub fn output_root(&self) -> &Path {
        self.project.output_root.as_path()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ManifestDocument {
    project: ManifestProject,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ManifestProject {
    source_root: PathBuf,
    output_root: PathBuf,
}

fn resolve_manifest_path(manifest_directory: &Path, configured_path: &Path) -> PathBuf {
    if configured_path.is_absolute() {
        configured_path.to_path_buf()
    } else {
        manifest_directory.join(configured_path)
    }
}
