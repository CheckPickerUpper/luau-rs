use crate::{
    source_language::{
        parse_source_program::SourceProgramParser, ParsedComparisonOperation,
        ParsedComparisonOperator, ParsedEqualityOperation, ParsedEqualityOperator,
        ParsedExpression, ParsedFunctionCall, ParsedLiteral, ParsedLogicalNegation,
        ParsedLogicalOperation, ParsedLogicalOperator, ParsedNumericOperation,
        ParsedNumericOperator, ParsedRobloxRemoteOperation, ParsedRobloxRemoteOperationKind,
        SourceToken, SourceTokenKind,
    },
    CompilationProblem, SourceRange,
};

/// Parses value expressions and the source language's complete operator precedence ladder.
impl SourceProgramParser {
    /// Parses one complete value expression at the current token position.
    pub(super) fn parse_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        self.parse_disjunction_expression()
    }

    pub(super) fn parse_condition_expression(
        &mut self,
    ) -> Result<ParsedExpression, CompilationProblem> {
        let record_literals_were_allowed = self.record_literals_are_allowed;
        self.record_literals_are_allowed = false;
        let parsed_expression = self.parse_expression();
        self.record_literals_are_allowed = record_literals_were_allowed;
        parsed_expression
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
                Ok(SourceTokenKind::LeftBracket) => {
                    match self.take_required_symbol(&SourceTokenKind::LeftBracket) {
                        Ok(consumed_symbol) => drop(consumed_symbol),
                        Err(compilation_problem) => return Err(compilation_problem),
                    }
                    let index_expression = match self.parse_expression() {
                        Ok(index_expression) => index_expression,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    let right_bracket =
                        match self.take_required_symbol(&SourceTokenKind::RightBracket) {
                            Ok(right_bracket) => right_bracket,
                            Err(compilation_problem) => return Err(compilation_problem),
                        };
                    let expression_range = parsed_expression
                        .source_range()
                        .through(right_bracket.source_range());
                    parsed_expression = ParsedExpression::ArrayRead(
                        crate::source_language::ParsedArrayRead::from_read((
                            Box::new(parsed_expression),
                            Box::new(index_expression),
                            expression_range,
                        )),
                    );
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
            Ok(SourceTokenKind::LeftBracket) => self.parse_array_literal(),
            Ok(SourceTokenKind::Bang) => {
                let operator_token = match self.take_required_symbol(&SourceTokenKind::Bang) {
                    Ok(operator_token) => operator_token,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
                let negated_expression = match self.parse_unary_expression() {
                    Ok(negated_expression) => negated_expression,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
                let expression_range = operator_token
                    .source_range()
                    .through(negated_expression.source_range());
                Ok(ParsedExpression::LogicalNegation(
                    ParsedLogicalNegation::from_parts((
                        Box::new(negated_expression),
                        operator_token.source_range(),
                        expression_range,
                    )),
                ))
            }
            Ok(_) => self.parse_postfix_expression(),
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
        left_operand
            .source_range()
            .through(right_operand.source_range())
    }

    fn parse_postfix_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        let mut parsed_expression = match self.parse_primary_expression() {
            Ok(parsed_expression) => parsed_expression,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        loop {
            match self.current_token_kind() {
                Ok(SourceTokenKind::Dot) => {
                    match self.take_required_symbol(&SourceTokenKind::Dot) {
                        Ok(consumed_symbol) => drop(consumed_symbol),
                        Err(compilation_problem) => return Err(compilation_problem),
                    }
                    let (field_name, field_name_range) = match self.take_identifier_name() {
                        Ok(field_name_at_range) => field_name_at_range,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    let expression_range =
                        parsed_expression.source_range().through(field_name_range);
                    parsed_expression = ParsedExpression::FieldRead(
                        crate::source_language::ParsedFieldRead::from_read((
                            Box::new(parsed_expression),
                            field_name,
                            field_name_range,
                            expression_range,
                        )),
                    );
                }
                Ok(_) => break,
                Err(compilation_problem) => return Err(compilation_problem),
            }
        }
        Ok(parsed_expression)
    }

    fn parse_primary_expression(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        match self.current_token_kind() {
            Ok(SourceTokenKind::LeftParenthesis) => {
                match self.take_required_symbol(&SourceTokenKind::LeftParenthesis) {
                    Ok(consumed_symbol) => drop(consumed_symbol),
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                let record_literals_were_allowed = self.record_literals_are_allowed;
                self.record_literals_are_allowed = true;
                let grouped_expression = match self.parse_expression() {
                    Ok(grouped_expression) => grouped_expression,
                    Err(compilation_problem) => {
                        self.record_literals_are_allowed = record_literals_were_allowed;
                        return Err(compilation_problem);
                    }
                };
                self.record_literals_are_allowed = record_literals_were_allowed;
                match self.take_required_symbol(&SourceTokenKind::RightParenthesis) {
                    Ok(consumed_symbol) => drop(consumed_symbol),
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                Ok(grouped_expression)
            }
            Ok(SourceTokenKind::FunctionKeyword) => self.parse_function_literal(),
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
            SourceTokenKind::IdentifierName(identifier_name) => {
                let record_literals_are_allowed = self.record_literals_are_allowed;
                match self.current_token_kind() {
                    Ok(SourceTokenKind::DoubleColon) if identifier_name == "roblox" => {
                        self.parse_roblox_intrinsic(token_range)
                    }
                    Ok(SourceTokenKind::LeftBrace) if record_literals_are_allowed => {
                        self.parse_record_literal((identifier_name, token_range))
                    }
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
                        let call_range = token_range.through(right_parenthesis.source_range());
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
                }
            }
            _ => Err(CompilationProblem::from_problem_at_range((
                token_range,
                crate::CompilationProblemReason::SourceDoesNotFollowLanguageRules,
            ))),
        }
    }

    fn parse_roblox_intrinsic(
        &mut self,
        namespace_range: SourceRange,
    ) -> Result<ParsedExpression, CompilationProblem> {
        self.take_required_symbol(&SourceTokenKind::DoubleColon)?;
        let (intrinsic_name, _) = self.take_identifier_name()?;
        let remote_type = if intrinsic_name == "disconnect" {
            None
        } else {
            self.take_required_symbol(&SourceTokenKind::DoubleColon)?;
            self.take_required_symbol(&SourceTokenKind::LessThan)?;
            let remote_type = self.take_identifier_name()?;
            self.take_required_symbol(&SourceTokenKind::GreaterThan)?;
            Some(remote_type)
        };
        self.take_required_symbol(&SourceTokenKind::LeftParenthesis)?;
        match intrinsic_name.as_str() {
            "service" => {
                let Some((instance_type_name, instance_type_range)) = remote_type else {
                    return Err(self.problem_at_current_token());
                };
                let right_parenthesis =
                    self.take_required_symbol(&SourceTokenKind::RightParenthesis)?;
                Ok(ParsedExpression::RobloxServiceAcquisition {
                    service_type_name: instance_type_name,
                    service_type_range: instance_type_range,
                    expression_range: namespace_range.through(right_parenthesis.source_range()),
                })
            }
            "instance" => {
                let Some((instance_type_name, instance_type_range)) = remote_type else {
                    return Err(self.problem_at_current_token());
                };
                let mut arguments = self.parse_function_arguments()?.into_iter();
                let parent_expression = arguments.next().map(Box::new);
                if arguments.next().is_some() {
                    return Err(self.problem_at_current_token());
                }
                let right_parenthesis =
                    self.take_required_symbol(&SourceTokenKind::RightParenthesis)?;
                Ok(ParsedExpression::RobloxInstanceAcquisition {
                    instance_type_name,
                    instance_type_range,
                    parent_expression,
                    expression_range: namespace_range.through(right_parenthesis.source_range()),
                })
            }
            "wait_for_child" => {
                let Some((instance_type_name, instance_type_range)) = remote_type else {
                    return Err(self.problem_at_current_token());
                };
                let arguments = self.parse_function_arguments()?;
                let right_parenthesis =
                    self.take_required_symbol(&SourceTokenKind::RightParenthesis)?;
                if arguments.len() != 2 {
                    return Err(self.problem_at_current_token());
                }
                let mut arguments = arguments.into_iter();
                let Some(parent_expression) = arguments.next() else {
                    return Err(self.problem_at_current_token());
                };
                let Some(child_name_expression) = arguments.next() else {
                    return Err(self.problem_at_current_token());
                };
                Ok(ParsedExpression::RobloxInstanceWaitForChild {
                    instance_type_name,
                    instance_type_range,
                    parent_expression: Box::new(parent_expression),
                    child_name_expression: Box::new(child_name_expression),
                    expression_range: namespace_range.through(right_parenthesis.source_range()),
                })
            }
            "connect" | "disconnect" | "wait" | "fire_server" | "fire_client"
            | "fire_all_clients" | "invoke_server" | "invoke_client" | "set_callback" => {
                let arguments = self.parse_function_arguments()?;
                let right_parenthesis =
                    self.take_required_symbol(&SourceTokenKind::RightParenthesis)?;
                let operation_kind = match intrinsic_name.as_str() {
                    "connect" => ParsedRobloxRemoteOperationKind::Connect,
                    "disconnect" => ParsedRobloxRemoteOperationKind::Disconnect,
                    "wait" => ParsedRobloxRemoteOperationKind::Wait,
                    "fire_server" => ParsedRobloxRemoteOperationKind::FireServer,
                    "fire_client" => ParsedRobloxRemoteOperationKind::FireClient,
                    "fire_all_clients" => ParsedRobloxRemoteOperationKind::FireAllClients,
                    "invoke_server" => ParsedRobloxRemoteOperationKind::InvokeServer,
                    "invoke_client" => ParsedRobloxRemoteOperationKind::InvokeClient,
                    "set_callback" => ParsedRobloxRemoteOperationKind::SetCallback,
                    _ => return Err(self.problem_at_current_token()),
                };
                let Some(operation) = ParsedRobloxRemoteOperation::from_syntax((
                    operation_kind,
                    remote_type,
                    arguments,
                    namespace_range.through(right_parenthesis.source_range()),
                )) else {
                    return Err(self.problem_at_current_token());
                };
                Ok(ParsedExpression::RobloxRemoteOperation(operation))
            }
            _ => Err(self.problem_at_current_token()),
        }
    }

    fn parse_array_literal(&mut self) -> Result<ParsedExpression, CompilationProblem> {
        let left_bracket = self.take_required_symbol(&SourceTokenKind::LeftBracket)?;
        if matches!(self.current_token_kind(), Ok(SourceTokenKind::RightBracket)) {
            return Err(self.problem_at_current_token());
        }
        let mut element_expressions = Vec::new();
        loop {
            element_expressions.push(self.parse_expression()?);
            match self.current_token_kind()? {
                SourceTokenKind::Comma => {
                    drop(self.take_required_symbol(&SourceTokenKind::Comma)?);
                }
                SourceTokenKind::RightBracket => break,
                _ => return Err(self.problem_at_current_token()),
            }
        }
        let right_bracket = self.take_required_symbol(&SourceTokenKind::RightBracket)?;
        Ok(ParsedExpression::ArrayLiteral(
            crate::source_language::ParsedArrayLiteral::from_elements((
                element_expressions,
                left_bracket
                    .source_range()
                    .through(right_bracket.source_range()),
            )),
        ))
    }

    fn parse_record_literal(
        &mut self,
        record_name_at_range: (String, SourceRange),
    ) -> Result<ParsedExpression, CompilationProblem> {
        let (record_name, record_name_range) = record_name_at_range;
        match self.take_required_symbol(&SourceTokenKind::LeftBrace) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let mut field_initializers = Vec::new();
        loop {
            match self.current_token_kind() {
                Ok(SourceTokenKind::RightBrace) => break,
                Ok(_) => {
                    let (field_name, field_name_range) = match self.take_identifier_name() {
                        Ok(field_name_at_range) => field_name_at_range,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    match self.take_required_symbol(&SourceTokenKind::Colon) {
                        Ok(consumed_symbol) => drop(consumed_symbol),
                        Err(compilation_problem) => return Err(compilation_problem),
                    }
                    let initialized_value = match self.parse_expression() {
                        Ok(initialized_value) => initialized_value,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    field_initializers.push(
                        crate::source_language::ParsedRecordFieldInitializer::from_initializer((
                            field_name,
                            field_name_range,
                            initialized_value,
                        )),
                    );
                    match self.current_token_kind() {
                        Ok(SourceTokenKind::Comma) => {
                            match self.take_required_symbol(&SourceTokenKind::Comma) {
                                Ok(consumed_symbol) => drop(consumed_symbol),
                                Err(compilation_problem) => return Err(compilation_problem),
                            }
                        }
                        Ok(SourceTokenKind::RightBrace) => {}
                        Ok(_) => return Err(self.problem_at_current_token()),
                        Err(compilation_problem) => return Err(compilation_problem),
                    }
                }
                Err(compilation_problem) => return Err(compilation_problem),
            }
        }
        let right_brace = match self.take_required_symbol(&SourceTokenKind::RightBrace) {
            Ok(right_brace) => right_brace,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        let literal_range = record_name_range.through(right_brace.source_range());
        Ok(ParsedExpression::RecordLiteral(
            crate::source_language::ParsedRecordLiteral::from_literal((
                record_name,
                record_name_range,
                field_initializers,
                literal_range,
            )),
        ))
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
