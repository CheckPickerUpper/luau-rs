use std::collections::{hash_map::Entry, HashMap};

use crate::{CompilationProblem, CompilationProblemReason, SourceRange};

use super::{SourceToken, SourceTokenKind};

#[derive(Clone)]
pub(super) struct MacroDefinition {
    pub(super) name: String,
    pub(super) parameter: Option<String>,
    pub(super) body: Vec<SourceToken>,
    pub(super) definition_range: SourceRange,
    pub(super) definition_module: Option<String>,
}

#[derive(Default)]
/// Collects the deterministic single-rule macros visible to one compilation.
pub struct MacroCatalog {
    definitions: HashMap<String, MacroDefinition>,
}

impl MacroCatalog {
    pub fn merge(&mut self, other_catalog: Self) -> Result<(), CompilationProblem> {
        for definition in other_catalog.definitions.into_values() {
            self.insert(definition)?;
        }
        Ok(())
    }

    fn insert(&mut self, definition: MacroDefinition) -> Result<(), CompilationProblem> {
        match self.definitions.entry(definition.name.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(definition);
                Ok(())
            }
            Entry::Occupied(_) => Err(problem_at_range(
                definition.definition_range,
                CompilationProblemReason::MacroMatcherAmbiguous,
            )),
        }
    }

    pub(super) fn definition(&self, name: &str) -> Option<&MacroDefinition> {
        self.definitions.get(name)
    }
}

pub fn extract_macro_definitions(
    source_tokens: &[SourceToken],
    definition_module: Option<&str>,
) -> Result<(MacroCatalog, Vec<SourceToken>), CompilationProblem> {
    let mut catalog = MacroCatalog::default();
    let mut remaining_tokens = Vec::with_capacity(source_tokens.len());
    let mut delimiter_stack = Vec::new();
    let mut token_index = 0;
    while token_index < source_tokens.len() {
        if delimiter_stack.is_empty()
            && matches!(
                source_tokens[token_index].token_kind(),
                SourceTokenKind::MacroKeyword
            )
        {
            let (definition, closing_index) =
                parse_macro_definition(source_tokens, token_index, definition_module)?;
            catalog.insert(definition)?;
            token_index = closing_index + 1;
        } else {
            let source_token = &source_tokens[token_index];
            update_delimiter_stack(source_token, &mut delimiter_stack)?;
            remaining_tokens.push(source_token.clone());
            token_index += 1;
        }
    }
    if let Some(source_token) = remaining_tokens.last() {
        if !delimiter_stack.is_empty() {
            return Err(problem_at_token(
                source_token,
                CompilationProblemReason::SourceDoesNotFollowLanguageRules,
            ));
        }
    }
    Ok((catalog, remaining_tokens))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Delimiter {
    Parenthesis,
    Brace,
    Bracket,
}

fn parse_macro_definition(
    source_tokens: &[SourceToken],
    macro_index: usize,
    definition_module: Option<&str>,
) -> Result<(MacroDefinition, usize), CompilationProblem> {
    let Some(macro_token) = source_tokens.get(macro_index) else {
        return Err(problem_at_range(
            SourceRange::from_byte_range((0, 0)),
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    let Some(name_token) = source_tokens.get(macro_index + 1) else {
        return Err(problem_at_token(
            macro_token,
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    let Some(name) = identifier_name(Some(name_token)) else {
        return Err(problem_at_token(
            name_token,
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    let parameter_open_index = macro_index + 2;
    let Some(parameter_open_token) = source_tokens.get(parameter_open_index) else {
        return Err(problem_at_token(
            name_token,
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    if !matches!(
        parameter_open_token.token_kind(),
        SourceTokenKind::LeftParenthesis
    ) {
        return Err(problem_at_token(
            parameter_open_token,
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    }
    let parameter_close_index = find_group_end(source_tokens, parameter_open_index)?;
    let body_open_index = parameter_close_index + 1;
    let Some(body_open_token) = source_tokens.get(body_open_index) else {
        return Err(problem_at_range(
            parameter_open_token.source_range(),
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    if !matches!(body_open_token.token_kind(), SourceTokenKind::LeftBrace) {
        return Err(problem_at_token(
            body_open_token,
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    }
    let body_close_index = find_group_end(source_tokens, body_open_index)?;
    let parameter =
        parse_macro_parameter(&source_tokens[parameter_open_index + 1..parameter_close_index])?;
    let body = source_tokens[body_open_index + 1..body_close_index].to_vec();
    let Some(body_close_token) = source_tokens.get(body_close_index) else {
        return Err(problem_at_token(
            body_open_token,
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    Ok((
        MacroDefinition {
            name: name.to_owned(),
            parameter,
            body,
            definition_range: macro_token
                .source_range()
                .through(body_close_token.source_range()),
            definition_module: definition_module.map(str::to_owned),
        },
        body_close_index,
    ))
}

fn parse_macro_parameter(
    parameter_tokens: &[SourceToken],
) -> Result<Option<String>, CompilationProblem> {
    if parameter_tokens.is_empty() {
        return Ok(None);
    }
    if parameter_tokens.len() == 2
        && matches!(parameter_tokens[0].token_kind(), SourceTokenKind::Dollar)
    {
        if let Some(parameter_name) = identifier_name(parameter_tokens.get(1)) {
            return Ok(Some(parameter_name.to_owned()));
        }
    }
    let Some(first_token) = parameter_tokens.first() else {
        return Err(problem_at_range(
            SourceRange::from_byte_range((0, 0)),
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    Err(problem_at_token(
        first_token,
        CompilationProblemReason::MacroDefinitionInvalid,
    ))
}

fn update_delimiter_stack(
    source_token: &SourceToken,
    delimiter_stack: &mut Vec<Delimiter>,
) -> Result<(), CompilationProblem> {
    if let Some(delimiter) = opening_delimiter(source_token.token_kind()) {
        delimiter_stack.push(delimiter);
        return Ok(());
    }
    let Some(delimiter) = closing_delimiter(source_token.token_kind()) else {
        return Ok(());
    };
    if delimiter_stack.last().copied() != Some(delimiter) {
        return Err(problem_at_token(
            source_token,
            CompilationProblemReason::SourceDoesNotFollowLanguageRules,
        ));
    }
    delimiter_stack.pop();
    Ok(())
}

pub(super) fn find_group_end(
    source_tokens: &[SourceToken],
    opening_index: usize,
) -> Result<usize, CompilationProblem> {
    let Some(opening_token) = source_tokens.get(opening_index) else {
        return Err(problem_at_range(
            SourceRange::from_byte_range((0, 0)),
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    let Some(opening_kind) = opening_delimiter(opening_token.token_kind()) else {
        return Err(problem_at_token(
            opening_token,
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    let mut delimiter_stack = vec![opening_kind];
    for (token_index, source_token) in source_tokens.iter().enumerate().skip(opening_index + 1) {
        if let Some(delimiter) = opening_delimiter(source_token.token_kind()) {
            delimiter_stack.push(delimiter);
            continue;
        }
        let Some(delimiter) = closing_delimiter(source_token.token_kind()) else {
            continue;
        };
        if delimiter_stack.last().copied() != Some(delimiter) {
            return Err(problem_at_token(
                source_token,
                CompilationProblemReason::MacroDefinitionInvalid,
            ));
        }
        delimiter_stack.pop();
        if delimiter_stack.is_empty() {
            return Ok(token_index);
        }
    }
    Err(problem_at_token(
        source_tokens.last().unwrap_or(opening_token),
        CompilationProblemReason::MacroDefinitionInvalid,
    ))
}

fn identifier_name(source_token: Option<&SourceToken>) -> Option<&str> {
    match source_token?.token_kind() {
        SourceTokenKind::IdentifierName(identifier_name) => Some(identifier_name.as_str()),
        _ => None,
    }
}

const fn opening_delimiter(token_kind: &SourceTokenKind) -> Option<Delimiter> {
    match token_kind {
        SourceTokenKind::LeftParenthesis => Some(Delimiter::Parenthesis),
        SourceTokenKind::LeftBrace => Some(Delimiter::Brace),
        SourceTokenKind::LeftBracket => Some(Delimiter::Bracket),
        _ => None,
    }
}

const fn closing_delimiter(token_kind: &SourceTokenKind) -> Option<Delimiter> {
    match token_kind {
        SourceTokenKind::RightParenthesis => Some(Delimiter::Parenthesis),
        SourceTokenKind::RightBrace => Some(Delimiter::Brace),
        SourceTokenKind::RightBracket => Some(Delimiter::Bracket),
        _ => None,
    }
}

const fn problem_at_token(
    source_token: &SourceToken,
    reason: CompilationProblemReason,
) -> CompilationProblem {
    problem_at_range(source_token.source_range(), reason)
}

const fn problem_at_range(
    source_range: SourceRange,
    reason: CompilationProblemReason,
) -> CompilationProblem {
    CompilationProblem::from_problem_at_range((source_range, reason))
}
