use crate::{
    checked_program::{
        check_declaration_names::DeclarationNameChecker, check_expression::ExpressionChecker,
        program_check_context::ProgramCheckContext, CheckedFunction, CheckedFunctionBody,
        CheckedIfElse, CheckedParameter, CheckedStatement, CheckedValueType,
    },
    source_language::{ParsedFunction, ParsedFunctionBody, ParsedIfElse, ParsedStatement},
    CompilationProblem, CompilationProblemReason,
};

use super::FunctionBodyCompletion;

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

        let (checked_function_body, function_completion) =
            match self.check_function_body(parsed_function.function_body()) {
                Ok(checked_body) => checked_body,
                Err(compilation_problem) => return Err(compilation_problem),
            };
        match (
            self.check_context.expected_returned_value_type(),
            function_completion,
        ) {
            (
                CheckedValueType::Number | CheckedValueType::String | CheckedValueType::Boolean,
                FunctionBodyCompletion::ReachesEnd,
            ) => {
                return Err(CompilationProblem::from_problem_at_range((
                    parsed_function.function_name_range(),
                    CompilationProblemReason::MissingReturn,
                )));
            }
            (
                CheckedValueType::Number
                | CheckedValueType::String
                | CheckedValueType::Boolean
                | CheckedValueType::NoReturnedValues,
                FunctionBodyCompletion::AlwaysReturns,
            )
            | (CheckedValueType::NoReturnedValues, FunctionBodyCompletion::ReachesEnd) => {}
        }
        Ok(CheckedFunction::from_checked_declaration((
            parsed_function.function_name().to_owned(),
            checked_parameters,
            self.check_context.expected_returned_value_type(),
            checked_function_body,
        )))
    }

    fn check_function_body(
        &mut self,
        parsed_function_body: &ParsedFunctionBody,
    ) -> Result<(CheckedFunctionBody, FunctionBodyCompletion), CompilationProblem> {
        let mut checked_statements = Vec::new();
        let mut function_completion = FunctionBodyCompletion::ReachesEnd;
        for parsed_statement in parsed_function_body.body_statements() {
            match function_completion {
                FunctionBodyCompletion::AlwaysReturns => {
                    return Err(CompilationProblem::from_problem_at_range((
                        parsed_statement.source_range(),
                        CompilationProblemReason::SourceDoesNotFollowLanguageRules,
                    )));
                }
                FunctionBodyCompletion::ReachesEnd => {}
            }
            let (checked_statement, statement_completion) =
                match self.check_statement(parsed_statement) {
                    Ok(checked_statement) => checked_statement,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
            checked_statements.push(checked_statement);
            function_completion = statement_completion;
        }
        Ok((
            CheckedFunctionBody::from_statements(checked_statements),
            function_completion,
        ))
    }

    fn check_statement(
        &mut self,
        parsed_statement: &ParsedStatement,
    ) -> Result<(CheckedStatement, FunctionBodyCompletion), CompilationProblem> {
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
                Ok((
                    CheckedStatement::ImmutableLocal {
                        local_name: local_name.to_owned(),
                        value_type: checked_value_type,
                        initial_value: checked_initial_value,
                    },
                    FunctionBodyCompletion::ReachesEnd,
                ))
            }
            ParsedStatement::CallFunctionAndIgnoreResult(parsed_function_call) => {
                let mut expression_checker = ExpressionChecker::from_context(self.check_context);
                match expression_checker.check_function_call(parsed_function_call) {
                    Ok((checked_function_call, _)) => Ok((
                        CheckedStatement::CallFunctionAndIgnoreResult(checked_function_call),
                        FunctionBodyCompletion::ReachesEnd,
                    )),
                    Err(compilation_problem) => Err(compilation_problem),
                }
            }
            ParsedStatement::ReturnsValue(returned_value) => {
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
                    Ok(()) => Ok((
                        CheckedStatement::ReturnsValue(checked_expression),
                        FunctionBodyCompletion::AlwaysReturns,
                    )),
                    Err(compilation_problem) => Err(compilation_problem),
                }
            }
            ParsedStatement::IfElse(parsed_if_else) => self.check_if_else(parsed_if_else),
        }
    }

    fn check_if_else(
        &mut self,
        parsed_if_else: &ParsedIfElse,
    ) -> Result<(CheckedStatement, FunctionBodyCompletion), CompilationProblem> {
        let (checked_condition, condition_type) = {
            let mut expression_checker = ExpressionChecker::from_context(self.check_context);
            match expression_checker.check_expression(parsed_if_else.condition()) {
                Ok(checked_condition) => checked_condition,
                Err(compilation_problem) => return Err(compilation_problem),
            }
        };
        match ExpressionChecker::require_matching_type((
            condition_type,
            CheckedValueType::Boolean,
            parsed_if_else.condition_range(),
        )) {
            Ok(()) => {}
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let local_scope_boundary = self.check_context.local_scope_boundary();
        let (checked_then_body, then_completion) =
            match self.check_function_body(parsed_if_else.then_body()) {
                Ok(checked_body) => checked_body,
                Err(compilation_problem) => return Err(compilation_problem),
            };
        self.check_context.end_local_scope(local_scope_boundary);
        let (checked_else_body, else_completion) =
            match self.check_function_body(parsed_if_else.else_body()) {
                Ok(checked_body) => checked_body,
                Err(compilation_problem) => return Err(compilation_problem),
            };
        self.check_context.end_local_scope(local_scope_boundary);
        let if_else_completion = match (then_completion, else_completion) {
            (FunctionBodyCompletion::AlwaysReturns, FunctionBodyCompletion::AlwaysReturns) => {
                FunctionBodyCompletion::AlwaysReturns
            }
            (FunctionBodyCompletion::ReachesEnd, FunctionBodyCompletion::AlwaysReturns)
            | (FunctionBodyCompletion::AlwaysReturns, FunctionBodyCompletion::ReachesEnd)
            | (FunctionBodyCompletion::ReachesEnd, FunctionBodyCompletion::ReachesEnd) => {
                FunctionBodyCompletion::ReachesEnd
            }
        };
        Ok((
            CheckedStatement::IfElse(CheckedIfElse::from_parts((
                checked_condition,
                checked_then_body,
                checked_else_body,
            ))),
            if_else_completion,
        ))
    }
}
