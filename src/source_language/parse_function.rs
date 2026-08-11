use crate::{
    source_language::{
        parse_source_program::SourceProgramParser, ParsedExpression, ParsedFunction,
        ParsedFunctionBody, ParsedFunctionLiteral, ParsedIfElse, ParsedParameter, ParsedStatement,
        ParsedValueType, ParsedWhileLoop, SourceTokenKind,
    },
    CompilationProblem,
};

enum ParsedLocalMutability {
    Immutable,
    Mutable,
}

struct FlattenedRecordFieldAssignmentTarget {
    root_binding_name: String,
    root_binding_range: crate::SourceRange,
    reversed_steps: Vec<crate::source_language::ParsedPlaceStep>,
}

/// Parses function declarations, bodies, parameters, and non-return statements.
impl SourceProgramParser {
    pub(super) fn parse_function_literal(
        &mut self,
    ) -> Result<ParsedExpression, CompilationProblem> {
        let function_keyword_range = self
            .take_required_symbol(&SourceTokenKind::FunctionKeyword)?
            .source_range();
        self.take_required_symbol(&SourceTokenKind::LeftParenthesis)?;
        let function_parameters = self.parse_function_parameters()?;
        self.take_required_symbol(&SourceTokenKind::RightParenthesis)?;
        let returned_value_type = match self.current_token_kind()? {
            SourceTokenKind::Arrow => {
                self.take_required_symbol(&SourceTokenKind::Arrow)?;
                self.parse_value_type()?
            }
            _ => ParsedValueType::NoReturnedValues,
        };
        self.take_required_symbol(&SourceTokenKind::LeftBrace)?;
        let function_body = self.parse_function_body()?;
        let right_brace = self.take_required_symbol(&SourceTokenKind::RightBrace)?;
        Ok(ParsedExpression::FunctionLiteral(
            ParsedFunctionLiteral::from_parts((
                function_parameters,
                returned_value_type,
                function_body,
                function_keyword_range.through(right_brace.source_range()),
            )),
        ))
    }

    /// Parses one complete function declaration at the current token position.
    pub(super) fn parse_function(&mut self) -> Result<ParsedFunction, CompilationProblem> {
        let visibility = match self.current_token_kind() {
            Ok(SourceTokenKind::PublicKeyword) => {
                match self.take_required_symbol(&SourceTokenKind::PublicKeyword) {
                    Ok(consumed_symbol) => drop(consumed_symbol),
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                crate::source_language::ParsedFunctionVisibility::Public
            }
            Ok(_) => crate::source_language::ParsedFunctionVisibility::Private,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_required_symbol(&SourceTokenKind::FunctionKeyword) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let (function_name, function_name_range) = match self.take_declaration_name() {
            Ok(declaration_name) => declaration_name,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_required_symbol(&SourceTokenKind::LeftParenthesis) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let function_parameters = match self.parse_function_parameters() {
            Ok(function_parameters) => function_parameters,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_required_symbol(&SourceTokenKind::RightParenthesis) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let returned_value_type = match self.current_token_kind() {
            Ok(SourceTokenKind::Arrow) => {
                match self.take_required_symbol(&SourceTokenKind::Arrow) {
                    Ok(consumed_symbol) => drop(consumed_symbol),
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                match self.parse_value_type() {
                    Ok(value_type) => value_type,
                    Err(compilation_problem) => return Err(compilation_problem),
                }
            }
            Ok(_) => ParsedValueType::NoReturnedValues,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_required_symbol(&SourceTokenKind::LeftBrace) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let function_body = match self.parse_function_body() {
            Ok(function_body) => function_body,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_required_symbol(&SourceTokenKind::RightBrace) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        Ok(ParsedFunction::from_declaration((
            visibility,
            function_name,
            function_name_range,
            function_parameters,
            returned_value_type,
            function_body,
        )))
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<ParsedParameter>, CompilationProblem> {
        let mut function_parameters = Vec::new();
        loop {
            match self.current_token_kind() {
                Ok(SourceTokenKind::RightParenthesis) => break,
                Ok(_) => {
                    let (parameter_name, parameter_name_range) = match self.take_declaration_name()
                    {
                        Ok(declaration_name) => declaration_name,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    match self.take_required_symbol(&SourceTokenKind::Colon) {
                        Ok(consumed_symbol) => drop(consumed_symbol),
                        Err(compilation_problem) => return Err(compilation_problem),
                    }
                    let value_type = match self.parse_value_type() {
                        Ok(value_type) => value_type,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    function_parameters.push(ParsedParameter::from_declaration((
                        parameter_name,
                        parameter_name_range,
                        value_type,
                    )));
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
        Ok(function_parameters)
    }

    fn parse_value_type(&mut self) -> Result<ParsedValueType, CompilationProblem> {
        if matches!(
            self.current_token_kind(),
            Ok(SourceTokenKind::FunctionKeyword)
        ) {
            self.take_required_symbol(&SourceTokenKind::FunctionKeyword)?;
            self.take_required_symbol(&SourceTokenKind::LeftParenthesis)?;
            let mut parameter_types = Vec::new();
            loop {
                if matches!(
                    self.current_token_kind()?,
                    SourceTokenKind::RightParenthesis
                ) {
                    break;
                }
                parameter_types.push(self.parse_value_type()?);
                match self.current_token_kind()? {
                    SourceTokenKind::Comma => {
                        self.take_required_symbol(&SourceTokenKind::Comma)?;
                    }
                    SourceTokenKind::RightParenthesis => {}
                    _ => return Err(self.problem_at_current_token()),
                }
            }
            self.take_required_symbol(&SourceTokenKind::RightParenthesis)?;
            let returned_value_type =
                if matches!(self.current_token_kind()?, SourceTokenKind::Arrow) {
                    self.take_required_symbol(&SourceTokenKind::Arrow)?;
                    self.parse_value_type()?
                } else {
                    ParsedValueType::NoReturnedValues
                };
            return Ok(ParsedValueType::Function {
                parameter_types,
                returned_value_type: Box::new(returned_value_type),
            });
        }
        if matches!(self.current_token_kind(), Ok(SourceTokenKind::LeftBracket)) {
            match self.take_required_symbol(&SourceTokenKind::LeftBracket) {
                Ok(consumed_symbol) => drop(consumed_symbol),
                Err(compilation_problem) => return Err(compilation_problem),
            }
            let element_type = match self.parse_value_type() {
                Ok(element_type) => element_type,
                Err(compilation_problem) => return Err(compilation_problem),
            };
            match self.take_required_symbol(&SourceTokenKind::RightBracket) {
                Ok(consumed_symbol) => drop(consumed_symbol),
                Err(compilation_problem) => return Err(compilation_problem),
            }
            return Ok(ParsedValueType::Array(Box::new(element_type)));
        }
        let (type_name, type_range) = match self.take_identifier_name() {
            Ok(type_name_at_range) => type_name_at_range,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match type_name.as_str() {
            "number" => Ok(ParsedValueType::Number),
            "string" => Ok(ParsedValueType::String),
            "boolean" => Ok(ParsedValueType::Boolean),
            _ => Ok(ParsedValueType::NamedRecord {
                record_name: type_name,
                record_name_range: type_range,
            }),
        }
    }

    pub(super) fn parse_record_declaration(
        &mut self,
    ) -> Result<crate::source_language::ParsedRecordDeclaration, CompilationProblem> {
        match self.take_required_symbol(&SourceTokenKind::StructKeyword) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let (record_name, record_name_range) = match self.take_declaration_name() {
            Ok(declaration_name) => declaration_name,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_required_symbol(&SourceTokenKind::LeftBrace) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let mut record_fields = Vec::new();
        loop {
            match self.current_token_kind() {
                Ok(SourceTokenKind::RightBrace) => break,
                Ok(_) => {
                    let (field_name, field_name_range) = match self.take_declaration_name() {
                        Ok(declaration_name) => declaration_name,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    match self.take_required_symbol(&SourceTokenKind::Colon) {
                        Ok(consumed_symbol) => drop(consumed_symbol),
                        Err(compilation_problem) => return Err(compilation_problem),
                    }
                    let value_type = match self.parse_value_type() {
                        Ok(value_type) => value_type,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    record_fields.push(
                        crate::source_language::ParsedRecordField::from_declaration((
                            field_name,
                            field_name_range,
                            value_type,
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
        match self.take_required_symbol(&SourceTokenKind::RightBrace) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        Ok(
            crate::source_language::ParsedRecordDeclaration::from_declaration((
                record_name,
                record_name_range,
                record_fields,
            )),
        )
    }

    fn parse_function_body(&mut self) -> Result<ParsedFunctionBody, CompilationProblem> {
        let mut function_statements = Vec::new();
        loop {
            match self.current_token_kind() {
                Ok(SourceTokenKind::RightBrace) => {
                    return Ok(ParsedFunctionBody::from_statements(function_statements));
                }
                Ok(_) => match self.parse_non_return_statement() {
                    Ok(parsed_statement) => function_statements.push(parsed_statement),
                    Err(compilation_problem) => return Err(compilation_problem),
                },
                Err(compilation_problem) => return Err(compilation_problem),
            }
        }
    }

    fn parse_non_return_statement(&mut self) -> Result<ParsedStatement, CompilationProblem> {
        match self.current_token_kind() {
            Ok(SourceTokenKind::ReturnKeyword) => self.parse_return_statement(),
            Ok(SourceTokenKind::BreakKeyword) => self.parse_loop_exit_statement(true),
            Ok(SourceTokenKind::ContinueKeyword) => self.parse_loop_exit_statement(false),
            Ok(SourceTokenKind::IfKeyword) => self.parse_if_else_statement(),
            Ok(SourceTokenKind::WhileKeyword) => self.parse_while_loop_statement(),
            Ok(SourceTokenKind::LetKeyword) => {
                match self.take_required_symbol(&SourceTokenKind::LetKeyword) {
                    Ok(consumed_symbol) => drop(consumed_symbol),
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                let local_mutability = match self.current_token_kind() {
                    Ok(SourceTokenKind::MutKeyword) => {
                        match self.take_required_symbol(&SourceTokenKind::MutKeyword) {
                            Ok(consumed_symbol) => drop(consumed_symbol),
                            Err(compilation_problem) => return Err(compilation_problem),
                        }
                        ParsedLocalMutability::Mutable
                    }
                    Ok(_) => ParsedLocalMutability::Immutable,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
                let (local_name, local_name_range) = match self.take_declaration_name() {
                    Ok(declaration_name) => declaration_name,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
                match self.take_required_symbol(&SourceTokenKind::Colon) {
                    Ok(consumed_symbol) => drop(consumed_symbol),
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                let value_type = match self.parse_value_type() {
                    Ok(value_type) => value_type,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
                match self.take_required_symbol(&SourceTokenKind::Equals) {
                    Ok(consumed_symbol) => drop(consumed_symbol),
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                let initial_value = match self.parse_expression() {
                    Ok(initial_value) => initial_value,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
                match self.take_required_symbol(&SourceTokenKind::Semicolon) {
                    Ok(consumed_symbol) => drop(consumed_symbol),
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                match local_mutability {
                    ParsedLocalMutability::Mutable => Ok(ParsedStatement::MutableLocal {
                        local_name,
                        local_name_range,
                        value_type,
                        initial_value,
                    }),
                    ParsedLocalMutability::Immutable => Ok(ParsedStatement::ImmutableLocal {
                        local_name,
                        local_name_range,
                        value_type,
                        initial_value,
                    }),
                }
            }
            Ok(_) => {
                let statement_problem = self.problem_at_current_token();
                let expression = match self.parse_expression() {
                    Ok(expression) => expression,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
                match expression {
                    ParsedExpression::NameReference {
                        referenced_name,
                        name_range,
                    } => match self.current_token_kind() {
                        Ok(SourceTokenKind::Equals) => {
                            self.parse_local_assignment((referenced_name, name_range))
                        }
                        Ok(_) => Err(statement_problem),
                        Err(compilation_problem) => Err(compilation_problem),
                    },
                    ParsedExpression::FunctionCall(function_call) => {
                        match self.take_required_symbol(&SourceTokenKind::Semicolon) {
                            Ok(consumed_symbol) => drop(consumed_symbol),
                            Err(compilation_problem) => return Err(compilation_problem),
                        }
                        Ok(ParsedStatement::CallFunctionAndIgnoreResult(function_call))
                    }
                    ParsedExpression::RobloxRemoteOperation(operation) => {
                        match self.take_required_symbol(&SourceTokenKind::Semicolon) {
                            Ok(consumed_symbol) => drop(consumed_symbol),
                            Err(compilation_problem) => return Err(compilation_problem),
                        }
                        Ok(ParsedStatement::RobloxRemoteOperation(operation))
                    }
                    ParsedExpression::NumberLiteral(_)
                    | ParsedExpression::StringLiteral(_)
                    | ParsedExpression::BooleanLiteral { .. }
                    | ParsedExpression::RobloxServiceAcquisition { .. }
                    | ParsedExpression::RobloxInstanceAcquisition { .. }
                    | ParsedExpression::RobloxInstanceWaitForChild { .. }
                    | ParsedExpression::ArrayLiteral(_)
                    | ParsedExpression::RecordLiteral(_)
                    | ParsedExpression::NumericOperation(_)
                    | ParsedExpression::ComparisonOperation(_)
                    | ParsedExpression::EqualityOperation(_)
                    | ParsedExpression::LogicalNegation(_)
                    | ParsedExpression::LogicalOperation(_)
                    | ParsedExpression::FunctionLiteral(_) => Err(statement_problem),
                    ParsedExpression::FieldRead(_) | ParsedExpression::ArrayRead(_) => {
                        match self.current_token_kind() {
                            Ok(SourceTokenKind::Equals) => {
                                self.parse_place_assignment((expression, statement_problem))
                            }
                            Ok(_) => Err(statement_problem),
                            Err(compilation_problem) => Err(compilation_problem),
                        }
                    }
                }
            }
            Err(compilation_problem) => Err(compilation_problem),
        }
    }

    fn parse_local_assignment(
        &mut self,
        local_name_at_range: (String, crate::SourceRange),
    ) -> Result<ParsedStatement, CompilationProblem> {
        let (local_name, local_name_range) = local_name_at_range;
        match self.take_required_symbol(&SourceTokenKind::Equals) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let assigned_value = match self.parse_expression() {
            Ok(assigned_value) => assigned_value,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_required_symbol(&SourceTokenKind::Semicolon) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        Ok(ParsedStatement::AssignLocal {
            local_name,
            local_name_range,
            assigned_value,
        })
    }

    fn parse_place_assignment(
        &mut self,
        target_and_problem: (ParsedExpression, CompilationProblem),
    ) -> Result<ParsedStatement, CompilationProblem> {
        let (target, statement_problem) = target_and_problem;
        let Some(FlattenedRecordFieldAssignmentTarget {
            root_binding_name,
            root_binding_range,
            mut reversed_steps,
        }) = Self::flatten_place_assignment_target(target)
        else {
            return Err(statement_problem);
        };
        reversed_steps.reverse();
        match self.take_required_symbol(&SourceTokenKind::Equals) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let assigned_value = match self.parse_expression() {
            Ok(assigned_value) => assigned_value,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_required_symbol(&SourceTokenKind::Semicolon) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        Ok(ParsedStatement::AssignPlace(
            crate::source_language::ParsedPlaceAssignment::from_parts((
                root_binding_name,
                root_binding_range,
                reversed_steps,
                assigned_value,
            )),
        ))
    }

    fn flatten_place_assignment_target(
        target: ParsedExpression,
    ) -> Option<FlattenedRecordFieldAssignmentTarget> {
        let mut base_expression;
        let mut reversed_steps;
        match target {
            ParsedExpression::FieldRead(field_read) => {
                let (base, field_name, field_range, _) = field_read.into_read();
                let base_range = base.source_range();
                base_expression = base;
                reversed_steps = vec![crate::source_language::ParsedPlaceStep::Field {
                    field_name,
                    field_range,
                    base_range,
                }];
            }
            ParsedExpression::ArrayRead(array_read) => {
                let (base, index_expression, _) = array_read.into_read();
                let base_range = base.source_range();
                base_expression = base;
                reversed_steps = vec![crate::source_language::ParsedPlaceStep::Index {
                    index_expression,
                    base_range,
                }];
            }
            _ => return None,
        }
        loop {
            match *base_expression {
                ParsedExpression::FieldRead(field_read) => {
                    let (next_base, field_name, field_range, _) = field_read.into_read();
                    let base_range = next_base.source_range();
                    reversed_steps.push(crate::source_language::ParsedPlaceStep::Field {
                        field_name,
                        field_range,
                        base_range,
                    });
                    base_expression = next_base;
                }
                ParsedExpression::ArrayRead(array_read) => {
                    let (next_base, index_expression, _) = array_read.into_read();
                    let base_range = next_base.source_range();
                    reversed_steps.push(crate::source_language::ParsedPlaceStep::Index {
                        index_expression,
                        base_range,
                    });
                    base_expression = next_base;
                }
                ParsedExpression::NameReference {
                    referenced_name,
                    name_range,
                } => {
                    return Some(FlattenedRecordFieldAssignmentTarget {
                        root_binding_name: referenced_name,
                        root_binding_range: name_range,
                        reversed_steps,
                    });
                }
                ParsedExpression::NumberLiteral(_)
                | ParsedExpression::StringLiteral(_)
                | ParsedExpression::BooleanLiteral { .. }
                | ParsedExpression::RobloxServiceAcquisition { .. }
                | ParsedExpression::RobloxInstanceAcquisition { .. }
                | ParsedExpression::RobloxInstanceWaitForChild { .. }
                | ParsedExpression::RecordLiteral(_)
                | ParsedExpression::ArrayLiteral(_)
                | ParsedExpression::NumericOperation(_)
                | ParsedExpression::ComparisonOperation(_)
                | ParsedExpression::EqualityOperation(_)
                | ParsedExpression::LogicalNegation(_)
                | ParsedExpression::LogicalOperation(_)
                | ParsedExpression::FunctionCall(_)
                | ParsedExpression::FunctionLiteral(_)
                | ParsedExpression::RobloxRemoteOperation(_) => return None,
            }
        }
    }

    fn parse_return_statement(&mut self) -> Result<ParsedStatement, CompilationProblem> {
        match self.take_required_symbol(&SourceTokenKind::ReturnKeyword) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let returned_value = match self.parse_expression() {
            Ok(returned_value) => returned_value,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_required_symbol(&SourceTokenKind::Semicolon) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        Ok(ParsedStatement::ReturnsValue(returned_value))
    }

    fn parse_loop_exit_statement(
        &mut self,
        exits_loop: bool,
    ) -> Result<ParsedStatement, CompilationProblem> {
        let keyword_kind = if exits_loop {
            SourceTokenKind::BreakKeyword
        } else {
            SourceTokenKind::ContinueKeyword
        };
        let keyword_range = match self.take_required_symbol(&keyword_kind) {
            Ok(consumed_symbol) => consumed_symbol.source_range(),
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_required_symbol(&SourceTokenKind::Semicolon) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        if exits_loop {
            Ok(ParsedStatement::BreaksLoop(keyword_range))
        } else {
            Ok(ParsedStatement::ContinuesLoop(keyword_range))
        }
    }

    fn parse_if_else_statement(&mut self) -> Result<ParsedStatement, CompilationProblem> {
        match self.take_required_symbol(&SourceTokenKind::IfKeyword) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let condition = match self.parse_condition_expression() {
            Ok(condition) => condition,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        let condition_range = condition.source_range();
        match self.take_required_symbol(&SourceTokenKind::LeftBrace) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let then_body = match self.parse_function_body() {
            Ok(then_body) => then_body,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_required_symbol(&SourceTokenKind::RightBrace) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        match self.take_required_symbol(&SourceTokenKind::ElseKeyword) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        match self.take_required_symbol(&SourceTokenKind::LeftBrace) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let else_body = match self.parse_function_body() {
            Ok(else_body) => else_body,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_required_symbol(&SourceTokenKind::RightBrace) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        Ok(ParsedStatement::IfElse(ParsedIfElse::from_parts((
            condition,
            then_body,
            else_body,
            condition_range,
        ))))
    }

    fn parse_while_loop_statement(&mut self) -> Result<ParsedStatement, CompilationProblem> {
        match self.take_required_symbol(&SourceTokenKind::WhileKeyword) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let condition = match self.parse_condition_expression() {
            Ok(condition) => condition,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        let condition_range = condition.source_range();
        match self.take_required_symbol(&SourceTokenKind::LeftBrace) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let body = match self.parse_function_body() {
            Ok(body) => body,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_required_symbol(&SourceTokenKind::RightBrace) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        Ok(ParsedStatement::WhileLoop(ParsedWhileLoop::from_parts((
            condition,
            body,
            condition_range,
        ))))
    }
}
