use crate::{
    checked_program::{
        check_declaration_names::DeclarationNameChecker, check_expression::ExpressionChecker,
        program_check_context::ProgramCheckContext, CheckedFunction, CheckedFunctionReturn,
        CheckedParameter, CheckedStatement, CheckedValueType,
    },
    source_language::{ParsedFunction, ParsedFunctionReturn, ParsedStatement},
    CompilationProblem, CompilationProblemReason,
};

/// Validates one function's parameters, locals, statements, and return contract.
pub(super) struct FunctionChecker<'context, 'program> {
    check_context: &'context mut ProgramCheckContext<'program>,
}

/// Keeps function-scope mutation separate from program orchestration and expression rules.
impl<'context, 'program> FunctionChecker<'context, 'program> {
    /// Borrows the active program context for one complete function check.
    pub(super) fn from_context(check_context: &'context mut ProgramCheckContext<'program>) -> Self {
        Self { check_context }
    }

    /// Produces a checked function only after its whole local scope and return contract validate.
    pub(super) fn check_function(
        &mut self,
        parsed_function: &ParsedFunction,
    ) -> Result<CheckedFunction, CompilationProblem> {
        self.check_context.begin_function(parsed_function);
        let mut checked_parameters = Vec::new();
        for parsed_parameter in parsed_function.function_parameters() {
            match DeclarationNameChecker::check_local_name((
                self.check_context,
                parsed_parameter.parameter_name(),
                parsed_parameter.parameter_name_range(),
            )) {
                Ok(()) => {}
                Err(compilation_problem) => return Err(compilation_problem),
            }
            let checked_value_type =
                ProgramCheckContext::to_checked_value_type(parsed_parameter.value_type());
            self.check_context.add_local_binding((
                parsed_parameter.parameter_name().to_owned(),
                checked_value_type,
            ));
            checked_parameters.push(CheckedParameter::from_checked_declaration((
                parsed_parameter.parameter_name().to_owned(),
                checked_value_type,
            )));
        }

        let mut checked_statements = Vec::new();
        for parsed_statement in parsed_function.function_statements() {
            match self.check_statement(parsed_statement) {
                Ok(checked_statement) => checked_statements.push(checked_statement),
                Err(compilation_problem) => return Err(compilation_problem),
            }
        }
        let checked_function_return = match parsed_function.function_return() {
            ParsedFunctionReturn::NoReturn => {
                match self.check_context.expected_returned_value_type() {
                    CheckedValueType::Number => {
                        return Err(CompilationProblem::from_problem_at_range((
                            parsed_function.function_name_range(),
                            CompilationProblemReason::MissingReturn,
                        )));
                    }
                    CheckedValueType::NoReturnedValues => CheckedFunctionReturn::NoReturn,
                }
            }
            ParsedFunctionReturn::ReturnsValue(returned_value) => {
                let (checked_expression, actual_type) = {
                    let mut expression_checker =
                        ExpressionChecker::from_context(self.check_context);
                    match expression_checker.check_expression(returned_value) {
                        Ok(checked_expression) => checked_expression,
                        Err(compilation_problem) => return Err(compilation_problem),
                    }
                };
                match ExpressionChecker::require_matching_type((
                    actual_type,
                    self.check_context.expected_returned_value_type(),
                    returned_value.source_range(),
                )) {
                    Ok(()) => {}
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                CheckedFunctionReturn::ReturnsValue(checked_expression)
            }
        };
        Ok(CheckedFunction::from_checked_declaration((
            parsed_function.function_name().to_owned(),
            checked_parameters,
            self.check_context.expected_returned_value_type(),
            checked_statements,
            checked_function_return,
        )))
    }

    fn check_statement(
        &mut self,
        parsed_statement: &ParsedStatement,
    ) -> Result<CheckedStatement, CompilationProblem> {
        match parsed_statement {
            ParsedStatement::ImmutableLocal {
                local_name,
                local_name_range,
                value_type,
                initial_value,
            } => {
                match DeclarationNameChecker::check_local_name((
                    self.check_context,
                    local_name,
                    *local_name_range,
                )) {
                    Ok(()) => {}
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                let checked_value_type = ProgramCheckContext::to_checked_value_type(*value_type);
                let (checked_initial_value, actual_type) = {
                    let mut expression_checker =
                        ExpressionChecker::from_context(self.check_context);
                    match expression_checker.check_expression(initial_value) {
                        Ok(checked_expression) => checked_expression,
                        Err(compilation_problem) => return Err(compilation_problem),
                    }
                };
                match ExpressionChecker::require_matching_type((
                    actual_type,
                    checked_value_type,
                    initial_value.source_range(),
                )) {
                    Ok(()) => {}
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                self.check_context
                    .add_local_binding((local_name.to_owned(), checked_value_type));
                Ok(CheckedStatement::ImmutableLocal {
                    local_name: local_name.to_owned(),
                    value_type: checked_value_type,
                    initial_value: checked_initial_value,
                })
            }
            ParsedStatement::CallFunctionAndIgnoreResult(parsed_function_call) => {
                let mut expression_checker = ExpressionChecker::from_context(self.check_context);
                match expression_checker.check_function_call(parsed_function_call) {
                    Ok((checked_function_call, _)) => Ok(
                        CheckedStatement::CallFunctionAndIgnoreResult(checked_function_call),
                    ),
                    Err(compilation_problem) => Err(compilation_problem),
                }
            }
        }
    }
}
