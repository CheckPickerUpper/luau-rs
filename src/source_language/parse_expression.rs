use crate::{
    source_language::{
        parse_source_program::SourceProgramParser, ParsedExpression, ParsedFunctionCall,
        SourceToken, SourceTokenKind,
    },
    CompilationProblem, SourceRange,
};

/// Parses value expressions, additions, and function argument lists.
impl SourceProgramParser {
    /// Parses one complete value expression at the current token position.
    pub(super) fn parse_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        let mut parsed_expression = match self.parse_primary_expression() {
            Ok(parsed_expression) => parsed_expression,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        loop {
            match self.current_token_kind() {
                Ok(SourceTokenKind::Plus) => {
                    let plus_token = match self.take_required_symbol(SourceTokenKind::Plus) {
                        Ok(plus_token) => plus_token,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    let right_operand = match self.parse_primary_expression() {
                        Ok(right_operand) => right_operand,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    let addition_range = SourceRange::from_byte_range((
                        parsed_expression.source_range().start_byte(),
                        right_operand.source_range().end_byte(),
                    ));
                    parsed_expression = ParsedExpression::Addition {
                        left_operand: Box::new(parsed_expression),
                        right_operand: Box::new(right_operand),
                        operator_range: plus_token.source_range(),
                        addition_range,
                    };
                }
                Ok(_) => break,
                Err(compilation_problem) => return Err(compilation_problem),
            }
        }
        Ok(parsed_expression)
    }

    fn parse_primary_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        let source_token = match self.take_required_expression_token() {
            Ok(source_token) => source_token,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        let (token_kind, token_range) = source_token.into_token_at_range();
        match token_kind {
            SourceTokenKind::NumberLiteral(number_literal) => Ok(ParsedExpression::NumberLiteral {
                number_literal,
                literal_range: token_range,
            }),
            SourceTokenKind::IdentifierName(identifier_name) => match self.current_token_kind() {
                Ok(SourceTokenKind::LeftParenthesis) => {
                    match self.take_required_symbol(SourceTokenKind::LeftParenthesis) {
                        Ok(consumed_symbol) => drop(consumed_symbol),
                        Err(compilation_problem) => return Err(compilation_problem),
                    }
                    let function_arguments = match self.parse_function_arguments() {
                        Ok(function_arguments) => function_arguments,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    let right_parenthesis =
                        match self.take_required_symbol(SourceTokenKind::RightParenthesis) {
                            Ok(right_parenthesis) => right_parenthesis,
                            Err(compilation_problem) => return Err(compilation_problem),
                        };
                    let call_range = SourceRange::from_byte_range((
                        token_range.start_byte(),
                        right_parenthesis.source_range().end_byte(),
                    ));
                    Ok(ParsedExpression::FunctionCall(
                        ParsedFunctionCall::from_call((
                            identifier_name,
                            token_range,
                            function_arguments,
                            call_range,
                        )),
                    ))
                }
                Ok(_) => Ok(ParsedExpression::NameReference {
                    referenced_name: identifier_name,
                    name_range: token_range,
                }),
                Err(compilation_problem) => Err(compilation_problem),
            },
            _ => Err(CompilationProblem::from_problem_at_range((
                token_range,
                crate::CompilationProblemReason::SourceDoesNotFollowLanguageRules,
            ))),
        }
    }

    fn parse_function_arguments(&mut self) -> Result<Vec<ParsedExpression>, CompilationProblem> {
        let mut function_arguments = Vec::new();
        loop {
            match self.current_token_kind() {
                Ok(SourceTokenKind::RightParenthesis) => break,
                Ok(_) => {
                    match self.parse_expression() {
                        Ok(function_argument) => function_arguments.push(function_argument),
                        Err(compilation_problem) => return Err(compilation_problem),
                    }
                    match self.current_token_kind() {
                        Ok(SourceTokenKind::Comma) => {
                            match self.take_required_symbol(SourceTokenKind::Comma) {
                                Ok(consumed_symbol) => drop(consumed_symbol),
                                Err(compilation_problem) => return Err(compilation_problem),
                            }
                        }
                        Ok(SourceTokenKind::RightParenthesis) => {}
                        Ok(_) => return Err(self.problem_at_current_token()),
                        Err(compilation_problem) => return Err(compilation_problem),
                    }
                }
                Err(compilation_problem) => return Err(compilation_problem),
            }
        }
        Ok(function_arguments)
    }

    fn take_required_expression_token(&mut self) -> Result<SourceToken, CompilationProblem> {
        match self.current_token_kind() {
            Ok(SourceTokenKind::IdentifierName(_)) | Ok(SourceTokenKind::NumberLiteral(_)) => {
                match self.take_next_token() {
                    Ok(source_token) => Ok(source_token),
                    Err(compilation_problem) => Err(compilation_problem),
                }
            }
            Ok(_) => Err(self.problem_at_current_token()),
            Err(compilation_problem) => Err(compilation_problem),
        }
    }
}
