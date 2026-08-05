use crate::{
    source_language::{SourceBooleanLiteral, SourceToken, SourceTokenKind},
    CompilationProblem, CompilationProblemReason, SourceRange,
};

/// Produces a complete token stream or located problems without leaking partial tokens.
pub fn split_source_into_tokens(source: &str) -> Result<Vec<SourceToken>, CompilationProblem> {
    let source_range = |byte_range| SourceRange::from_byte_range(byte_range);
    let unexpected_syntax = |byte_range| {
        CompilationProblem::from_problem_at_range((
            source_range(byte_range),
            CompilationProblemReason::SourceDoesNotFollowLanguageRules,
        ))
    };
    let unexpected_character = |character_at_bytes| {
        let (character, start_byte, end_byte) = character_at_bytes;
        CompilationProblem::from_problem_at_range((
            source_range((start_byte, end_byte)),
            CompilationProblemReason::UnsupportedCharacter(character),
        ))
    };
    let mut characters = source.char_indices().peekable();
    let mut source_tokens = Vec::new();

    while let Some((start_byte, character)) = characters.next() {
        match character {
            character if character.is_whitespace() => {}
            character if character.is_ascii_alphabetic() || character == '_' => {
                let mut end_byte = start_byte + character.len_utf8();
                loop {
                    match characters.peek().copied() {
                        Some((next_start_byte, next_character))
                            if next_character.is_ascii_alphanumeric() || next_character == '_' =>
                        {
                            characters.next();
                            end_byte = next_start_byte + next_character.len_utf8();
                        }
                        _ => break,
                    }
                }
                let identifier = match source.get(start_byte..end_byte) {
                    Some(identifier) => identifier.to_owned(),
                    None => return Err(unexpected_syntax((start_byte, end_byte))),
                };
                let token_kind = match identifier.as_str() {
                    "struct" => SourceTokenKind::StructKeyword,
                    "fn" => SourceTokenKind::FunctionKeyword,
                    "pub" => SourceTokenKind::PublicKeyword,
                    "use" => SourceTokenKind::UseKeyword,
                    "let" => SourceTokenKind::LetKeyword,
                    "mut" => SourceTokenKind::MutKeyword,
                    "return" => SourceTokenKind::ReturnKeyword,
                    "break" => SourceTokenKind::BreakKeyword,
                    "continue" => SourceTokenKind::ContinueKeyword,
                    "if" => SourceTokenKind::IfKeyword,
                    "else" => SourceTokenKind::ElseKeyword,
                    "while" => SourceTokenKind::WhileKeyword,
                    "true" => SourceTokenKind::BooleanLiteral(SourceBooleanLiteral::True),
                    "false" => SourceTokenKind::BooleanLiteral(SourceBooleanLiteral::False),
                    _ => SourceTokenKind::IdentifierName(identifier),
                };
                source_tokens.push(make_source_token((token_kind, (start_byte, end_byte))));
            }
            character if character.is_ascii_digit() => {
                let mut end_byte = start_byte + character.len_utf8();
                loop {
                    match characters.peek().copied() {
                        Some((next_start_byte, next_character))
                            if next_character.is_ascii_digit() =>
                        {
                            characters.next();
                            end_byte = next_start_byte + next_character.len_utf8();
                        }
                        _ => break,
                    }
                }
                let number_literal = match source.get(start_byte..end_byte) {
                    Some(number_literal) => number_literal.to_owned(),
                    None => return Err(unexpected_syntax((start_byte, end_byte))),
                };
                source_tokens.push(make_source_token((
                    SourceTokenKind::NumberLiteral(number_literal),
                    (start_byte, end_byte),
                )));
            }
            '"' => {
                let end_byte = loop {
                    match characters.next() {
                        Some((next_start_byte, '"')) => break next_start_byte + 1,
                        Some((next_start_byte, '\\')) => {
                            return Err(unexpected_character((
                                '\\',
                                next_start_byte,
                                next_start_byte + 1,
                            )));
                        }
                        Some((next_start_byte, newline @ ('\n' | '\r'))) => {
                            return Err(unexpected_character((
                                newline,
                                next_start_byte,
                                next_start_byte + newline.len_utf8(),
                            )));
                        }
                        Some(_) => {}
                        None => return Err(unexpected_syntax((start_byte, source.len()))),
                    }
                };
                let string_literal = match source.get(start_byte..end_byte) {
                    Some(string_literal) => string_literal.to_owned(),
                    None => return Err(unexpected_syntax((start_byte, end_byte))),
                };
                source_tokens.push(make_source_token((
                    SourceTokenKind::StringLiteral(string_literal),
                    (start_byte, end_byte),
                )));
            }
            '(' => source_tokens.push(make_source_token((
                SourceTokenKind::LeftParenthesis,
                (start_byte, start_byte + 1),
            ))),
            ')' => source_tokens.push(make_source_token((
                SourceTokenKind::RightParenthesis,
                (start_byte, start_byte + 1),
            ))),
            '{' => source_tokens.push(make_source_token((
                SourceTokenKind::LeftBrace,
                (start_byte, start_byte + 1),
            ))),
            '}' => source_tokens.push(make_source_token((
                SourceTokenKind::RightBrace,
                (start_byte, start_byte + 1),
            ))),
            '[' => source_tokens.push(make_source_token((
                SourceTokenKind::LeftBracket,
                (start_byte, start_byte + 1),
            ))),
            ']' => source_tokens.push(make_source_token((
                SourceTokenKind::RightBracket,
                (start_byte, start_byte + 1),
            ))),
            ':' => match characters.peek().copied() {
                Some((next_start_byte, ':')) => {
                    characters.next();
                    source_tokens.push(make_source_token((
                        SourceTokenKind::DoubleColon,
                        (start_byte, next_start_byte + 1),
                    )));
                }
                _ => source_tokens.push(make_source_token((
                    SourceTokenKind::Colon,
                    (start_byte, start_byte + 1),
                ))),
            },
            ',' => source_tokens.push(make_source_token((
                SourceTokenKind::Comma,
                (start_byte, start_byte + 1),
            ))),
            ';' => source_tokens.push(make_source_token((
                SourceTokenKind::Semicolon,
                (start_byte, start_byte + 1),
            ))),
            '=' => match characters.peek().copied() {
                Some((next_start_byte, '=')) => {
                    characters.next();
                    source_tokens.push(make_source_token((
                        SourceTokenKind::EqualEqual,
                        (start_byte, next_start_byte + 1),
                    )));
                }
                _ => source_tokens.push(make_source_token((
                    SourceTokenKind::Equals,
                    (start_byte, start_byte + 1),
                ))),
            },
            '!' => match characters.peek().copied() {
                Some((next_start_byte, '=')) => {
                    characters.next();
                    source_tokens.push(make_source_token((
                        SourceTokenKind::BangEqual,
                        (start_byte, next_start_byte + 1),
                    )));
                }
                _ => source_tokens.push(make_source_token((
                    SourceTokenKind::Bang,
                    (start_byte, start_byte + 1),
                ))),
            },
            '+' => source_tokens.push(make_source_token((
                SourceTokenKind::Plus,
                (start_byte, start_byte + 1),
            ))),
            '*' => source_tokens.push(make_source_token((
                SourceTokenKind::Star,
                (start_byte, start_byte + 1),
            ))),
            '/' => source_tokens.push(make_source_token((
                SourceTokenKind::Slash,
                (start_byte, start_byte + 1),
            ))),
            '<' => match characters.peek().copied() {
                Some((next_start_byte, '=')) => {
                    characters.next();
                    source_tokens.push(make_source_token((
                        SourceTokenKind::LessThanOrEqual,
                        (start_byte, next_start_byte + 1),
                    )));
                }
                _ => source_tokens.push(make_source_token((
                    SourceTokenKind::LessThan,
                    (start_byte, start_byte + 1),
                ))),
            },
            '>' => match characters.peek().copied() {
                Some((next_start_byte, '=')) => {
                    characters.next();
                    source_tokens.push(make_source_token((
                        SourceTokenKind::GreaterThanOrEqual,
                        (start_byte, next_start_byte + 1),
                    )));
                }
                _ => source_tokens.push(make_source_token((
                    SourceTokenKind::GreaterThan,
                    (start_byte, start_byte + 1),
                ))),
            },
            '&' => match characters.next() {
                Some((next_start_byte, '&')) => source_tokens.push(make_source_token((
                    SourceTokenKind::AmpersandAmpersand,
                    (start_byte, next_start_byte + 1),
                ))),
                Some((next_start_byte, next_character)) => {
                    return Err(unexpected_character((
                        next_character,
                        next_start_byte,
                        next_start_byte + next_character.len_utf8(),
                    )));
                }
                None => return Err(unexpected_character(('&', start_byte, start_byte + 1))),
            },
            '|' => match characters.next() {
                Some((next_start_byte, '|')) => source_tokens.push(make_source_token((
                    SourceTokenKind::PipePipe,
                    (start_byte, next_start_byte + 1),
                ))),
                Some((next_start_byte, next_character)) => {
                    return Err(unexpected_character((
                        next_character,
                        next_start_byte,
                        next_start_byte + next_character.len_utf8(),
                    )));
                }
                None => return Err(unexpected_character(('|', start_byte, start_byte + 1))),
            },
            '.' => source_tokens.push(make_source_token((
                SourceTokenKind::Dot,
                (start_byte, start_byte + 1),
            ))),
            '-' => match characters.peek().copied() {
                Some((next_start_byte, '>')) => {
                    characters.next();
                    source_tokens.push(make_source_token((
                        SourceTokenKind::Arrow,
                        (start_byte, next_start_byte + 1),
                    )));
                }
                _ => source_tokens.push(make_source_token((
                    SourceTokenKind::Minus,
                    (start_byte, start_byte + 1),
                ))),
            },
            _ => {
                return Err(unexpected_character((
                    character,
                    start_byte,
                    start_byte + character.len_utf8(),
                )));
            }
        }
    }

    source_tokens.push(make_source_token((
        SourceTokenKind::EndOfSource,
        (source.len(), source.len()),
    )));
    Ok(source_tokens)
}

fn make_source_token(token_at_bytes: (SourceTokenKind, (usize, usize))) -> SourceToken {
    let (token_kind, byte_range) = token_at_bytes;
    SourceToken::from_token_at_range((token_kind, SourceRange::from_byte_range(byte_range)))
}
