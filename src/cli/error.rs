use std::{fmt, io::ErrorKind, path::PathBuf};

use roblox_rust::{CompilationDiagnostic, ProjectOutputRejection};

use super::HELP_TEXT;

pub(super) enum CliError {
    Usage(String),
    Input {
        path: PathBuf,
        line: Option<usize>,
        message: String,
    },
    Compilation(String),
    Output {
        path: PathBuf,
        operation: String,
        error_kind: ErrorKind,
    },
}

impl CliError {
    pub(super) fn input(input_parts: (PathBuf, Option<usize>, String)) -> Self {
        let (path, line, message) = input_parts;
        Self::Input {
            path,
            line,
            message,
        }
    }

    pub(super) fn compilation(diagnostic: &CompilationDiagnostic) -> Self {
        Self::Compilation(diagnostic.to_text())
    }

    pub(super) fn from_output_rejection(rejection: &ProjectOutputRejection) -> Self {
        Self::Output {
            path: rejection.output_path().to_owned(),
            operation: format!("{:?}", rejection.operation()),
            error_kind: rejection.error_kind(),
        }
    }

    pub(super) const fn exit_code(&self) -> i32 {
        match self {
            Self::Usage(_) => 2,
            Self::Input { .. } | Self::Compilation(_) | Self::Output { .. } => 1,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) => write!(formatter, "error: {message}\n\n{HELP_TEXT}"),
            Self::Input {
                path,
                line,
                message,
            } => match line {
                Some(line) => write!(formatter, "{}:{line}: error: {message}", path.display()),
                None => write!(formatter, "{}: error: {message}", path.display()),
            },
            Self::Compilation(diagnostic) => formatter.write_str(diagnostic),
            Self::Output {
                path,
                operation,
                error_kind,
            } => write!(
                formatter,
                "{}: output {operation} failed: {error_kind:?}",
                path.display()
            ),
        }
    }
}
