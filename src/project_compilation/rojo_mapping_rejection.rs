use std::{io::ErrorKind, path::PathBuf};

use crate::ProjectModuleIdentity;

/// Identifies the mapping configuration or generated module that failed validation.
#[derive(Debug, PartialEq, Eq)]
pub enum RojoMappingField {
    /// The human-readable Rojo project name was invalid.
    ProjectName,
    /// The root path used to locate generated Luau files was invalid.
    GeneratedRoot,
    /// The destination project file could not be written.
    DestinationPath,
    /// A generated module could not receive a unique Rojo instance path.
    ModuleOutputPath,
}

/// Classifies a rejected Rojo mapping without requiring callers to parse text.
#[derive(Debug, PartialEq, Eq)]
pub enum RojoMappingProblem {
    /// A required text value was empty.
    EmptyValue,
    /// A generated root escaped the project file or used an unsupported separator.
    InvalidRelativePath,
    /// Two generated modules claimed one Rojo instance path.
    DuplicateInstancePath,
    /// The destination filesystem operation failed.
    Filesystem(ErrorKind),
}

/// Preserves the responsible configuration field or source module for a mapping failure.
#[derive(Debug, PartialEq, Eq)]
pub struct RojoMappingRejection {
    field: RojoMappingField,
    module_identity: Option<ProjectModuleIdentity>,
    problem: RojoMappingProblem,
    destination_path: Option<PathBuf>,
}

impl RojoMappingRejection {
    pub(crate) fn from_parts(
        rejection_parts: (
            RojoMappingField,
            Option<ProjectModuleIdentity>,
            RojoMappingProblem,
            Option<PathBuf>,
        ),
    ) -> Self {
        let (field, module_identity, problem, destination_path) = rejection_parts;
        Self {
            field,
            module_identity,
            problem,
            destination_path,
        }
    }

    /// Gives the configuration field or generated surface that failed.
    #[must_use]
    pub const fn field(&self) -> &RojoMappingField {
        &self.field
    }

    /// Gives the source module responsible for a module-output failure, when applicable.
    #[must_use]
    pub const fn module_identity(&self) -> Option<&ProjectModuleIdentity> {
        self.module_identity.as_ref()
    }

    /// Gives the typed mapping failure reason.
    #[must_use]
    pub const fn problem(&self) -> &RojoMappingProblem {
        &self.problem
    }

    /// Gives the destination project file when the filesystem operation named one.
    #[must_use]
    pub const fn destination_path(&self) -> Option<&PathBuf> {
        self.destination_path.as_ref()
    }
}
