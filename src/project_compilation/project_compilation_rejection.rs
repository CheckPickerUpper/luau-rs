use crate::{CompilationDiagnostic, ProjectCompilationProblem};

/// Guarantees a rejected project always retains its first typed and file-aware diagnostic.
#[derive(Debug)]
pub struct ProjectCompilationRejection {
    first_problem: ProjectCompilationProblem,
    first_diagnostic: CompilationDiagnostic,
}

/// Keeps the project failure surface non-empty without exposing a mutable diagnostic collection.
impl ProjectCompilationRejection {
    pub(crate) fn from_parts(
        rejection_parts: (ProjectCompilationProblem, CompilationDiagnostic),
    ) -> Self {
        let (first_problem, first_diagnostic) = rejection_parts;
        Self {
            first_problem,
            first_diagnostic,
        }
    }

    /// @why Gives callers one exhaustive project-level failure to present before a later diagnostic aggregation phase exists.
    #[must_use]
    pub const fn first_problem(&self) -> &ProjectCompilationProblem {
        &self.first_problem
    }

    /// Converts the first project rejection into the stable file-aware diagnostic surface.
    #[must_use]
    pub const fn first_diagnostic(&self) -> &CompilationDiagnostic {
        &self.first_diagnostic
    }
}
