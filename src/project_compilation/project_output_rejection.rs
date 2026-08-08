use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

/// Identifies the filesystem phase that prevented an accepted project from being published.
#[derive(Debug, PartialEq, Eq)]
pub enum ProjectOutputOperation {
    /// Checks the destination's parent and existing shape before staging output.
    InspectDestination,
    /// Creates the sibling staging directory for a complete artifact set.
    CreateStagingDirectory,
    /// Creates one generated module's parent directory.
    CreateModuleDirectory,
    /// Writes one generated module into the staging tree.
    WriteModule,
    /// Flushes one generated module before publication.
    FlushModule,
    /// Moves the previous output aside while replacing it atomically.
    MovePreviousOutput,
    /// Publishes the complete staging tree at the requested destination.
    PublishStagingDirectory,
    /// Restores the previous output after publication could not complete.
    RestorePreviousOutput,
}

/// Preserves the destination, operation, and operating-system reason for a failed publication.
#[derive(Debug, PartialEq, Eq)]
pub struct ProjectOutputRejection {
    output_path: PathBuf,
    operation: ProjectOutputOperation,
    error_kind: ErrorKind,
}

impl ProjectOutputRejection {
    pub(crate) fn from_parts(output_parts: (PathBuf, ProjectOutputOperation, ErrorKind)) -> Self {
        let (output_path, operation, error_kind) = output_parts;
        Self {
            output_path,
            operation,
            error_kind,
        }
    }

    /// Gives the output destination whose publication failed.
    #[must_use]
    pub fn output_path(&self) -> &Path {
        &self.output_path
    }

    /// Gives the publication phase that failed.
    #[must_use]
    pub const fn operation(&self) -> &ProjectOutputOperation {
        &self.operation
    }

    /// Gives the stable operating-system error classification.
    #[must_use]
    pub const fn error_kind(&self) -> ErrorKind {
        self.error_kind
    }
}
