use std::cmp::Ordering;

use crate::{
    checked_program::{
        program_check_context::ProgramCheckContext, CheckedExpression, CheckedFunctionCall,
        CheckedValueType,
    },
    source_language::{ParsedExpression, ParsedFunctionCall},
    ArgumentCount, CompilationProblem, CompilationProblemReason, SourceRange,
};

/// Validates expressions and calls against the active program and function scopes.
pub(super) struct ExpressionChecker<'context, 'program> {
    check_context: &'context mut ProgramCheckContext<'program>,
}

/// Keeps reference, call, arity, and value-type checks at the expression boundary.
impl<'context, 'program> ExpressionChecker<'context, 'program> {
    /// Borrows the active semantic context for one expression-checking operation.
    pub(super) fn from_context(check_context: &'context mut ProgramCheckContext<'program>) -> Self {
        Self { check_context }
    }

    /// Produces a checked expression and its proven value type.
    pub(super) fn check_expression(
        &mut self,
        parsed_expression: &ParsedExpression,
    ) -> Result<(CheckedExpression, CheckedValueType), CompilationProblem> {
        match parsed_expression {
            ParsedExpression::NameReference {
                referenced_name,
                name_range,
            } => match self.resolve_local((referenced_name, *name_range)) {
                Ok(resolved_type) => Ok((
                    CheckedExpression::NameReference(referenced_name.to_owned()),
                    resolved_type,
                )),
                Err(compilation_problem) => Err(compilation_problem),
            },
            ParsedExpression::NumberLiteral { number_literal, .. } => Ok((
                CheckedExpression::NumberLiteral(number_literal.to_owned()),
                CheckedValueType::Number,
            )),
            ParsedExpression::Addition {
                left_operand,
                right_operand,
                operator_range,
                ..
            } => {
                let (checked_left, left_type) = match self.check_expression(left_operand) {
                    Ok(checked_operand) => checked_operand,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
                match Self::require_matching_type((
                    left_type,
                    CheckedValueType::Number,
                    *operator_range,
                )) {
                    Ok(()) => {}
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                let (checked_right, right_type) = match self.check_expression(right_operand) {
                    Ok(checked_operand) => checked_operand,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
                match Self::require_matching_type((
                    right_type,
                    CheckedValueType::Number,
                    *operator_range,
                )) {
                    Ok(()) => {}
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                Ok((
                    CheckedExpression::Addition {
                        left_operand: Box::new(checked_left),
                        right_operand: Box::new(checked_right),
                    },
                    CheckedValueType::Number,
                ))
            }
            ParsedExpression::FunctionCall(parsed_function_call) => {
                match self.check_function_call(parsed_function_call) {
                    Ok((checked_function_call, returned_value_type)) => Ok((
                        CheckedExpression::FunctionCall(checked_function_call),
                        returned_value_type,
                    )),
                    Err(compilation_problem) => Err(compilation_problem),
                }
            }
        }
    }

    /// Resolves and validates a call while preserving its returned value type.
    pub(super) fn check_function_call(
        &mut self,
        parsed_function_call: &ParsedFunctionCall,
    ) -> Result<(CheckedFunctionCall, CheckedValueType), CompilationProblem> {
        let function_name = parsed_function_call.function_name();
        let function_name_range = parsed_function_call.function_name_range();
        let function_arguments = parsed_function_call.function_arguments();
        let (expected_argument_types, returned_value_type) =
            match self.resolve_function_signature((function_name, function_name_range)) {
                Ok(function_signature) => function_signature,
                Err(compilation_problem) => return Err(compilation_problem),
            };
        match function_arguments.len().cmp(&expected_argument_types.len()) {
            Ordering::Equal => {}
            Ordering::Less | Ordering::Greater => {
                return Err(CompilationProblem::from_problem_at_range((
                    function_name_range,
                    CompilationProblemReason::WrongArgumentCount {
                        expected: ArgumentCount::from_number_of_arguments(
                            expected_argument_types.len(),
                        ),
                        actual: ArgumentCount::from_number_of_arguments(function_arguments.len()),
                    },
                )));
            }
        }

        let mut checked_arguments = Vec::new();
        for (parsed_argument, expected_type) in function_arguments
            .iter()
            .zip(expected_argument_types.iter())
        {
            let (checked_argument, actual_type) = match self.check_expression(parsed_argument) {
                Ok(checked_argument) => checked_argument,
                Err(compilation_problem) => return Err(compilation_problem),
            };
            match Self::require_matching_type((
                actual_type,
                *expected_type,
                parsed_argument.source_range(),
            )) {
                Ok(()) => {}
                Err(compilation_problem) => return Err(compilation_problem),
            }
            checked_arguments.push(checked_argument);
        }
        Ok((
            CheckedFunctionCall::from_checked_call((function_name.to_owned(), checked_arguments)),
            returned_value_type,
        ))
    }

    /// Resolves a callable name against builtins and source-ordered visible functions.
    pub(super) fn resolve_function_signature(
        &self,
        name_at_range: (&str, SourceRange),
    ) -> Result<(Vec<CheckedValueType>, CheckedValueType), CompilationProblem> {
        let (function_name, function_name_range) = name_at_range;
        match function_name.cmp("print") {
            Ordering::Equal => {
                return Ok((
                    vec![CheckedValueType::Number],
                    CheckedValueType::NoReturnedValues,
                ));
            }
            Ordering::Less | Ordering::Greater => {}
        }
        for (visible_name, parameter_types, returned_value_type) in
            self.check_context.visible_function_signatures()
        {
            match visible_name.as_str().cmp(function_name) {
                Ordering::Equal => {
                    return Ok((parameter_types.clone(), *returned_value_type));
                }
                Ordering::Less | Ordering::Greater => {}
            }
        }
        for parsed_function in self.check_context.parsed_program().parsed_functions() {
            match parsed_function.function_name().cmp(function_name) {
                Ordering::Equal => {
                    return Err(CompilationProblem::from_problem_at_range((
                        function_name_range,
                        CompilationProblemReason::NameUsedBeforeDeclaration,
                    )));
                }
                Ordering::Less | Ordering::Greater => {}
            }
        }
        Err(CompilationProblem::from_problem_at_range((
            function_name_range,
            CompilationProblemReason::UnknownName,
        )))
    }

    fn resolve_local(
        &self,
        name_at_range: (&str, SourceRange),
    ) -> Result<CheckedValueType, CompilationProblem> {
        let (referenced_name, name_range) = name_at_range;
        for (local_name, local_type) in self.check_context.local_bindings().iter().rev() {
            match local_name.as_str().cmp(referenced_name) {
                Ordering::Equal => return Ok(*local_type),
                Ordering::Less | Ordering::Greater => {}
            }
        }
        Err(CompilationProblem::from_problem_at_range((
            name_range,
            CompilationProblemReason::UnknownName,
        )))
    }

    /// Rejects a value whose proven type differs from its required type.
    pub(super) fn require_matching_type(
        type_requirement: (CheckedValueType, CheckedValueType, SourceRange),
    ) -> Result<(), CompilationProblem> {
        let (actual_type, expected_type, source_range) = type_requirement;
        match (actual_type, expected_type) {
            (CheckedValueType::Number, CheckedValueType::Number)
            | (CheckedValueType::NoReturnedValues, CheckedValueType::NoReturnedValues) => Ok(()),
            _ => Err(CompilationProblem::from_problem_at_range((
                source_range,
                CompilationProblemReason::TypesDoNotMatch,
            ))),
        }
    }
}
