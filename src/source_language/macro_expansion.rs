use std::collections::{hash_map::Entry, HashMap, HashSet};

use crate::{CompilationProblem, CompilationProblemReason, MacroExpansionFrame, SourceRange};

use super::{SourceToken, SourceTokenKind};

const MAX_MACRO_EXPANSION_DEPTH: usize = 64;
const MAX_MACRO_EXPANSION_OUTPUT_TOKENS: usize = 100_000;

#[derive(Clone)]
struct MacroDefinition {
    name: String,
    parameter: Option<String>,
    body: Vec<SourceToken>,
    definition_range: SourceRange,
    definition_module: Option<String>,
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
            Entry::Occupied(_) => Err(macro_problem_at_range(
                definition.definition_range,
                CompilationProblemReason::MacroMatcherAmbiguous,
            )),
        }
    }

    fn definition(&self, name: &str) -> Option<&MacroDefinition> {
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
            return Err(macro_problem_at_token(
                source_token,
                CompilationProblemReason::SourceDoesNotFollowLanguageRules,
            ));
        }
    }
    Ok((catalog, remaining_tokens))
}

pub fn expand_macros(
    source_tokens: &[SourceToken],
    catalog: &MacroCatalog,
    invocation_module: Option<&str>,
) -> Result<Vec<SourceToken>, CompilationProblem> {
    let mut state = ExpansionState::default();
    expand_token_stream(source_tokens, catalog, invocation_module, 0, &mut state)
}

#[derive(Default)]
struct ExpansionState {
    next_hygiene_id: usize,
    generated_token_count: usize,
    next_macro_origin_id: usize,
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
        return Err(macro_problem_at_range(
            SourceRange::from_byte_range((0, 0)),
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    let Some(name_token) = source_tokens.get(macro_index + 1) else {
        return Err(macro_problem_at_token(
            macro_token,
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    let Some(name) = identifier_name(Some(name_token)) else {
        return Err(macro_problem_at_token(
            name_token,
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    let parameter_open_index = macro_index + 2;
    let Some(parameter_open_token) = source_tokens.get(parameter_open_index) else {
        return Err(macro_problem_at_token(
            name_token,
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    if !matches!(
        parameter_open_token.token_kind(),
        SourceTokenKind::LeftParenthesis
    ) {
        return Err(macro_problem_at_token(
            parameter_open_token,
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    }
    let parameter_close_index = find_group_end(source_tokens, parameter_open_index)?;
    let body_open_index = parameter_close_index + 1;
    let Some(body_open_token) = source_tokens.get(body_open_index) else {
        return Err(macro_problem_at_range(
            parameter_open_token.source_range(),
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    if !matches!(body_open_token.token_kind(), SourceTokenKind::LeftBrace) {
        return Err(macro_problem_at_token(
            body_open_token,
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    }
    let body_close_index = find_group_end(source_tokens, body_open_index)?;
    let parameter =
        parse_macro_parameter(&source_tokens[parameter_open_index + 1..parameter_close_index])?;
    let body = source_tokens[body_open_index + 1..body_close_index].to_vec();
    let Some(body_close_token) = source_tokens.get(body_close_index) else {
        return Err(macro_problem_at_token(
            body_open_token,
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    let definition_range = SourceRange::from_byte_range((
        macro_token.source_range().start_byte(),
        body_close_token.source_range().end_byte(),
    ));
    Ok((
        MacroDefinition {
            name: name.to_owned(),
            parameter,
            body,
            definition_range,
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
        return Err(macro_problem_at_range(
            SourceRange::from_byte_range((0, 0)),
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    Err(macro_problem_at_token(
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
        return Err(macro_problem_at_token(
            source_token,
            CompilationProblemReason::SourceDoesNotFollowLanguageRules,
        ));
    }
    delimiter_stack.pop();
    Ok(())
}

fn find_group_end(
    source_tokens: &[SourceToken],
    opening_index: usize,
) -> Result<usize, CompilationProblem> {
    let Some(opening_token) = source_tokens.get(opening_index) else {
        return Err(macro_problem_at_range(
            SourceRange::from_byte_range((0, 0)),
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    let Some(opening_kind) = opening_delimiter(opening_token.token_kind()) else {
        return Err(macro_problem_at_token(
            opening_token,
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    let mut delimiter_stack = Vec::new();
    delimiter_stack.push(opening_kind);
    for (token_index, source_token) in source_tokens.iter().enumerate().skip(opening_index + 1) {
        if let Some(delimiter) = opening_delimiter(source_token.token_kind()) {
            delimiter_stack.push(delimiter);
            continue;
        }
        let Some(delimiter) = closing_delimiter(source_token.token_kind()) else {
            continue;
        };
        if delimiter_stack.last().copied() != Some(delimiter) {
            return Err(macro_problem_at_token(
                source_token,
                CompilationProblemReason::MacroDefinitionInvalid,
            ));
        }
        delimiter_stack.pop();
        if delimiter_stack.is_empty() {
            return Ok(token_index);
        }
    }
    let Some(last_token) = source_tokens.last() else {
        return Err(macro_problem_at_token(
            opening_token,
            CompilationProblemReason::MacroDefinitionInvalid,
        ));
    };
    Err(macro_problem_at_token(
        last_token,
        CompilationProblemReason::MacroDefinitionInvalid,
    ))
}

fn expand_token_stream(
    source_tokens: &[SourceToken],
    catalog: &MacroCatalog,
    invocation_module: Option<&str>,
    expansion_depth: usize,
    state: &mut ExpansionState,
) -> Result<Vec<SourceToken>, CompilationProblem> {
    let mut expanded_tokens = Vec::with_capacity(source_tokens.len());
    let mut token_index = 0;
    while token_index < source_tokens.len() {
        let Some(macro_name) = macro_invocation_name(source_tokens, token_index) else {
            expanded_tokens.push(source_tokens[token_index].clone());
            token_index += 1;
            continue;
        };
        let macro_name = macro_name.to_owned();
        let Some(definition) = catalog.definition(&macro_name) else {
            return Err(macro_problem_at_token(
                &source_tokens[token_index],
                CompilationProblemReason::UnknownMacro,
            ));
        };
        if expansion_depth >= MAX_MACRO_EXPANSION_DEPTH {
            return Err(macro_problem_at_token(
                &source_tokens[token_index],
                CompilationProblemReason::MacroExpansionDepthExceeded,
            ));
        }
        let argument_open_index = token_index + 2;
        let argument_close_index = find_group_end(source_tokens, argument_open_index)?;
        let argument_tokens = source_tokens[argument_open_index + 1..argument_close_index].to_vec();
        if !macro_argument_shape_matches(definition, &argument_tokens) {
            return Err(macro_problem_at_token(
                &source_tokens[token_index],
                CompilationProblemReason::MacroArgumentShapeMismatch,
            ));
        }
        let invocation_range = SourceRange::from_byte_range((
            source_tokens[token_index].source_range().start_byte(),
            source_tokens[argument_close_index]
                .source_range()
                .end_byte(),
        ));
        let definition_tokens = expand_macro_definition(
            definition,
            &argument_tokens,
            invocation_range,
            invocation_module,
            state,
        )?;
        let nested_tokens = expand_token_stream(
            &definition_tokens,
            catalog,
            invocation_module,
            expansion_depth + 1,
            state,
        )?;
        let expansion_owns_statement_terminator = matches!(
            nested_tokens.last().map(SourceToken::token_kind),
            Some(SourceTokenKind::Semicolon)
        );
        let invocation_has_statement_terminator = matches!(
            source_tokens
                .get(argument_close_index + 1)
                .map(SourceToken::token_kind),
            Some(SourceTokenKind::Semicolon)
        );
        expanded_tokens.extend(nested_tokens);
        token_index = argument_close_index
            + 1
            + usize::from(
                expansion_owns_statement_terminator && invocation_has_statement_terminator,
            );
    }
    Ok(expanded_tokens)
}

const fn macro_argument_shape_matches(
    definition: &MacroDefinition,
    argument_tokens: &[SourceToken],
) -> bool {
    match definition.parameter {
        Some(_) => !argument_tokens.is_empty(),
        None => argument_tokens.is_empty(),
    }
}

fn expand_macro_definition(
    definition: &MacroDefinition,
    argument_tokens: &[SourceToken],
    invocation_range: SourceRange,
    invocation_module: Option<&str>,
    state: &mut ExpansionState,
) -> Result<Vec<SourceToken>, CompilationProblem> {
    let expansion_frame = MacroExpansionFrame::from_expansion((
        definition.name.clone(),
        definition.definition_module.clone(),
        definition.definition_range,
        invocation_module.map(str::to_owned),
        invocation_range,
    ));
    let hygiene_names = hygienic_local_names(definition, argument_tokens, state);
    let mut expanded_tokens = Vec::new();
    let mut token_index = 0;
    while token_index < definition.body.len() {
        let body_token = &definition.body[token_index];
        if matches!(body_token.token_kind(), SourceTokenKind::Dollar) {
            let Some(parameter_name) = definition.parameter.as_deref() else {
                return Err(macro_problem_with_frame(
                    body_token,
                    CompilationProblemReason::MacroDefinitionInvalid,
                    &expansion_frame,
                ));
            };
            let Some(argument_name) = identifier_name(definition.body.get(token_index + 1)) else {
                return Err(macro_problem_with_frame(
                    body_token,
                    CompilationProblemReason::MacroDefinitionInvalid,
                    &expansion_frame,
                ));
            };
            if argument_name != parameter_name {
                return Err(macro_problem_with_frame(
                    body_token,
                    CompilationProblemReason::MacroDefinitionInvalid,
                    &expansion_frame,
                ));
            }
            for argument_token in argument_tokens {
                let expanded_token = token_with_expansion_frame(
                    argument_token,
                    argument_token.token_kind().clone(),
                    &expansion_frame,
                    state,
                );
                account_generated_token(&expanded_token, state)?;
                expanded_tokens.push(expanded_token);
            }
            token_index += 2;
            continue;
        }
        let token_kind = identifier_name(Some(body_token))
            .and_then(|name| hygiene_names.get(name))
            .map_or_else(
                || body_token.token_kind().clone(),
                |hygienic_name| SourceTokenKind::IdentifierName(hygienic_name.clone()),
            );
        let expanded_token =
            token_with_expansion_frame(body_token, token_kind, &expansion_frame, state);
        account_generated_token(&expanded_token, state)?;
        expanded_tokens.push(expanded_token);
        token_index += 1;
    }
    Ok(expanded_tokens)
}

fn hygienic_local_names(
    definition: &MacroDefinition,
    argument_tokens: &[SourceToken],
    state: &mut ExpansionState,
) -> HashMap<String, String> {
    let mut reserved_names = HashSet::new();
    collect_identifier_names(&definition.body, &mut reserved_names);
    collect_identifier_names(argument_tokens, &mut reserved_names);
    let mut hygienic_names = HashMap::new();
    for (token_index, source_token) in definition.body.iter().enumerate() {
        if !matches!(source_token.token_kind(), SourceTokenKind::LetKeyword) {
            continue;
        }
        let mut binding_index = token_index + 1;
        if matches!(
            definition
                .body
                .get(binding_index)
                .map(SourceToken::token_kind),
            Some(SourceTokenKind::MutKeyword)
        ) {
            binding_index += 1;
        }
        let Some(binding_name) = identifier_name(definition.body.get(binding_index)) else {
            continue;
        };
        if let Entry::Vacant(entry) = hygienic_names.entry(binding_name.to_owned()) {
            let original_name = entry.key().clone();
            let hygienic_name =
                fresh_hygienic_name(&definition.name, &original_name, &mut reserved_names, state);
            entry.insert(hygienic_name);
        }
    }
    hygienic_names
}

fn collect_identifier_names(source_tokens: &[SourceToken], names: &mut HashSet<String>) {
    for source_token in source_tokens {
        if let Some(identifier_name) = identifier_name(Some(source_token)) {
            names.insert(identifier_name.to_owned());
        }
    }
}

fn fresh_hygienic_name(
    macro_name: &str,
    original_name: &str,
    reserved_names: &mut HashSet<String>,
    state: &mut ExpansionState,
) -> String {
    loop {
        let candidate = format!(
            "__macro_{macro_name}_{}_{}",
            state.next_hygiene_id, original_name
        );
        state.next_hygiene_id += 1;
        if reserved_names.insert(candidate.clone()) {
            return candidate;
        }
    }
}

fn account_generated_token(
    source_token: &SourceToken,
    state: &mut ExpansionState,
) -> Result<(), CompilationProblem> {
    let Some(next_count) = state.generated_token_count.checked_add(1) else {
        return Err(macro_problem_at_token(
            source_token,
            CompilationProblemReason::MacroExpansionOutputLimitExceeded,
        ));
    };
    if next_count > MAX_MACRO_EXPANSION_OUTPUT_TOKENS {
        return Err(macro_problem_at_token(
            source_token,
            CompilationProblemReason::MacroExpansionOutputLimitExceeded,
        ));
    }
    state.generated_token_count = next_count;
    Ok(())
}

fn token_with_expansion_frame(
    source_token: &SourceToken,
    token_kind: SourceTokenKind,
    expansion_frame: &MacroExpansionFrame,
    state: &mut ExpansionState,
) -> SourceToken {
    let mut macro_backtrace = source_token.macro_backtrace().to_vec();
    macro_backtrace.push(expansion_frame.clone());
    let macro_origin_id = state.next_macro_origin_id;
    state.next_macro_origin_id += 1;
    SourceToken::from_token_at_origin((
        token_kind,
        source_token
            .source_range()
            .with_macro_origin_id(macro_origin_id),
        macro_backtrace,
    ))
}

fn macro_invocation_name(source_tokens: &[SourceToken], token_index: usize) -> Option<&str> {
    if !matches!(
        source_tokens
            .get(token_index + 1)
            .map(SourceToken::token_kind),
        Some(SourceTokenKind::Bang)
    ) || !matches!(
        source_tokens
            .get(token_index + 2)
            .map(SourceToken::token_kind),
        Some(SourceTokenKind::LeftParenthesis)
    ) {
        return None;
    }
    identifier_name(source_tokens.get(token_index))
}

fn identifier_name(source_token: Option<&SourceToken>) -> Option<&str> {
    source_token.and_then(|source_token| match source_token.token_kind() {
        SourceTokenKind::IdentifierName(identifier_name) => Some(identifier_name.as_str()),
        _ => None,
    })
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

fn macro_problem_at_token(
    source_token: &SourceToken,
    reason: CompilationProblemReason,
) -> CompilationProblem {
    CompilationProblem::from_problem_at_origin((
        source_token.source_range(),
        reason,
        source_token.macro_backtrace().to_vec(),
    ))
}

fn macro_problem_with_frame(
    source_token: &SourceToken,
    reason: CompilationProblemReason,
    expansion_frame: &MacroExpansionFrame,
) -> CompilationProblem {
    let mut macro_backtrace = source_token.macro_backtrace().to_vec();
    macro_backtrace.push(expansion_frame.clone());
    CompilationProblem::from_problem_at_origin((
        source_token.source_range(),
        reason,
        macro_backtrace,
    ))
}

const fn macro_problem_at_range(
    source_range: SourceRange,
    reason: CompilationProblemReason,
) -> CompilationProblem {
    CompilationProblem::from_problem_at_range((source_range, reason))
}
