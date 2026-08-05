use std::{iter::Peekable, vec::IntoIter};

use crate::{
    source_language::{ParsedProgram, SourceToken, SourceTokenKind},
    CompilationProblem, CompilationProblemReason, SourceRange,
};

/// Keeps the token cursor and end location together so grammar operations retain diagnostic positions.
pub(super) struct SourceProgramParser {
    remaining_tokens: Peekable<IntoIter<SourceToken>>,
    end_of_source_range: SourceRange,
    pub(super) record_literals_are_allowed: bool,
}

/// Converts a complete token stream into an explicitly shaped source program.
pub fn parse_source_program(
    source_tokens: Vec<SourceToken>,
) -> Result<ParsedProgram, CompilationProblem> {
    SourceProgramParser::from_tokens(source_tokens).parse_program()
}

/// Owns token navigation and whole-program orchestration for source parsing.
impl SourceProgramParser {
    fn from_tokens(source_tokens: Vec<SourceToken>) -> Self {
        let end_of_source_range = source_tokens.last().map_or_else(
            || SourceRange::from_byte_range((source_tokens.len(), source_tokens.len())),
            SourceToken::source_range,
        );
        Self {
            remaining_tokens: source_tokens.into_iter().peekable(),
            end_of_source_range,
            record_literals_are_allowed: true,
        }
    }

    fn parse_program(&mut self) -> Result<ParsedProgram, CompilationProblem> {
        let mut parsed_imports = Vec::new();
        let mut parsed_records = Vec::new();
        let mut parsed_functions = Vec::new();
        loop {
            match self.current_token_kind() {
                Ok(SourceTokenKind::EndOfSource) => {
                    let end_of_source_token = match self.take_next_token() {
                        Ok(end_of_source_token) => end_of_source_token,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    return Ok(ParsedProgram::from_declarations((
                        parsed_imports,
                        parsed_records,
                        parsed_functions,
                        end_of_source_token.source_range(),
                    )));
                }
                Ok(SourceTokenKind::UseKeyword) => match self.parse_project_import() {
                    Ok(parsed_import) => parsed_imports.push(parsed_import),
                    Err(compilation_problem) => return Err(compilation_problem),
                },
                Ok(SourceTokenKind::StructKeyword) => match self.parse_record_declaration() {
                    Ok(parsed_record) => parsed_records.push(parsed_record),
                    Err(compilation_problem) => return Err(compilation_problem),
                },
                Ok(_) => match self.parse_function() {
                    Ok(parsed_function) => parsed_functions.push(parsed_function),
                    Err(compilation_problem) => return Err(compilation_problem),
                },
                Err(compilation_problem) => return Err(compilation_problem),
            }
        }
    }

    /// Borrows the next lexical category without consuming its located token.
    pub(super) fn current_token_kind(&mut self) -> Result<&SourceTokenKind, CompilationProblem> {
        let end_of_source_range = self.end_of_source_range;
        self.remaining_tokens.peek().map_or_else(
            || Err(Self::problem_at_range(end_of_source_range)),
            |source_token| Ok(source_token.token_kind()),
        )
    }

    /// Consumes an ordinary identifier while preserving its source location.
    pub(super) fn take_identifier_name(
        &mut self,
    ) -> Result<(String, SourceRange), CompilationProblem> {
        let source_token = match self.take_next_token() {
            Ok(source_token) => source_token,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        let (token_kind, token_range) = source_token.into_token_at_range();
        let SourceTokenKind::IdentifierName(identifier_name) = token_kind else {
            return Err(Self::problem_at_range(token_range));
        };
        Ok((identifier_name, token_range))
    }

    /// Consumes a declaration name, including source keywords requiring a typed rejection later.
    pub(super) fn take_declaration_name(
        &mut self,
    ) -> Result<(String, SourceRange), CompilationProblem> {
        let source_token = match self.take_next_token() {
            Ok(source_token) => source_token,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        let (token_kind, token_range) = source_token.into_token_at_range();
        if let SourceTokenKind::IdentifierName(identifier_name) = token_kind {
            return Ok((identifier_name, token_range));
        }
        let SourceTokenKind::ReturnKeyword = token_kind else {
            return Err(Self::problem_at_range(token_range));
        };
        Ok(("return".to_owned(), token_range))
    }

    /// Consumes one required grammar symbol or reports the encountered token range.
    pub(super) fn take_required_symbol(
        &mut self,
        required_symbol: &SourceTokenKind,
    ) -> Result<SourceToken, CompilationProblem> {
        let source_token = match self.take_next_token() {
            Ok(source_token) => source_token,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match (required_symbol, source_token.token_kind()) {
            (SourceTokenKind::StructKeyword, SourceTokenKind::StructKeyword)
            | (SourceTokenKind::FunctionKeyword, SourceTokenKind::FunctionKeyword)
            | (SourceTokenKind::PublicKeyword, SourceTokenKind::PublicKeyword)
            | (SourceTokenKind::UseKeyword, SourceTokenKind::UseKeyword)
            | (SourceTokenKind::LetKeyword, SourceTokenKind::LetKeyword)
            | (SourceTokenKind::MutKeyword, SourceTokenKind::MutKeyword)
            | (SourceTokenKind::ReturnKeyword, SourceTokenKind::ReturnKeyword)
            | (SourceTokenKind::BreakKeyword, SourceTokenKind::BreakKeyword)
            | (SourceTokenKind::ContinueKeyword, SourceTokenKind::ContinueKeyword)
            | (SourceTokenKind::IfKeyword, SourceTokenKind::IfKeyword)
            | (SourceTokenKind::ElseKeyword, SourceTokenKind::ElseKeyword)
            | (SourceTokenKind::WhileKeyword, SourceTokenKind::WhileKeyword)
            | (SourceTokenKind::LeftParenthesis, SourceTokenKind::LeftParenthesis)
            | (SourceTokenKind::RightParenthesis, SourceTokenKind::RightParenthesis)
            | (SourceTokenKind::LeftBrace, SourceTokenKind::LeftBrace)
            | (SourceTokenKind::RightBrace, SourceTokenKind::RightBrace)
            | (SourceTokenKind::LeftBracket, SourceTokenKind::LeftBracket)
            | (SourceTokenKind::RightBracket, SourceTokenKind::RightBracket)
            | (SourceTokenKind::Colon, SourceTokenKind::Colon)
            | (SourceTokenKind::DoubleColon, SourceTokenKind::DoubleColon)
            | (SourceTokenKind::Comma, SourceTokenKind::Comma)
            | (SourceTokenKind::Semicolon, SourceTokenKind::Semicolon)
            | (SourceTokenKind::Arrow, SourceTokenKind::Arrow)
            | (SourceTokenKind::Equals, SourceTokenKind::Equals)
            | (SourceTokenKind::EqualEqual, SourceTokenKind::EqualEqual)
            | (SourceTokenKind::BangEqual, SourceTokenKind::BangEqual)
            | (SourceTokenKind::Bang, SourceTokenKind::Bang)
            | (SourceTokenKind::Plus, SourceTokenKind::Plus)
            | (SourceTokenKind::Minus, SourceTokenKind::Minus)
            | (SourceTokenKind::Star, SourceTokenKind::Star)
            | (SourceTokenKind::Slash, SourceTokenKind::Slash)
            | (SourceTokenKind::LessThan, SourceTokenKind::LessThan)
            | (SourceTokenKind::LessThanOrEqual, SourceTokenKind::LessThanOrEqual)
            | (SourceTokenKind::GreaterThan, SourceTokenKind::GreaterThan)
            | (SourceTokenKind::GreaterThanOrEqual, SourceTokenKind::GreaterThanOrEqual)
            | (SourceTokenKind::AmpersandAmpersand, SourceTokenKind::AmpersandAmpersand)
            | (SourceTokenKind::PipePipe, SourceTokenKind::PipePipe)
            | (SourceTokenKind::Dot, SourceTokenKind::Dot) => Ok(source_token),
            _ => Err(Self::problem_at_range(source_token.source_range())),
        }
    }

    /// Consumes the next located token or reports the stored end-of-source range.
    pub(super) fn take_next_token(&mut self) -> Result<SourceToken, CompilationProblem> {
        self.remaining_tokens
            .next()
            .map_or_else(|| Err(self.problem_at_end_of_source()), Ok)
    }

    /// Reports invalid grammar at the next token or the stored end-of-source range.
    pub(super) fn problem_at_current_token(&mut self) -> CompilationProblem {
        let end_of_source_range = self.end_of_source_range;
        self.remaining_tokens.peek().map_or_else(
            || Self::problem_at_range(end_of_source_range),
            |source_token| Self::problem_at_range(source_token.source_range()),
        )
    }

    const fn problem_at_end_of_source(&self) -> CompilationProblem {
        Self::problem_at_range(self.end_of_source_range)
    }

    pub(super) const fn problem_at_range(source_range: SourceRange) -> CompilationProblem {
        CompilationProblem::from_problem_at_range((
            source_range,
            CompilationProblemReason::SourceDoesNotFollowLanguageRules,
        ))
    }
}
