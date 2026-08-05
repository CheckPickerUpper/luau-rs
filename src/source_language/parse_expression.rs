use crate::{
    source_language::{
        parse_source_program::SourceProgramParser, ParsedComparisonOperation,
        ParsedComparisonOperator, ParsedEqualityOperation, ParsedEqualityOperator,
        ParsedExpression, ParsedFunctionCall, ParsedLiteral, ParsedLogicalNegation,
        ParsedLogicalOperation, ParsedLogicalOperator, ParsedNumericOperation,
        ParsedNumericOperator, SourceToken, SourceTokenKind,
    },
    CompilationProblem, SourceRange,
};

/// Parses value expressions and the source language's complete operator precedence ladder.
impl SourceProgramParser {
    /// Parses one complete value expression at the current token position.
    pub(super) fn parse_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        self.parse_disjunction_expression()
    }

    fn parse_disjunction_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        let mut parsed_expression = match self.parse_conjunction_expression() {
            Ok(parsed_expression) => parsed_expression,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        loop {
            match self.current_token_kind() {
                Ok(SourceTokenKind::PipePipe) => {
                    let operator_token = match self.take_required_symbol(&SourceTokenKind::PipePipe)
                    {
                        Ok(operator_token) => operator_token,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    let right_operand = match self.parse_conjunction_expression() {
                        Ok(right_operand) => right_operand,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    parsed_expression = Self::make_logical_operation((
                        parsed_expression,
                        right_operand,
                        ParsedLogicalOperator::Disjunction,
                        operator_token.source_range(),
                    ));
                }
                Ok(_) => break,
                Err(compilation_problem) => return Err(compilation_problem),
            }
        }
        Ok(parsed_expression)
    }

    fn parse_conjunction_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        let mut parsed_expression = match self.parse_equality_expression() {
            Ok(parsed_expression) => parsed_expression,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        loop {
            match self.current_token_kind() {
                Ok(SourceTokenKind::AmpersandAmpersand) => {
                    let operator_token =
                        match self.take_required_symbol(&SourceTokenKind::AmpersandAmpersand) {
                            Ok(operator_token) => operator_token,
                            Err(compilation_problem) => return Err(compilation_problem),
                        };
                    let right_operand = match self.parse_equality_expression() {
                        Ok(right_operand) => right_operand,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    parsed_expression = Self::make_logical_operation((
                        parsed_expression,
                        right_operand,
                        ParsedLogicalOperator::Conjunction,
                        operator_token.source_range(),
                    ));
                }
                Ok(_) => break,
                Err(compilation_problem) => return Err(compilation_problem),
            }
        }
        Ok(parsed_expression)
    }

    fn parse_equality_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        let mut parsed_expression = match self.parse_comparison_expression() {
            Ok(parsed_expression) => parsed_expression,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        loop {
            let operator_kind = match self.current_token_kind() {
                Ok(SourceTokenKind::EqualEqual) => ParsedEqualityOperator::Equal,
                Ok(SourceTokenKind::BangEqual) => ParsedEqualityOperator::NotEqual,
                Ok(_) => break,
                Err(compilation_problem) => return Err(compilation_problem),
            };
            let operator_token = match self.take_next_token() {
                Ok(operator_token) => operator_token,
                Err(compilation_problem) => return Err(compilation_problem),
            };
            let right_operand = match self.parse_comparison_expression() {
                Ok(right_operand) => right_operand,
                Err(compilation_problem) => return Err(compilation_problem),
            };
            parsed_expression = Self::make_equality_operation((
                parsed_expression,
                right_operand,
                operator_kind,
                operator_token.source_range(),
            ));
        }
        Ok(parsed_expression)
    }

    fn parse_comparison_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        let mut parsed_expression = match self.parse_additive_expression() {
            Ok(parsed_expression) => parsed_expression,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        loop {
            let operator_kind = match self.current_token_kind() {
                Ok(SourceTokenKind::LessThan) => ParsedComparisonOperator::LessThan,
                Ok(SourceTokenKind::LessThanOrEqual) => ParsedComparisonOperator::LessThanOrEqual,
                Ok(SourceTokenKind::GreaterThan) => ParsedComparisonOperator::GreaterThan,
                Ok(SourceTokenKind::GreaterThanOrEqual) => {
                    ParsedComparisonOperator::GreaterThanOrEqual
                }
                Ok(_) => break,
                Err(compilation_problem) => return Err(compilation_problem),
            };
            let operator_token = match self.take_next_token() {
                Ok(operator_token) => operator_token,
                Err(compilation_problem) => return Err(compilation_problem),
            };
            let right_operand = match self.parse_additive_expression() {
                Ok(right_operand) => right_operand,
                Err(compilation_problem) => return Err(compilation_problem),
            };
            parsed_expression = Self::make_comparison_operation((
                parsed_expression,
                right_operand,
                operator_kind,
                operator_token.source_range(),
            ));
        }
        Ok(parsed_expression)
    }

    fn parse_additive_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        let mut parsed_expression = match self.parse_multiplicative_expression() {
            Ok(parsed_expression) => parsed_expression,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        loop {
            let operator_kind = match self.current_token_kind() {
                Ok(SourceTokenKind::Plus) => ParsedNumericOperator::Addition,
                Ok(SourceTokenKind::Minus) => ParsedNumericOperator::Subtraction,
                Ok(_) => break,
                Err(compilation_problem) => return Err(compilation_problem),
            };
            let operator_token = match self.take_next_token() {
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
        Ok(parsed_expression)
    }

    fn parse_multiplicative_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        let mut parsed_expression = match self.parse_unary_expression() {
            Ok(parsed_expression) => parsed_expression,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        loop {
            let operator_kind = match self.current_token_kind() {
                Ok(SourceTokenKind::Star) => ParsedNumericOperator::Multiplication,
                Ok(SourceTokenKind::Slash) => ParsedNumericOperator::Division,
                Ok(_) => break,
                Err(compilation_problem) => return Err(compilation_problem),
            };
            let operator_token = match self.take_next_token() {
                Ok(operator_token) => operator_token,
                Err(compilation_problem) => return Err(compilation_problem),
            };
            let right_operand = match self.parse_unary_expression() {
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
        Ok(parsed_expression)
    }

    fn parse_unary_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        match self.current_token_kind() {
            Ok(SourceTokenKind::Bang) => {
                let operator_token = match self.take_required_symbol(&SourceTokenKind::Bang) {
                    Ok(operator_token) => operator_token,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
                let negated_expression = match self.parse_unary_expression() {
                    Ok(negated_expression) => negated_expression,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
                let expression_range = SourceRange::from_byte_range((
                    operator_token.source_range().start_byte(),
                    negated_expression.source_range().end_byte(),
                ));
                Ok(ParsedExpression::LogicalNegation(
                    ParsedLogicalNegation::from_parts((
                        Box::new(negated_expression),
                        operator_token.source_range(),
                        expression_range,
                    )),
                ))
            }
            Ok(_) => self.parse_primary_expression(),
            Err(compilation_problem) => Err(compilation_problem),
        }
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
        let expression_range = Self::range_spanning_operands((&left_operand, &right_operand));
        ParsedExpression::NumericOperation(ParsedNumericOperation::from_parts((
            Box::new(left_operand),
            Box::new(right_operand),
            operator,
            operator_range,
            expression_range,
        )))
    }

    fn make_comparison_operation(
        operation_parts: (
            ParsedExpression,
            ParsedExpression,
            ParsedComparisonOperator,
            SourceRange,
        ),
    ) -> ParsedExpression {
        let (left_operand, right_operand, operator, operator_range) = operation_parts;
        let expression_range = Self::range_spanning_operands((&left_operand, &right_operand));
        ParsedExpression::ComparisonOperation(ParsedComparisonOperation::from_parts((
            Box::new(left_operand),
            Box::new(right_operand),
            operator,
            operator_range,
            expression_range,
        )))
    }

    fn make_equality_operation(
        operation_parts: (
            ParsedExpression,
            ParsedExpression,
            ParsedEqualityOperator,
            SourceRange,
        ),
    ) -> ParsedExpression {
        let (left_operand, right_operand, operator, operator_range) = operation_parts;
        let expression_range = Self::range_spanning_operands((&left_operand, &right_operand));
        ParsedExpression::EqualityOperation(ParsedEqualityOperation::from_parts((
            Box::new(left_operand),
            Box::new(right_operand),
            operator,
            operator_range,
            expression_range,
        )))
    }

    fn make_logical_operation(
        operation_parts: (
            ParsedExpression,
            ParsedExpression,
            ParsedLogicalOperator,
            SourceRange,
        ),
    ) -> ParsedExpression {
        let (left_operand, right_operand, operator, operator_range) = operation_parts;
        let expression_range = Self::range_spanning_operands((&left_operand, &right_operand));
        ParsedExpression::LogicalOperation(ParsedLogicalOperation::from_parts((
            Box::new(left_operand),
            Box::new(right_operand),
            operator,
            operator_range,
            expression_range,
        )))
    }

    const fn range_spanning_operands(
        operands: (&ParsedExpression, &ParsedExpression),
    ) -> SourceRange {
        let (left_operand, right_operand) = operands;
        SourceRange::from_byte_range((
            left_operand.source_range().start_byte(),
            right_operand.source_range().end_byte(),
        ))
    }

    fn parse_primary_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        match self.current_token_kind() {
            Ok(SourceTokenKind::LeftParenthesis) => {
                match self.take_required_symbol(&SourceTokenKind::LeftParenthesis) {
                    Ok(consumed_symbol) => drop(consumed_symbol),
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                let grouped_expression = match self.parse_expression() {
                    Ok(grouped_expression) => grouped_expression,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
                match self.take_required_symbol(&SourceTokenKind::RightParenthesis) {
                    Ok(consumed_symbol) => drop(consumed_symbol),
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                Ok(grouped_expression)
            }
            Ok(_) => self.parse_ungrouped_primary_expression(),
            Err(compilation_problem) => Err(compilation_problem),
        }
    }

    fn parse_ungrouped_primary_expression(
        &mut self,
    ) -> Result<ParsedExpression, CompilationProblem> {
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
                    match self.take_required_symbol(&SourceTokenKind::LeftParenthesis) {
                        Ok(consumed_symbol) => drop(consumed_symbol),
                        Err(compilation_problem) => return Err(compilation_problem),
                    }
                    let function_arguments = match self.parse_function_arguments() {
                        Ok(function_arguments) => function_arguments,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    let right_parenthesis =
                        match self.take_required_symbol(&SourceTokenKind::RightParenthesis) {
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
                            match self.take_required_symbol(&SourceTokenKind::Comma) {
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
            Ok(
                SourceTokenKind::IdentifierName(_)
                | SourceTokenKind::NumberLiteral(_)
                | SourceTokenKind::StringLiteral(_)
                | SourceTokenKind::BooleanLiteral(_),
            ) => match self.take_next_token() {
                Ok(source_token) => Ok(source_token),
                Err(compilation_problem) => Err(compilation_problem),
            },
            Ok(_) => Err(self.problem_at_current_token()),
            Err(compilation_problem) => Err(compilation_problem),
        }
    }
}
