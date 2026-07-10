use crate::{
    source_language::{
        parse_source_program::SourceProgramParser, ParsedExpression, ParsedFunctionCall,
        ParsedLiteral, ParsedNumericOperation, ParsedNumericOperator, SourceToken, SourceTokenKind,
    },
    CompilationProblem, SourceRange,
};

/// Parses value expressions, numeric precedence, and function argument lists.
impl SourceProgramParser {
    /// Parses one complete value expression at the current token position.
    pub(super) fn parse_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        self.parse_additive_expression()
    }

    fn parse_additive_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        let mut parsed_expression = match self.parse_multiplicative_expression() {
            Ok(parsed_expression) => parsed_expression,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        loop {
            match self.current_token_kind() {
                Ok(SourceTokenKind::Plus) | Ok(SourceTokenKind::Minus) => {
                    let (operator_kind, operator_token_kind) = match self.current_token_kind() {
                        Ok(SourceTokenKind::Plus) => {
                            (ParsedNumericOperator::Addition, SourceTokenKind::Plus)
                        }
                        Ok(SourceTokenKind::Minus) => {
                            (ParsedNumericOperator::Subtraction, SourceTokenKind::Minus)
                        }
                        Ok(_) => return Err(self.problem_at_current_token()),
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    let operator_token = match self.take_required_symbol(operator_token_kind) {
                        Ok(operator_token) => operator_token,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    let right_operand = match self.parse_multiplicative_expression() {
                        Ok(right_operand) => right_operand,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    parsed_expression = Self::make_numeric_operation((
                        parsed_expression,
                        right_operand,
                        operator_kind,
                        operator_token.source_range(),
                    ));
                }
                Ok(_) => break,
                Err(compilation_problem) => return Err(compilation_problem),
            }
        }
        Ok(parsed_expression)
    }

    fn parse_multiplicative_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        let mut parsed_expression = match self.parse_primary_expression() {
            Ok(parsed_expression) => parsed_expression,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        loop {
            match self.current_token_kind() {
                Ok(SourceTokenKind::Star) | Ok(SourceTokenKind::Slash) => {
                    let (operator_kind, operator_token_kind) = match self.current_token_kind() {
                        Ok(SourceTokenKind::Star) => {
                            (ParsedNumericOperator::Multiplication, SourceTokenKind::Star)
                        }
                        Ok(SourceTokenKind::Slash) => {
                            (ParsedNumericOperator::Division, SourceTokenKind::Slash)
                        }
                        Ok(_) => return Err(self.problem_at_current_token()),
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    let operator_token = match self.take_required_symbol(operator_token_kind) {
                        Ok(operator_token) => operator_token,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    let right_operand = match self.parse_primary_expression() {
                        Ok(right_operand) => right_operand,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    parsed_expression = Self::make_numeric_operation((
                        parsed_expression,
                        right_operand,
                        operator_kind,
                        operator_token.source_range(),
                    ));
                }
                Ok(_) => break,
                Err(compilation_problem) => return Err(compilation_problem),
            }
        }
        Ok(parsed_expression)
    }

    fn make_numeric_operation(
        operation_parts: (
            ParsedExpression,
            ParsedExpression,
            ParsedNumericOperator,
            SourceRange,
        ),
    ) -> ParsedExpression {
        let (left_operand, right_operand, operator, operator_range) = operation_parts;
        let expression_range = SourceRange::from_byte_range((
            left_operand.source_range().start_byte(),
            right_operand.source_range().end_byte(),
        ));
        ParsedExpression::NumericOperation(ParsedNumericOperation::from_parts((
            Box::new(left_operand),
            Box::new(right_operand),
            operator,
            operator_range,
            expression_range,
        )))
    }

    fn parse_primary_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        let source_token = match self.take_required_expression_token() {
            Ok(source_token) => source_token,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        let (token_kind, token_range) = source_token.into_token_at_range();
        match token_kind {
            SourceTokenKind::NumberLiteral(number_literal) => Ok(ParsedExpression::NumberLiteral(
                ParsedLiteral::from_spelling_at_range((number_literal, token_range)),
            )),
            SourceTokenKind::StringLiteral(string_literal) => Ok(ParsedExpression::StringLiteral(
                ParsedLiteral::from_spelling_at_range((string_literal, token_range)),
            )),
            SourceTokenKind::BooleanLiteral(boolean_literal) => {
                Ok(ParsedExpression::BooleanLiteral {
                    boolean_literal,
                    literal_range: token_range,
                })
            }
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
            Ok(SourceTokenKind::IdentifierName(_))
            | Ok(SourceTokenKind::NumberLiteral(_))
            | Ok(SourceTokenKind::StringLiteral(_))
            | Ok(SourceTokenKind::BooleanLiteral(_)) => match self.take_next_token() {
                Ok(source_token) => Ok(source_token),
                Err(compilation_problem) => Err(compilation_problem),
            },
            Ok(_) => Err(self.problem_at_current_token()),
            Err(compilation_problem) => Err(compilation_problem),
        }
    }
}
