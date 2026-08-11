use std::cmp::Ordering;

use crate::{
    checked_program::{
        program_check_context::ProgramCheckContext, CheckedArrayLiteral, CheckedArrayRead,
        CheckedBooleanLiteral, CheckedExpression, CheckedFieldRead, CheckedFunctionCall,
        CheckedInstanceConstruction, CheckedInstanceLookup, CheckedNumericOperation,
        CheckedNumericOperator, CheckedRecordFieldInitializer, CheckedRecordLiteral,
        CheckedValueType,
    },
    source_language::{
        ParsedArrayLiteral, ParsedArrayRead, ParsedExpression, ParsedFieldRead, ParsedFunctionCall,
        ParsedNumericOperation, ParsedNumericOperator, ParsedRecordLiteral, SourceBooleanLiteral,
    },
    ArgumentCount, CompilationProblem, CompilationProblemReason, SourceRange,
};

/// Validates expressions and calls against the active program and function scopes.
pub(super) struct ExpressionChecker<'context, 'program> {
    check_context: &'context mut ProgramCheckContext<'program>,
}

/// Keeps reference, call, arity, and value-type checks at the expression boundary.
impl<'context, 'program> ExpressionChecker<'context, 'program> {
    /// Borrows the active semantic context for one expression-checking operation.
    pub(super) const fn from_context(
        check_context: &'context mut ProgramCheckContext<'program>,
    ) -> Self {
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
            ParsedExpression::NumberLiteral(parsed_literal) => Ok((
                CheckedExpression::NumberLiteral(parsed_literal.literal_spelling().to_owned()),
                CheckedValueType::Number,
            )),
            ParsedExpression::StringLiteral(parsed_literal) => Ok((
                CheckedExpression::StringLiteral(parsed_literal.literal_spelling().to_owned()),
                CheckedValueType::String,
            )),
            ParsedExpression::BooleanLiteral {
                boolean_literal, ..
            } => {
                let checked_boolean_literal = match boolean_literal {
                    SourceBooleanLiteral::True => CheckedBooleanLiteral::True,
                    SourceBooleanLiteral::False => CheckedBooleanLiteral::False,
                };
                Ok((
                    CheckedExpression::BooleanLiteral(checked_boolean_literal),
                    CheckedValueType::Boolean,
                ))
            }
            ParsedExpression::RobloxServiceAcquisition {
                service_type_name,
                service_type_range,
                ..
            } => {
                let roblox_service = self
                    .check_context
                    .acquire_roblox_service((service_type_name, *service_type_range))?;
                Ok((
                    CheckedExpression::RobloxServiceAcquisition(roblox_service),
                    CheckedValueType::RobloxService(roblox_service),
                ))
            }
            ParsedExpression::RobloxInstanceAcquisition {
                instance_type_name,
                instance_type_range,
                parent_expression,
                ..
            } => {
                let roblox_instance = ProgramCheckContext::acquire_roblox_instance((
                    instance_type_name,
                    *instance_type_range,
                ))?;
                let checked_parent = match parent_expression {
                    Some(parent_expression) => {
                        let (checked_parent, parent_type) =
                            self.check_expression(parent_expression)?;
                        if !matches!(
                            parent_type,
                            CheckedValueType::RobloxInstance(_)
                                | CheckedValueType::RobloxService(_)
                        ) {
                            return Err(CompilationProblem::from_problem_at_range((
                                parent_expression.source_range(),
                                CompilationProblemReason::TypesDoNotMatch,
                            )));
                        }
                        Some(Box::new(checked_parent))
                    }
                    None => None,
                };
                Ok((
                    CheckedExpression::RobloxInstanceAcquisition(
                        CheckedInstanceConstruction::from_parts((roblox_instance, checked_parent)),
                    ),
                    CheckedValueType::RobloxInstance(roblox_instance),
                ))
            }
            ParsedExpression::RobloxInstanceWaitForChild {
                instance_type_name,
                instance_type_range,
                parent_expression,
                child_name_expression,
                ..
            } => {
                let roblox_instance = ProgramCheckContext::acquire_roblox_instance((
                    instance_type_name,
                    *instance_type_range,
                ))?;
                let (checked_parent, parent_type) = self.check_expression(parent_expression)?;
                if !matches!(
                    parent_type,
                    CheckedValueType::RobloxInstance(_) | CheckedValueType::RobloxService(_)
                ) {
                    return Err(CompilationProblem::from_problem_at_range((
                        parent_expression.source_range(),
                        CompilationProblemReason::FieldAccessRequiresRecord,
                    )));
                }
                let (checked_child_name, child_name_type) =
                    self.check_expression(child_name_expression)?;
                Self::require_matching_type((
                    child_name_type,
                    CheckedValueType::String,
                    child_name_expression.source_range(),
                ))?;
                Ok((
                    CheckedExpression::RobloxInstanceWaitForChild(
                        CheckedInstanceLookup::from_parts((
                            roblox_instance,
                            Box::new(checked_parent),
                            Box::new(checked_child_name),
                        )),
                    ),
                    CheckedValueType::RobloxInstance(roblox_instance),
                ))
            }
            ParsedExpression::ArrayLiteral(array_literal) => {
                self.check_array_literal(array_literal)
            }
            ParsedExpression::RecordLiteral(record_literal) => {
                self.check_record_literal(record_literal)
            }
            ParsedExpression::FieldRead(field_read) => self.check_field_read(field_read),
            ParsedExpression::ArrayRead(array_read) => self.check_array_read(array_read),
            ParsedExpression::NumericOperation(operation) => {
                self.check_numeric_operation(operation)
            }
            ParsedExpression::ComparisonOperation(operation) => {
                self.check_comparison_operation(operation)
            }
            ParsedExpression::EqualityOperation(operation) => {
                self.check_equality_operation(operation)
            }
            ParsedExpression::LogicalNegation(negation) => self.check_logical_negation(negation),
            ParsedExpression::LogicalOperation(operation) => {
                self.check_logical_operation(operation)
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

    fn check_array_literal(
        &mut self,
        array_literal: &ParsedArrayLiteral,
    ) -> Result<(CheckedExpression, CheckedValueType), CompilationProblem> {
        let Some((first_element, remaining_elements)) =
            array_literal.element_expressions().split_first()
        else {
            return Err(CompilationProblem::from_problem_at_range((
                array_literal.literal_range(),
                CompilationProblemReason::SourceDoesNotFollowLanguageRules,
            )));
        };
        let (checked_first, element_type) = self.check_expression(first_element)?;
        let mut checked_elements = vec![checked_first];
        for element in remaining_elements {
            let (checked_element, actual_type) = self.check_expression(element)?;
            Self::require_matching_type((
                actual_type,
                element_type.clone(),
                element.source_range(),
            ))?;
            checked_elements.push(checked_element);
        }
        Ok((
            CheckedExpression::ArrayLiteral(CheckedArrayLiteral::from_elements(checked_elements)),
            CheckedValueType::Array(Box::new(element_type)),
        ))
    }

    fn check_array_read(
        &mut self,
        array_read: &ParsedArrayRead,
    ) -> Result<(CheckedExpression, CheckedValueType), CompilationProblem> {
        let (checked_base, base_type) = self.check_expression(array_read.base_expression())?;
        let CheckedValueType::Array(element_type) = base_type else {
            return Err(CompilationProblem::from_problem_at_range((
                array_read.base_expression().source_range(),
                CompilationProblemReason::TypesDoNotMatch,
            )));
        };
        let (checked_index, index_type) = self.check_expression(array_read.index_expression())?;
        Self::require_matching_type((
            index_type,
            CheckedValueType::Number,
            array_read.index_expression().source_range(),
        ))?;
        Ok((
            CheckedExpression::ArrayRead(CheckedArrayRead::from_read((
                Box::new(checked_base),
                Box::new(checked_index),
            ))),
            *element_type,
        ))
    }

    fn check_record_literal(
        &mut self,
        record_literal: &ParsedRecordLiteral,
    ) -> Result<(CheckedExpression, CheckedValueType), CompilationProblem> {
        let declared_fields = self
            .check_context
            .checked_record_declaration((
                record_literal.record_name(),
                record_literal.record_name_range(),
            ))?
            .record_fields()
            .iter()
            .map(|field| (field.field_name().to_owned(), field.value_type().clone()))
            .collect::<Vec<_>>();
        let mut checked_initializers = Vec::new();
        for field_initializer in record_literal.field_initializers() {
            if checked_initializers.iter().any(
                |checked_initializer: &CheckedRecordFieldInitializer| {
                    checked_initializer.field_name() == field_initializer.field_name()
                },
            ) {
                return Err(CompilationProblem::from_problem_at_range((
                    field_initializer.field_name_range(),
                    CompilationProblemReason::DuplicateRecordField,
                )));
            }
            let Some((_, expected_value_type)) = declared_fields
                .iter()
                .find(|(field_name, _)| field_name == field_initializer.field_name())
            else {
                return Err(CompilationProblem::from_problem_at_range((
                    field_initializer.field_name_range(),
                    CompilationProblemReason::UnknownRecordField,
                )));
            };
            let (checked_value, actual_type) =
                self.check_expression(field_initializer.initialized_value())?;
            match Self::require_matching_type((
                actual_type,
                expected_value_type.clone(),
                field_initializer.initialized_value().source_range(),
            )) {
                Ok(()) => {}
                Err(_) => {
                    return Err(CompilationProblem::from_problem_at_range((
                        field_initializer.initialized_value().source_range(),
                        CompilationProblemReason::RecordFieldInitializerTypeMismatch,
                    )));
                }
            }
            checked_initializers.push(CheckedRecordFieldInitializer::from_initializer((
                field_initializer.field_name().to_owned(),
                checked_value,
            )));
        }
        for (record_field_name, _) in &declared_fields {
            if !checked_initializers
                .iter()
                .any(|checked_initializer| checked_initializer.field_name() == record_field_name)
            {
                return Err(CompilationProblem::from_problem_at_range((
                    record_literal.record_name_range(),
                    CompilationProblemReason::MissingRecordField,
                )));
            }
        }
        Ok((
            CheckedExpression::RecordLiteral(CheckedRecordLiteral::from_initializers(
                checked_initializers,
            )),
            CheckedValueType::NamedRecord(record_literal.record_name().to_owned()),
        ))
    }

    fn check_field_read(
        &mut self,
        field_read: &ParsedFieldRead,
    ) -> Result<(CheckedExpression, CheckedValueType), CompilationProblem> {
        let (checked_base, base_type) = self.check_expression(field_read.base_expression())?;
        let (checked_value_type, unknown_field_reason) = match base_type {
            CheckedValueType::NamedRecord(record_name) => (
                self.check_context
                    .checked_record_declaration((&record_name, field_read.field_name_range()))?
                    .record_fields()
                    .iter()
                    .find(|record_field| record_field.field_name() == field_read.field_name())
                    .map(|record_field| record_field.value_type().clone()),
                CompilationProblemReason::UnknownRecordAccessField,
            ),
            CheckedValueType::RobloxInstance(roblox_instance) => (
                roblox_instance.property_type(field_read.field_name()),
                CompilationProblemReason::UnknownRobloxInstanceMember,
            ),
            _ => {
                return Err(CompilationProblem::from_problem_at_range((
                    field_read.base_expression().source_range(),
                    CompilationProblemReason::FieldAccessRequiresRecord,
                )));
            }
        };
        let Some(checked_value_type) = checked_value_type else {
            return Err(CompilationProblem::from_problem_at_range((
                field_read.field_name_range(),
                unknown_field_reason,
            )));
        };
        Ok((
            CheckedExpression::FieldRead(CheckedFieldRead::from_read((
                Box::new(checked_base),
                field_read.field_name().to_owned(),
            ))),
            checked_value_type,
        ))
    }

    /// Checks both operands and preserves the stage-specific numeric operator.
    fn check_numeric_operation(
        &mut self,
        operation: &ParsedNumericOperation,
    ) -> Result<(CheckedExpression, CheckedValueType), CompilationProblem> {
        let (checked_left, left_type) = match self.check_expression(operation.left_operand()) {
            Ok(checked_operand) => checked_operand,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match Self::require_matching_type((
            left_type,
            CheckedValueType::Number,
            operation.operator_range(),
        )) {
            Ok(()) => {}
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let (checked_right, right_type) = match self.check_expression(operation.right_operand()) {
            Ok(checked_operand) => checked_operand,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match Self::require_matching_type((
            right_type,
            CheckedValueType::Number,
            operation.operator_range(),
        )) {
            Ok(()) => {}
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let checked_operator = match operation.operator() {
            ParsedNumericOperator::Addition => CheckedNumericOperator::Addition,
            ParsedNumericOperator::Subtraction => CheckedNumericOperator::Subtraction,
            ParsedNumericOperator::Multiplication => CheckedNumericOperator::Multiplication,
            ParsedNumericOperator::Division => CheckedNumericOperator::Division,
        };
        Ok((
            CheckedExpression::NumericOperation(CheckedNumericOperation::from_parts((
                Box::new(checked_left),
                Box::new(checked_right),
                checked_operator,
            ))),
            CheckedValueType::Number,
        ))
    }

    /// Resolves and validates a call while preserving its returned value type.
    pub(super) fn check_function_call(
        &mut self,
        parsed_function_call: &ParsedFunctionCall,
    ) -> Result<(CheckedFunctionCall, CheckedValueType), CompilationProblem> {
        let function_name = parsed_function_call.function_name();
        let function_name_range = parsed_function_call.function_name_range();
        let function_arguments = parsed_function_call.function_arguments();
        match function_name.cmp("print") {
            Ordering::Equal => return self.check_print_call(parsed_function_call),
            Ordering::Less | Ordering::Greater => {}
        }
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
                expected_type.clone(),
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

    fn check_print_call(
        &mut self,
        parsed_function_call: &ParsedFunctionCall,
    ) -> Result<(CheckedFunctionCall, CheckedValueType), CompilationProblem> {
        let function_arguments = parsed_function_call.function_arguments();
        match function_arguments.len().cmp(&1) {
            Ordering::Equal => {}
            Ordering::Less | Ordering::Greater => {
                return Err(CompilationProblem::from_problem_at_range((
                    parsed_function_call.function_name_range(),
                    CompilationProblemReason::WrongArgumentCount {
                        expected: ArgumentCount::from_number_of_arguments(1),
                        actual: ArgumentCount::from_number_of_arguments(function_arguments.len()),
                    },
                )));
            }
        }
        let Some(parsed_argument) = function_arguments.first() else {
            return Err(CompilationProblem::from_problem_at_range((
                parsed_function_call.function_name_range(),
                CompilationProblemReason::WrongArgumentCount {
                    expected: ArgumentCount::from_number_of_arguments(1),
                    actual: ArgumentCount::from_number_of_arguments(0),
                },
            )));
        };
        let (checked_argument, argument_type) = match self.check_expression(parsed_argument) {
            Ok(checked_argument) => checked_argument,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match argument_type {
            CheckedValueType::Number
            | CheckedValueType::String
            | CheckedValueType::Boolean
            | CheckedValueType::NamedRecord(_)
            | CheckedValueType::RobloxService(_)
            | CheckedValueType::RobloxInstance(_)
            | CheckedValueType::Array(_) => {}
            CheckedValueType::NoReturnedValues => {
                return Err(CompilationProblem::from_problem_at_range((
                    parsed_argument.source_range(),
                    CompilationProblemReason::TypesDoNotMatch,
                )));
            }
        }
        Ok((
            CheckedFunctionCall::from_checked_call((
                parsed_function_call.function_name().to_owned(),
                vec![checked_argument],
            )),
            CheckedValueType::NoReturnedValues,
        ))
    }

    /// Resolves a callable name against builtins and source-ordered visible functions.
    pub(super) fn resolve_function_signature(
        &self,
        name_at_range: (&str, SourceRange),
    ) -> Result<(Vec<CheckedValueType>, CheckedValueType), CompilationProblem> {
        let (function_name, function_name_range) = name_at_range;
        for (visible_name, parameter_types, returned_value_type) in
            self.check_context.visible_function_signatures()
        {
            match visible_name.as_str().cmp(function_name) {
                Ordering::Equal => {
                    return Ok((parameter_types.clone(), returned_value_type.clone()));
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
        for local_binding in self.check_context.local_bindings().iter().rev() {
            match local_binding.local_name().cmp(referenced_name) {
                Ordering::Equal => return Ok(local_binding.value_type()),
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
            | (CheckedValueType::String, CheckedValueType::String)
            | (CheckedValueType::Boolean, CheckedValueType::Boolean)
            | (CheckedValueType::NoReturnedValues, CheckedValueType::NoReturnedValues) => Ok(()),
            (
                CheckedValueType::NamedRecord(actual_name),
                CheckedValueType::NamedRecord(expected_name),
            ) if actual_name == expected_name => Ok(()),
            (
                CheckedValueType::RobloxService(actual_service),
                CheckedValueType::RobloxService(expected_service),
            ) if actual_service == expected_service => Ok(()),
            (
                CheckedValueType::RobloxInstance(actual_instance),
                CheckedValueType::RobloxInstance(expected_instance),
            ) if actual_instance == expected_instance => Ok(()),
            (
                CheckedValueType::Array(actual_element_type),
                CheckedValueType::Array(expected_element_type),
            ) => Self::require_matching_type((
                *actual_element_type,
                *expected_element_type,
                source_range,
            )),
            _ => Err(CompilationProblem::from_problem_at_range((
                source_range,
                CompilationProblemReason::TypesDoNotMatch,
            ))),
        }
    }
}
