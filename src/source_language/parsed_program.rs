use crate::{
    source_language::{ParsedFunction, ParsedProjectImport, ParsedRecordDeclaration},
    MacroExpansionFrame, SourceRange,
};

#[derive(Clone)]
pub(super) struct MacroOrigin {
    origin_id: usize,
    macro_backtrace: Vec<MacroExpansionFrame>,
}

impl MacroOrigin {
    pub(super) fn from_token(source_token: &super::SourceToken) -> Option<Self> {
        Some(Self {
            origin_id: source_token.source_range().macro_origin_id()?,
            macro_backtrace: source_token.macro_backtrace().to_vec(),
        })
    }
}
pub(super) type ParsedProgramDeclarations = (
    Vec<ParsedProjectImport>,
    Vec<ParsedRecordDeclaration>,
    Vec<ParsedFunction>,
    SourceRange,
    Vec<MacroOrigin>,
);

/// Owns every function accepted from one source file.
pub struct ParsedProgram {
    parsed_imports: Vec<ParsedProjectImport>,
    parsed_records: Vec<ParsedRecordDeclaration>,
    parsed_functions: Vec<ParsedFunction>,
    end_of_source_range: SourceRange,
    macro_origins: Vec<MacroOrigin>,
}

/// Keeps top-level source declarations separate from statements and expressions.
impl ParsedProgram {
    /// Preserves complete top-level declarations together with the end-of-source location.
    pub(super) fn from_declarations(parsed_program: ParsedProgramDeclarations) -> Self {
        let (parsed_imports, parsed_records, parsed_functions, end_of_source_range, macro_origins) =
            parsed_program;
        Self {
            parsed_imports,
            parsed_records,
            parsed_functions,
            end_of_source_range,
            macro_origins,
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

    pub(crate) fn macro_backtrace_for_range(
        &self,
        source_range: SourceRange,
    ) -> Option<&[MacroExpansionFrame]> {
        self.macro_origins
            .iter()
            .find(|macro_origin| source_range.macro_origin_id() == Some(macro_origin.origin_id))
            .map(|macro_origin| macro_origin.macro_backtrace.as_slice())
    }
}
