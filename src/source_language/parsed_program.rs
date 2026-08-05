use crate::{source_language::ParsedFunction, SourceRange};

/// Owns every function accepted from one source file.
pub struct ParsedProgram {
    parsed_functions: Vec<ParsedFunction>,
    end_of_source_range: SourceRange,
}

/// Keeps top-level source declarations separate from statements and expressions.
impl ParsedProgram {
    /// Preserves complete top-level declarations together with the end-of-source location.
    pub(crate) fn from_functions(parsed_program: (Vec<ParsedFunction>, SourceRange)) -> Self {
        let (parsed_functions, end_of_source_range) = parsed_program;
        Self {
            parsed_functions,
            end_of_source_range,
        }
    }

    /// Gives semantic checking the complete ordered function declaration set.
    pub(crate) fn parsed_functions(&self) -> &[ParsedFunction] {
        &self.parsed_functions
    }

    /// Gives entrypoint validation the real location immediately after the source program.
    pub(crate) const fn end_of_source_range(&self) -> SourceRange {
        self.end_of_source_range
    }
}
