use std::collections::{hash_map::Entry, HashMap, HashSet};

use crate::{CompilationProblem, CompilationProblemReason, MacroExpansionFrame, SourceRange};

use super::{SourceToken, SourceTokenKind};

const MAX_MACRO_EXPANSION_DEPTH: usize = 64;
const MAX_MACRO_EXPANSION_OUTPUT_TOKENS: usize = 100_000;

mod macro_definition;

pub use macro_definition::{extract_macro_definitions, MacroCatalog};
use macro_definition::{find_group_end, MacroDefinition};

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
        let invocation_range = source_tokens[token_index]
            .source_range()
            .through(source_tokens[argument_close_index].source_range());
        let definition_tokens = expand_macro_definition(
            definition,
            &argument_tokens,
            invocation_range,
            source_tokens[token_index].macro_backtrace(),
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
    invocation_backtrace: &[MacroExpansionFrame],
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
    let macro_origin_id = state.next_macro_origin_id;
    state.next_macro_origin_id += 1;
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
                let expanded_token = token_with_expansion_frame(ExpandedTokenParts {
                    source_token: argument_token,
                    token_kind: argument_token.token_kind().clone(),
                    inherited_backtrace: argument_token.macro_backtrace(),
                    expansion_frame: &expansion_frame,
                    macro_origin_id,
                });
                account_generated_token(&expanded_token, state)?;
                expanded_tokens.push(expanded_token);
            }
            token_index += 2;
            continue;
        }
        let token_kind = identifier_name(Some(body_token))
            .filter(|_| !identifier_is_member_name(&definition.body, token_index))
            .and_then(|name| hygiene_names.get(name))
            .map_or_else(
                || body_token.token_kind().clone(),
                |hygienic_name| SourceTokenKind::IdentifierName(hygienic_name.clone()),
            );
        let expanded_token = token_with_expansion_frame(ExpandedTokenParts {
            source_token: body_token,
            token_kind,
            inherited_backtrace: invocation_backtrace,
            expansion_frame: &expansion_frame,
            macro_origin_id,
        });
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

struct ExpandedTokenParts<'source> {
    source_token: &'source SourceToken,
    token_kind: SourceTokenKind,
    inherited_backtrace: &'source [MacroExpansionFrame],
    expansion_frame: &'source MacroExpansionFrame,
    macro_origin_id: usize,
}

fn token_with_expansion_frame(expanded_token: ExpandedTokenParts<'_>) -> SourceToken {
    let mut macro_backtrace = expanded_token.inherited_backtrace.to_vec();
    macro_backtrace.push(expanded_token.expansion_frame.clone());
    SourceToken::from_token_at_origin((
        expanded_token.token_kind,
        expanded_token
            .source_token
            .source_range()
            .with_macro_origin_id(expanded_token.macro_origin_id),
        macro_backtrace,
    ))
}

fn identifier_is_member_name(source_tokens: &[SourceToken], token_index: usize) -> bool {
    matches!(
        token_index
            .checked_sub(1)
            .and_then(|previous_index| source_tokens.get(previous_index))
            .map(SourceToken::token_kind),
        Some(SourceTokenKind::Dot)
    )
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
