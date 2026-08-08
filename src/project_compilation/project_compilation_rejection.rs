use crate::{CompilationDiagnostic, ProjectCompilationProblem, SourceRange};

/// Guarantees a rejected project always retains its first typed and file-aware diagnostic.
#[derive(Debug)]
pub struct ProjectCompilationRejection {
    first_problem: ProjectCompilationProblem,
}

/// Keeps the project failure surface non-empty without exposing a mutable diagnostic collection.
impl ProjectCompilationRejection {
    pub(crate) const fn from_problem(first_problem: ProjectCompilationProblem) -> Self {
        Self { first_problem }
    }

    /// @why Gives callers one exhaustive project-level failure to present before a later diagnostic aggregation phase exists.
    #[must_use]
    pub const fn first_problem(&self) -> &ProjectCompilationProblem {
        &self.first_problem
    }

    /// Converts the first project rejection into the stable file-aware diagnostic surface.
    #[must_use]
    pub fn first_diagnostic(&self, diagnostic_parts: (&str, &str)) -> CompilationDiagnostic {
        let (file_name, source_text) = diagnostic_parts;
        if let ProjectCompilationProblem::SourceModuleRejected {
            compilation_rejection,
            ..
        } = &self.first_problem
        {
            return compilation_rejection.first_diagnostic((file_name, source_text));
        }
        CompilationDiagnostic::from_parts((
            file_name,
            source_text,
            self.first_problem
                .source_range()
                .unwrap_or_else(|| SourceRange::from_byte_range((0, 0))),
            self.first_problem.code(),
        ))
    }
}
