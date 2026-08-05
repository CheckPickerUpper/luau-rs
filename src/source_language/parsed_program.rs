use crate::{
    source_language::{ParsedFunction, ParsedProjectImport, ParsedRecordDeclaration},
    SourceRange,
};

/// Owns every function accepted from one source file.
pub struct ParsedProgram {
    parsed_imports: Vec<ParsedProjectImport>,
    parsed_records: Vec<ParsedRecordDeclaration>,
    parsed_functions: Vec<ParsedFunction>,
    end_of_source_range: SourceRange,
}

/// Keeps top-level source declarations separate from statements and expressions.
impl ParsedProgram {
    /// Preserves complete top-level declarations together with the end-of-source location.
    pub(crate) fn from_declarations(
        parsed_program: (
            Vec<ParsedProjectImport>,
            Vec<ParsedRecordDeclaration>,
            Vec<ParsedFunction>,
            SourceRange,
        ),
    ) -> Self {
        let (parsed_imports, parsed_records, parsed_functions, end_of_source_range) =
            parsed_program;
        Self {
            parsed_imports,
            parsed_records,
            parsed_functions,
            end_of_source_range,
        }
    }

    /// Gives semantic checking every file-private record declaration before function bodies.
    pub(crate) fn parsed_records(&self) -> &[ParsedRecordDeclaration] {
        &self.parsed_records
    }

    /// Gives project compilation the complete top-level import declaration set.
    pub(crate) fn parsed_imports(&self) -> &[ParsedProjectImport] {
        &self.parsed_imports
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
