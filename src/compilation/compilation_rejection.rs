use crate::CompilationProblem;

/// Owns all typed failures from one source compilation attempt.
#[derive(Debug, PartialEq, Eq)]
pub struct CompilationRejection {
    first_problem: CompilationProblem,
    remaining_problems: Vec<CompilationProblem>,
}

/// Provides read access without exposing a mutable diagnostic collection.
impl CompilationRejection {
    /// Preserves a phase failure as a non-empty public rejection value.
    pub(crate) fn from_problem(first_problem: CompilationProblem) -> Self {
        Self {
            first_problem,
            remaining_problems: Vec::new(),
        }
    }

    /// @why Exposes problem cardinality so callers can summarize a rejection without taking ownership of its details.
    pub fn problem_count(&self) -> usize {
        1 + self.remaining_problems.len()
    }

    /// @why Exposes the guaranteed problem so callers can present every rejection without handling an impossible empty case.
    pub fn first_problem(&self) -> &CompilationProblem {
        &self.first_problem
    }
}
