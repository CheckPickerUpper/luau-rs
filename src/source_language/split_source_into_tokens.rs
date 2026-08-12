use std::ops::Range;

use logos::Logos;

use crate::{
    source_language::{SourceBooleanLiteral, SourceToken, SourceTokenKind},
    CompilationProblem, CompilationProblemReason, SourceRange,
};

/// Produces a complete token stream or located problems without leaking partial tokens.
pub fn split_source_into_tokens(source: &str) -> Result<Vec<SourceToken>, CompilationProblem> {
    let mut source_tokens = Vec::new();
    let token_stream = LogosToken::lexer(source).spanned();

    for (lexed_token, byte_range) in token_stream {
        let Ok(token) = lexed_token else {
            if let Some(character) = source
                .get(byte_range.clone())
                .and_then(|token_text| token_text.chars().next())
            {
                if character.is_whitespace() {
                    continue;
                }
                if character == '&' || character == '|' {
                    return Err(invalid_logical_operator_problem(source, &byte_range));
                }
                if character == '"' {
                    return Err(unclosed_string_problem(source, byte_range.start));
                }
                return Err(unsupported_character_problem((
                    character,
                    byte_range.start,
                    byte_range.end,
                )));
            }
            return Err(syntax_problem((byte_range.start, byte_range.end)));
        };

        if let LogosToken::StringLiteral(string_literal) = &token {
            if let Some((offset, character)) =
                string_literal.char_indices().find(|(_, character)| {
                    *character == '\\' || *character == '\n' || *character == '\r'
                })
            {
                let start_byte = byte_range.start + offset;
                return Err(unsupported_character_problem((
                    character,
                    start_byte,
                    start_byte + character.len_utf8(),
                )));
            }
        }

        source_tokens.push(make_source_token((token, byte_range)));
    }

    source_tokens.push(make_source_token((
        LogosToken::EndOfSource,
        source.len()..source.len(),
    )));
    Ok(source_tokens)
}

#[derive(Logos)]
#[logos(skip r"[ \t\n\r\f]+")]
enum LogosToken {
    #[token("struct")]
    StructKeyword,
    #[token("fn")]
    FunctionKeyword,
    #[token("pub")]
    PublicKeyword,
    #[token("use")]
    UseKeyword,
    #[token("let")]
    LetKeyword,
    #[token("mut")]
    MutKeyword,
    #[token("return")]
    ReturnKeyword,
    #[token("break")]
    BreakKeyword,
    #[token("continue")]
    ContinueKeyword,
    #[token("if")]
    IfKeyword,
    #[token("else")]
    ElseKeyword,
    #[token("while")]
    WhileKeyword,
    #[regex("[A-Za-z_][A-Za-z0-9_]*", |lex| lex.slice().to_owned())]
    Identifier(String),
    #[regex("[0-9]+", |lex| lex.slice().to_owned())]
    NumberLiteral(String),
    #[regex(r#""[^"]*""#, |lex| lex.slice().to_owned())]
    StringLiteral(String),
    #[token("(")]
    LeftParenthesis,
    #[token(")")]
    RightParenthesis,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token("::")]
    DoubleColon,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token("->")]
    Arrow,
    #[token("==")]
    EqualEqual,
    #[token("=")]
    Equals,
    #[token("!=")]
    BangEqual,
    #[token("!")]
    Bang,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("<=")]
    LessThanOrEqual,
    #[token("<")]
    LessThan,
    #[token(">=")]
    GreaterThanOrEqual,
    #[token(">")]
    GreaterThan,
    #[token("&&")]
    AmpersandAmpersand,
    #[token("||")]
    PipePipe,
    #[token(".")]
    Dot,
    EndOfSource,
}

fn make_source_token(token_at_bytes: (LogosToken, Range<usize>)) -> SourceToken {
    let (token, byte_range) = token_at_bytes;
    let token_kind = match token {
        LogosToken::StructKeyword => SourceTokenKind::StructKeyword,
        LogosToken::FunctionKeyword => SourceTokenKind::FunctionKeyword,
        LogosToken::PublicKeyword => SourceTokenKind::PublicKeyword,
        LogosToken::UseKeyword => SourceTokenKind::UseKeyword,
        LogosToken::LetKeyword => SourceTokenKind::LetKeyword,
        LogosToken::MutKeyword => SourceTokenKind::MutKeyword,
        LogosToken::ReturnKeyword => SourceTokenKind::ReturnKeyword,
        LogosToken::BreakKeyword => SourceTokenKind::BreakKeyword,
        LogosToken::ContinueKeyword => SourceTokenKind::ContinueKeyword,
        LogosToken::IfKeyword => SourceTokenKind::IfKeyword,
        LogosToken::ElseKeyword => SourceTokenKind::ElseKeyword,
        LogosToken::WhileKeyword => SourceTokenKind::WhileKeyword,
        LogosToken::Identifier(identifier) => identifier_token_kind(identifier),
        LogosToken::NumberLiteral(number_literal) => SourceTokenKind::NumberLiteral(number_literal),
        LogosToken::StringLiteral(string_literal) => SourceTokenKind::StringLiteral(string_literal),
        LogosToken::LeftParenthesis => SourceTokenKind::LeftParenthesis,
        LogosToken::RightParenthesis => SourceTokenKind::RightParenthesis,
        LogosToken::LeftBrace => SourceTokenKind::LeftBrace,
        LogosToken::RightBrace => SourceTokenKind::RightBrace,
        LogosToken::LeftBracket => SourceTokenKind::LeftBracket,
        LogosToken::RightBracket => SourceTokenKind::RightBracket,
        LogosToken::Colon => SourceTokenKind::Colon,
        LogosToken::DoubleColon => SourceTokenKind::DoubleColon,
        LogosToken::Comma => SourceTokenKind::Comma,
        LogosToken::Semicolon => SourceTokenKind::Semicolon,
        LogosToken::Arrow => SourceTokenKind::Arrow,
        LogosToken::Equals => SourceTokenKind::Equals,
        LogosToken::EqualEqual => SourceTokenKind::EqualEqual,
        LogosToken::BangEqual => SourceTokenKind::BangEqual,
        LogosToken::Bang => SourceTokenKind::Bang,
        LogosToken::Plus => SourceTokenKind::Plus,
        LogosToken::Minus => SourceTokenKind::Minus,
        LogosToken::Star => SourceTokenKind::Star,
        LogosToken::Slash => SourceTokenKind::Slash,
        LogosToken::LessThan => SourceTokenKind::LessThan,
        LogosToken::LessThanOrEqual => SourceTokenKind::LessThanOrEqual,
        LogosToken::GreaterThan => SourceTokenKind::GreaterThan,
        LogosToken::GreaterThanOrEqual => SourceTokenKind::GreaterThanOrEqual,
        LogosToken::AmpersandAmpersand => SourceTokenKind::AmpersandAmpersand,
        LogosToken::PipePipe => SourceTokenKind::PipePipe,
        LogosToken::Dot => SourceTokenKind::Dot,
        LogosToken::EndOfSource => SourceTokenKind::EndOfSource,
    };
    SourceToken::from_token_at_range((
        token_kind,
        SourceRange::from_byte_range((byte_range.start, byte_range.end)),
    ))
}

fn identifier_token_kind(identifier: String) -> SourceTokenKind {
    match identifier.as_str() {
        "true" => SourceTokenKind::BooleanLiteral(SourceBooleanLiteral::True),
        "false" => SourceTokenKind::BooleanLiteral(SourceBooleanLiteral::False),
        _ => SourceTokenKind::IdentifierName(identifier),
    }
}

fn invalid_logical_operator_problem(
    source: &str,
    operator_range: &Range<usize>,
) -> CompilationProblem {
    match source
        .get(operator_range.end..)
        .and_then(|remaining_source| remaining_source.char_indices().next())
    {
        Some((offset, character)) => {
            let start_byte = operator_range.end + offset;
            unsupported_character_problem((
                character,
                start_byte,
                start_byte + character.len_utf8(),
            ))
        }
        None => unsupported_character_problem((
            source
                .get(operator_range.clone())
                .and_then(|operator| operator.chars().next())
                .unwrap_or('&'),
            operator_range.start,
            operator_range.end,
        )),
    }
}

fn unclosed_string_problem(source: &str, start_byte: usize) -> CompilationProblem {
    match source.get(start_byte + 1..).and_then(|remaining_source| {
        remaining_source
            .char_indices()
            .find(|(_, character)| *character == '\\' || *character == '\n' || *character == '\r')
    }) {
        Some((offset, character)) => {
            let character_start = start_byte + 1 + offset;
            unsupported_character_problem((
                character,
                character_start,
                character_start + character.len_utf8(),
            ))
        }
        None => syntax_problem((start_byte, source.len())),
    }
}

const fn syntax_problem(byte_range: (usize, usize)) -> CompilationProblem {
    CompilationProblem::from_problem_at_range((
        SourceRange::from_byte_range(byte_range),
        CompilationProblemReason::SourceDoesNotFollowLanguageRules,
    ))
}

const fn unsupported_character_problem(
    character_at_bytes: (char, usize, usize),
) -> CompilationProblem {
    let (character, start_byte, end_byte) = character_at_bytes;
    CompilationProblem::from_problem_at_range((
        SourceRange::from_byte_range((start_byte, end_byte)),
        CompilationProblemReason::UnsupportedCharacter(character),
    ))
}
