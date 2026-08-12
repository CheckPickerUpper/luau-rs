use std::fmt;

use crate::ProjectCompilationProblem;

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
}

impl fmt::Display for ProjectCompilationRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.first_problem.fmt(formatter)
    }
}
