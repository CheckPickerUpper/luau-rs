use crate::{
    source_language::{
        parse_source_program::SourceProgramParser, ParsedProjectImport, SourceTokenKind,
    },
    CompilationProblem, ProjectModuleIdentity, SourceRange,
};

/// Parses project-only imports before function declarations so source modules expose their dependencies.
impl SourceProgramParser {
    pub(super) fn parse_project_import(
        &mut self,
    ) -> Result<ParsedProjectImport, CompilationProblem> {
        let import_start = match self.take_required_symbol(&SourceTokenKind::UseKeyword) {
            Ok(use_token) => use_token.source_range().start_byte(),
            Err(compilation_problem) => return Err(compilation_problem),
        };
        let (crate_name, crate_range) = match self.take_identifier_name() {
            Ok(identifier) => identifier,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        if crate_name != "crate" {
            return Err(Self::problem_at_range(crate_range));
        }
        match self.take_double_colon() {
            Ok(()) => {}
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let (side_name, side_range) = match self.take_identifier_name() {
            Ok(identifier) => identifier,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_double_colon() {
            Ok(()) => {}
            Err(compilation_problem) => return Err(compilation_problem),
        }

        let mut path_segments = Vec::new();
        loop {
            let path_segment = match self.take_identifier_name() {
                Ok(identifier) => identifier,
                Err(compilation_problem) => return Err(compilation_problem),
            };
            path_segments.push(path_segment);
            match self.current_token_kind() {
                Ok(SourceTokenKind::Semicolon) => break,
                Ok(SourceTokenKind::Colon | SourceTokenKind::DoubleColon) => {
                    match self.take_double_colon() {
                        Ok(()) => {}
                        Err(compilation_problem) => return Err(compilation_problem),
                    }
                }
                Ok(_) => return Err(self.problem_at_current_token()),
                Err(compilation_problem) => return Err(compilation_problem),
            }
        }
        let semicolon_range = match self.take_required_symbol(&SourceTokenKind::Semicolon) {
            Ok(semicolon) => semicolon.source_range(),
            Err(compilation_problem) => return Err(compilation_problem),
        };
        let Some((imported_function_name, imported_function_range)) = path_segments.pop() else {
            return Err(Self::problem_at_range(side_range));
        };
        if path_segments.is_empty() {
            return Err(Self::problem_at_range(imported_function_range));
        }
        let module_path = path_segments
            .into_iter()
            .map(|(segment_name, _)| segment_name)
            .collect::<Vec<_>>()
            .join("/");
        let target_module_identity = match side_name.as_str() {
            "server" => ProjectModuleIdentity::Server { module_path },
            "client" => ProjectModuleIdentity::Client { module_path },
            "shared" => ProjectModuleIdentity::Shared { module_path },
            _ => return Err(Self::problem_at_range(side_range)),
        };
        Ok(ParsedProjectImport::from_import_parts((
            target_module_identity,
            imported_function_name,
            imported_function_range,
            SourceRange::from_byte_range((import_start, semicolon_range.end_byte())),
        )))
    }

    fn take_double_colon(&mut self) -> Result<(), CompilationProblem> {
        if matches!(self.current_token_kind(), Ok(SourceTokenKind::DoubleColon)) {
            self.take_required_symbol(&SourceTokenKind::DoubleColon)?;
            return Ok(());
        }
        match self.take_required_symbol(&SourceTokenKind::Colon) {
            Ok(colon) => drop(colon),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        match self.take_required_symbol(&SourceTokenKind::Colon) {
            Ok(colon) => drop(colon),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        Ok(())
    }
}
