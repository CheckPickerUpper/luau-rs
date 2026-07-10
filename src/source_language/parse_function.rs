use crate::{
    source_language::{
        parse_source_program::SourceProgramParser, ParsedExpression, ParsedFunction,
        ParsedFunctionReturn, ParsedParameter, ParsedStatement, ParsedValueType, SourceTokenKind,
    },
    CompilationProblem,
};

/// Parses function declarations, bodies, parameters, and non-return statements.
impl SourceProgramParser {
    /// Parses one complete function declaration at the current token position.
    pub(super) fn parse_function(&mut self) -> Result<ParsedFunction, CompilationProblem> {
        match self.take_required_symbol(SourceTokenKind::FunctionKeyword) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let (function_name, function_name_range) = match self.take_declaration_name() {
            Ok(declaration_name) => declaration_name,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_required_symbol(SourceTokenKind::LeftParenthesis) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let function_parameters = match self.parse_function_parameters() {
            Ok(function_parameters) => function_parameters,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_required_symbol(SourceTokenKind::RightParenthesis) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let returned_value_type = match self.current_token_kind() {
            Ok(SourceTokenKind::Arrow) => {
                match self.take_required_symbol(SourceTokenKind::Arrow) {
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
        match self.take_required_symbol(SourceTokenKind::LeftBrace) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let (function_statements, function_return) = match self.parse_function_body() {
            Ok(function_body) => function_body,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match self.take_required_symbol(SourceTokenKind::RightBrace) {
            Ok(consumed_symbol) => drop(consumed_symbol),
            Err(compilation_problem) => return Err(compilation_problem),
        }
        Ok(ParsedFunction::from_declaration((
            function_name,
            function_name_range,
            function_parameters,
            returned_value_type,
            function_statements,
            function_return,
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
                    match self.take_required_symbol(SourceTokenKind::Colon) {
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
        Ok(function_parameters)
    }

    fn parse_value_type(&mut self) -> Result<ParsedValueType, CompilationProblem> {
        let (type_name, type_range) = match self.take_identifier_name() {
            Ok(type_name_at_range) => type_name_at_range,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match type_name.as_str() {
            "number" => Ok(ParsedValueType::Number),
            "string" => Ok(ParsedValueType::String),
            "boolean" => Ok(ParsedValueType::Boolean),
            _ => Err(CompilationProblem::from_problem_at_range((
                type_range,
                crate::CompilationProblemReason::SourceDoesNotFollowLanguageRules,
            ))),
        }
    }

    fn parse_function_body(
        &mut self,
    ) -> Result<(Vec<ParsedStatement>, ParsedFunctionReturn), CompilationProblem> {
        let mut function_statements = Vec::new();
        loop {
            match self.current_token_kind() {
                Ok(SourceTokenKind::RightBrace) => {
                    return Ok((function_statements, ParsedFunctionReturn::NoReturn));
                }
                Ok(SourceTokenKind::ReturnKeyword) => {
                    match self.take_required_symbol(SourceTokenKind::ReturnKeyword) {
                        Ok(consumed_symbol) => drop(consumed_symbol),
                        Err(compilation_problem) => return Err(compilation_problem),
                    }
                    let returned_value = match self.parse_expression() {
                        Ok(returned_value) => returned_value,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
                    match self.take_required_symbol(SourceTokenKind::Semicolon) {
                        Ok(consumed_symbol) => drop(consumed_symbol),
                        Err(compilation_problem) => return Err(compilation_problem),
                    }
                    match self.current_token_kind() {
                        Ok(SourceTokenKind::RightBrace) => {
                            return Ok((
                                function_statements,
                                ParsedFunctionReturn::ReturnsValue(returned_value),
                            ));
                        }
                        Ok(_) => return Err(self.problem_at_current_token()),
                        Err(compilation_problem) => return Err(compilation_problem),
                    }
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
            Ok(SourceTokenKind::LetKeyword) => {
                match self.take_required_symbol(SourceTokenKind::LetKeyword) {
                    Ok(consumed_symbol) => drop(consumed_symbol),
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                let (local_name, local_name_range) = match self.take_declaration_name() {
                    Ok(declaration_name) => declaration_name,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
                match self.take_required_symbol(SourceTokenKind::Colon) {
                    Ok(consumed_symbol) => drop(consumed_symbol),
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                let value_type = match self.parse_value_type() {
                    Ok(value_type) => value_type,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
                match self.take_required_symbol(SourceTokenKind::Equals) {
                    Ok(consumed_symbol) => drop(consumed_symbol),
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                let initial_value = match self.parse_expression() {
                    Ok(initial_value) => initial_value,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
                match self.take_required_symbol(SourceTokenKind::Semicolon) {
                    Ok(consumed_symbol) => drop(consumed_symbol),
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                Ok(ParsedStatement::ImmutableLocal {
                    local_name,
                    local_name_range,
                    value_type,
                    initial_value,
                })
            }
            Ok(_) => {
                let statement_problem = self.problem_at_current_token();
                let expression = match self.parse_expression() {
                    Ok(expression) => expression,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
                match self.take_required_symbol(SourceTokenKind::Semicolon) {
                    Ok(consumed_symbol) => drop(consumed_symbol),
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                match expression {
                    ParsedExpression::FunctionCall(function_call) => {
                        Ok(ParsedStatement::CallFunctionAndIgnoreResult(function_call))
                    }
                    _ => Err(statement_problem),
                }
            }
            Err(compilation_problem) => Err(compilation_problem),
        }
    }
}
