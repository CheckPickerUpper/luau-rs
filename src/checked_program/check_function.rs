use crate::{
    checked_program::{
        check_declaration_names::DeclarationNameChecker, check_expression::ExpressionChecker,
        program_check_context::ProgramCheckContext, CheckedFunction, CheckedFunctionBody,
        CheckedFunctionLiteral, CheckedIfElse, CheckedLocalBinding, CheckedParameter,
        CheckedPlaceAssignment, CheckedPlaceStep, CheckedStatement, CheckedValueType,
        CheckedWhileLoop, LocalAssignmentContract,
    },
    source_language::{
        ParsedFunction, ParsedFunctionBody, ParsedFunctionLiteral, ParsedIfElse,
        ParsedPlaceAssignment, ParsedPlaceStep, ParsedStatement, ParsedWhileLoop,
    },
    CompilationProblem, CompilationProblemReason,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum BodyControlFlow {
    ReachesEnd,
    Returns,
    BreaksLoop,
    ContinuesLoop,
}

struct BodyControlFlows {
    outcomes: Vec<BodyControlFlow>,
}

impl BodyControlFlows {
    fn reaches_end() -> Self {
        Self::from_outcome(BodyControlFlow::ReachesEnd)
    }

    fn returns() -> Self {
        Self::from_outcome(BodyControlFlow::Returns)
    }

    fn breaks_loop() -> Self {
        Self::from_outcome(BodyControlFlow::BreaksLoop)
    }

    fn continues_loop() -> Self {
        Self::from_outcome(BodyControlFlow::ContinuesLoop)
    }

    fn from_outcome(outcome: BodyControlFlow) -> Self {
        Self {
            outcomes: vec![outcome],
        }
    }

    fn includes(&self, outcome: BodyControlFlow) -> bool {
        self.outcomes.contains(&outcome)
    }

    fn is_exactly_returns(&self) -> bool {
        self.outcomes == [BodyControlFlow::Returns]
    }

    fn follow_with(&mut self, following_outcomes: &Self) {
        if !self.includes(BodyControlFlow::ReachesEnd) {
            return;
        }
        self.outcomes
            .retain(|outcome| *outcome != BodyControlFlow::ReachesEnd);
        self.union_with(following_outcomes);
    }

    fn union_with(&mut self, other: &Self) {
        for outcome in &other.outcomes {
            if !self.includes(*outcome) {
                self.outcomes.push(*outcome);
            }
        }
    }
}

/// Validates one function's parameters, locals, statements, and return contract.
pub(super) struct FunctionChecker<'context, 'program> {
    check_context: &'context mut ProgramCheckContext<'program>,
    loop_nesting: usize,
}

/// Keeps function-scope mutation separate from program orchestration and expression rules.
impl<'context, 'program> FunctionChecker<'context, 'program> {
    /// Borrows the active program context for one complete function check.
    pub(super) const fn from_context(
        check_context: &'context mut ProgramCheckContext<'program>,
    ) -> Self {
        Self {
            check_context,
            loop_nesting: 0,
        }
    }

    /// Produces a checked function only after its whole local scope and return contract validate.
    pub(super) fn check_function(
        &mut self,
        parsed_function: &ParsedFunction,
    ) -> Result<CheckedFunction, CompilationProblem> {
        if parsed_function.visibility() == crate::source_language::ParsedFunctionVisibility::Public
        {
            for parsed_parameter in parsed_function.function_parameters() {
                if let Some((_, record_name_range)) =
                    parsed_parameter.value_type().named_record_parts()
                {
                    return Err(CompilationProblem::from_problem_at_range((
                        record_name_range,
                        CompilationProblemReason::FilePrivateRecordTypeCannotBePublic,
                    )));
                }
            }
            if let Some((_, record_name_range)) =
                parsed_function.returned_value_type().named_record_parts()
            {
                return Err(CompilationProblem::from_problem_at_range((
                    record_name_range,
                    CompilationProblemReason::FilePrivateRecordTypeCannotBePublic,
                )));
            }
        }
        match self.check_context.begin_function(parsed_function) {
            Ok(()) => {}
            Err(compilation_problem) => return Err(compilation_problem),
        }
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
            let checked_value_type = match self
                .check_context
                .resolve_value_type(&parsed_parameter.value_type())
            {
                Ok(checked_value_type) => checked_value_type,
                Err(compilation_problem) => return Err(compilation_problem),
            };
            let parameter_binding = CheckedLocalBinding::from_parameter((
                parsed_parameter.parameter_name().to_owned(),
                checked_value_type.clone(),
                parsed_parameter.parameter_name_range(),
            ))?;
            self.check_context.add_local_binding(parameter_binding);
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
        match self.check_context.expected_returned_value_type() {
            CheckedValueType::Number
            | CheckedValueType::String
            | CheckedValueType::Boolean
            | CheckedValueType::NamedRecord(_)
            | CheckedValueType::RobloxService(_)
            | CheckedValueType::RobloxInstance(_)
            | CheckedValueType::Function { .. }
            | CheckedValueType::Array(_)
                if !function_completion.is_exactly_returns() =>
            {
                return Err(CompilationProblem::from_problem_at_range((
                    parsed_function.function_name_range(),
                    CompilationProblemReason::MissingReturn,
                )));
            }
            CheckedValueType::Number
            | CheckedValueType::String
            | CheckedValueType::Boolean
            | CheckedValueType::NamedRecord(_)
            | CheckedValueType::RobloxService(_)
            | CheckedValueType::RobloxInstance(_)
            | CheckedValueType::Function { .. }
            | CheckedValueType::Array(_)
            | CheckedValueType::NoReturnedValues => {}
        }
        Ok(CheckedFunction::from_checked_declaration((
            parsed_function.function_name().to_owned(),
            checked_parameters,
            self.check_context.expected_returned_value_type(),
            checked_function_body,
        )))
    }

    pub(super) fn check_function_literal(
        &mut self,
        parsed_function_literal: &ParsedFunctionLiteral,
    ) -> Result<
        (
            CheckedFunctionLiteral,
            CheckedValueType,
            Vec<CheckedValueType>,
        ),
        CompilationProblem,
    > {
        let enclosing_returned_value_type = self.check_context.expected_returned_value_type();
        let local_scope_boundary = self.check_context.local_scope_boundary();
        let checked_returned_value_type = match self
            .check_context
            .begin_nested_function(&parsed_function_literal.returned_value_type())
        {
            Ok(checked_returned_value_type) => checked_returned_value_type,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        let result =
            self.check_function_literal_body(parsed_function_literal, checked_returned_value_type);
        self.check_context.end_local_scope(local_scope_boundary);
        self.check_context
            .restore_expected_returned_value_type(enclosing_returned_value_type);
        result
    }

    fn check_function_literal_body(
        &mut self,
        parsed_function_literal: &ParsedFunctionLiteral,
        checked_returned_value_type: CheckedValueType,
    ) -> Result<
        (
            CheckedFunctionLiteral,
            CheckedValueType,
            Vec<CheckedValueType>,
        ),
        CompilationProblem,
    > {
        let mut checked_parameters = Vec::new();
        for parsed_parameter in parsed_function_literal.function_parameters() {
            let parsed_value_type = parsed_parameter.value_type();
            ProgramCheckContext::reject_service_type_outside_local_acquisition(&parsed_value_type)?;
            DeclarationNameChecker::check_local_name((
                self.check_context,
                parsed_parameter.parameter_name(),
                parsed_parameter.parameter_name_range(),
            ))?;
            let checked_value_type = self.check_context.resolve_value_type(&parsed_value_type)?;
            self.check_context
                .add_local_binding(CheckedLocalBinding::from_parameter((
                    parsed_parameter.parameter_name().to_owned(),
                    checked_value_type.clone(),
                    parsed_parameter.parameter_name_range(),
                ))?);
            checked_parameters.push(CheckedParameter::from_checked_declaration((
                parsed_parameter.parameter_name().to_owned(),
                checked_value_type,
            )));
        }
        let (checked_function_body, function_completion) =
            self.check_function_body(parsed_function_literal.function_body())?;
        if !matches!(
            checked_returned_value_type,
            CheckedValueType::NoReturnedValues
        ) && !function_completion.is_exactly_returns()
        {
            return Err(CompilationProblem::from_problem_at_range((
                parsed_function_literal.expression_range(),
                CompilationProblemReason::MissingReturn,
            )));
        }
        let parameter_types = checked_parameters
            .iter()
            .map(CheckedParameter::value_type)
            .collect();
        Ok((
            CheckedFunctionLiteral::from_parts((
                checked_parameters,
                checked_returned_value_type.clone(),
                checked_function_body,
            )),
            checked_returned_value_type,
            parameter_types,
        ))
    }

    fn check_function_body(
        &mut self,
        parsed_function_body: &ParsedFunctionBody,
    ) -> Result<(CheckedFunctionBody, BodyControlFlows), CompilationProblem> {
        let mut checked_statements = Vec::new();
        let mut function_completion = BodyControlFlows::reaches_end();
        for parsed_statement in parsed_function_body.body_statements() {
            if !function_completion.includes(BodyControlFlow::ReachesEnd) {
                return Err(CompilationProblem::from_problem_at_range((
                    parsed_statement.source_range(),
                    CompilationProblemReason::SourceDoesNotFollowLanguageRules,
                )));
            }
            let (checked_statement, statement_completion) =
                match self.check_statement(parsed_statement) {
                    Ok(checked_statement) => checked_statement,
                    Err(compilation_problem) => return Err(compilation_problem),
                };
            checked_statements.push(checked_statement);
            function_completion.follow_with(&statement_completion);
        }
        Ok((
            CheckedFunctionBody::from_statements(checked_statements),
            function_completion,
        ))
    }

    fn check_statement(
        &mut self,
        parsed_statement: &ParsedStatement,
    ) -> Result<(CheckedStatement, BodyControlFlows), CompilationProblem> {
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
                let checked_value_type =
                    match self.check_context.resolve_local_value_type(value_type) {
                        Ok(checked_value_type) => checked_value_type,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
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
                    checked_value_type.clone(),
                    initial_value.source_range(),
                )) {
                    Ok(()) => {}
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                let local_binding = CheckedLocalBinding::from_immutable_declaration((
                    local_name.to_owned(),
                    checked_value_type.clone(),
                    &checked_initial_value,
                    initial_value.source_range(),
                ))?;
                self.check_context.add_local_binding(local_binding);
                Ok((
                    CheckedStatement::ImmutableLocal {
                        local_name: local_name.to_owned(),
                        value_type: checked_value_type,
                        initial_value: checked_initial_value,
                    },
                    BodyControlFlows::reaches_end(),
                ))
            }
            ParsedStatement::MutableLocal {
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
                let checked_value_type =
                    match self.check_context.resolve_local_value_type(value_type) {
                        Ok(checked_value_type) => checked_value_type,
                        Err(compilation_problem) => return Err(compilation_problem),
                    };
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
                    checked_value_type.clone(),
                    initial_value.source_range(),
                )) {
                    Ok(()) => {}
                    Err(compilation_problem) => return Err(compilation_problem),
                }
                let local_binding = CheckedLocalBinding::from_mutable_declaration((
                    local_name.to_owned(),
                    checked_value_type.clone(),
                    initial_value.source_range(),
                ))?;
                self.check_context.add_local_binding(local_binding);
                Ok((
                    CheckedStatement::MutableLocal {
                        local_name: local_name.to_owned(),
                        value_type: checked_value_type,
                        initial_value: checked_initial_value,
                    },
                    BodyControlFlows::reaches_end(),
                ))
            }
            ParsedStatement::AssignLocal {
                local_name,
                local_name_range,
                assigned_value,
            } => self.check_local_assignment((local_name, *local_name_range, assigned_value)),
            ParsedStatement::AssignPlace(place_assignment) => {
                self.check_place_assignment(place_assignment)
            }
            ParsedStatement::CallFunctionAndIgnoreResult(parsed_function_call) => {
                let mut expression_checker = ExpressionChecker::from_context(self.check_context);
                match expression_checker.check_function_call(parsed_function_call) {
                    Ok((checked_function_call, _)) => Ok((
                        CheckedStatement::CallFunctionAndIgnoreResult(checked_function_call),
                        BodyControlFlows::reaches_end(),
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
                        BodyControlFlows::returns(),
                    )),
                    Err(compilation_problem) => Err(compilation_problem),
                }
            }
            ParsedStatement::BreaksLoop(keyword_range) => {
                if self.loop_nesting == 0 {
                    return Err(CompilationProblem::from_problem_at_range((
                        *keyword_range,
                        CompilationProblemReason::SourceDoesNotFollowLanguageRules,
                    )));
                }
                Ok((
                    CheckedStatement::BreaksLoop,
                    BodyControlFlows::breaks_loop(),
                ))
            }
            ParsedStatement::ContinuesLoop(keyword_range) => {
                if self.loop_nesting == 0 {
                    return Err(CompilationProblem::from_problem_at_range((
                        *keyword_range,
                        CompilationProblemReason::SourceDoesNotFollowLanguageRules,
                    )));
                }
                Ok((
                    CheckedStatement::ContinuesLoop,
                    BodyControlFlows::continues_loop(),
                ))
            }
            ParsedStatement::IfElse(parsed_if_else) => self.check_if_else(parsed_if_else),
            ParsedStatement::WhileLoop(parsed_while_loop) => {
                self.check_while_loop(parsed_while_loop)
            }
        }
    }

    fn check_local_assignment(
        &mut self,
        assignment_parts: (
            &str,
            crate::SourceRange,
            &crate::source_language::ParsedExpression,
        ),
    ) -> Result<(CheckedStatement, BodyControlFlows), CompilationProblem> {
        let (local_name, local_name_range, assigned_value) = assignment_parts;
        let assignment_contract = self
            .check_context
            .local_bindings()
            .iter()
            .rev()
            .find(|local_binding| local_binding.local_name() == local_name)
            .map(CheckedLocalBinding::assignment_contract);
        let expected_type = match assignment_contract {
            Some(LocalAssignmentContract::Forbidden) => {
                return Err(CompilationProblem::from_problem_at_range((
                    local_name_range,
                    CompilationProblemReason::ImmutableBindingCannotBeAssigned,
                )));
            }
            Some(LocalAssignmentContract::Allowed(value_type)) => value_type,
            None => {
                return Err(CompilationProblem::from_problem_at_range((
                    local_name_range,
                    CompilationProblemReason::UnknownName,
                )));
            }
        };
        let (checked_assigned_value, actual_type) = {
            let mut expression_checker = ExpressionChecker::from_context(self.check_context);
            match expression_checker.check_expression(assigned_value) {
                Ok(checked_expression) => checked_expression,
                Err(compilation_problem) => return Err(compilation_problem),
            }
        };
        match ExpressionChecker::require_matching_type((
            actual_type,
            expected_type,
            assigned_value.source_range(),
        )) {
            Ok(()) => Ok((
                CheckedStatement::AssignLocal {
                    local_name: local_name.to_owned(),
                    assigned_value: checked_assigned_value,
                },
                BodyControlFlows::reaches_end(),
            )),
            Err(compilation_problem) => Err(compilation_problem),
        }
    }

    fn check_place_assignment(
        &mut self,
        place_assignment: &ParsedPlaceAssignment,
    ) -> Result<(CheckedStatement, BodyControlFlows), CompilationProblem> {
        let root_binding_name = place_assignment.root_binding_name();
        let root_binding_range = place_assignment.root_binding_range();
        let assignment_contract = self
            .check_context
            .local_bindings()
            .iter()
            .rev()
            .find(|local_binding| local_binding.local_name() == root_binding_name)
            .map(CheckedLocalBinding::assignment_contract);
        let mut current_type = match assignment_contract {
            Some(LocalAssignmentContract::Forbidden) => {
                return Err(CompilationProblem::from_problem_at_range((
                    root_binding_range,
                    CompilationProblemReason::ImmutableBindingCannotBeAssigned,
                )));
            }
            Some(LocalAssignmentContract::Allowed(value_type)) => value_type,
            None => {
                return Err(CompilationProblem::from_problem_at_range((
                    root_binding_range,
                    CompilationProblemReason::UnknownName,
                )));
            }
        };
        let mut checked_steps = Vec::new();
        for step in place_assignment.steps() {
            match step {
                ParsedPlaceStep::Field {
                    field_name,
                    field_range,
                    base_range,
                } => {
                    current_type = match current_type {
                        CheckedValueType::NamedRecord(record_name) => {
                            let record_declaration = self
                                .check_context
                                .checked_record_declaration((&record_name, *field_range))?;
                            let Some(record_field) = record_declaration
                                .record_fields()
                                .iter()
                                .find(|record_field| record_field.field_name() == field_name)
                            else {
                                return Err(CompilationProblem::from_problem_at_range((
                                    *field_range,
                                    CompilationProblemReason::UnknownRecordAccessField,
                                )));
                            };
                            record_field.value_type().clone()
                        }
                        CheckedValueType::RobloxInstance(roblox_instance) => {
                            roblox_instance.property_type(field_name).ok_or_else(|| {
                                CompilationProblem::from_problem_at_range((
                                    *field_range,
                                    CompilationProblemReason::UnknownRobloxInstanceMember,
                                ))
                            })?
                        }
                        _ => {
                            return Err(CompilationProblem::from_problem_at_range((
                                *base_range,
                                CompilationProblemReason::FieldAccessRequiresRecord,
                            )));
                        }
                    };
                    checked_steps.push(CheckedPlaceStep::Field(field_name.to_owned()));
                }
                ParsedPlaceStep::Index {
                    index_expression,
                    base_range,
                } => {
                    let CheckedValueType::Array(element_type) = current_type else {
                        return Err(CompilationProblem::from_problem_at_range((
                            *base_range,
                            CompilationProblemReason::TypesDoNotMatch,
                        )));
                    };
                    let (checked_index, index_type) = {
                        let mut expression_checker =
                            ExpressionChecker::from_context(self.check_context);
                        expression_checker.check_expression(index_expression)?
                    };
                    ExpressionChecker::require_matching_type((
                        index_type,
                        CheckedValueType::Number,
                        index_expression.source_range(),
                    ))?;
                    current_type = *element_type;
                    checked_steps.push(CheckedPlaceStep::Index(checked_index));
                }
            }
        }
        let (checked_assigned_value, actual_type) = {
            let mut expression_checker = ExpressionChecker::from_context(self.check_context);
            match expression_checker.check_expression(place_assignment.assigned_value()) {
                Ok(checked_expression) => checked_expression,
                Err(compilation_problem) => return Err(compilation_problem),
            }
        };
        match ExpressionChecker::require_matching_type((
            actual_type,
            current_type,
            place_assignment.assigned_value().source_range(),
        )) {
            Ok(()) => {}
            Err(compilation_problem) => return Err(compilation_problem),
        }
        Ok((
            CheckedStatement::AssignPlace(CheckedPlaceAssignment::from_parts((
                root_binding_name.to_owned(),
                checked_steps,
                checked_assigned_value,
            ))),
            BodyControlFlows::reaches_end(),
        ))
    }

    fn check_if_else(
        &mut self,
        parsed_if_else: &ParsedIfElse,
    ) -> Result<(CheckedStatement, BodyControlFlows), CompilationProblem> {
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
        let mut if_else_completion = then_completion;
        if_else_completion.union_with(&else_completion);
        Ok((
            CheckedStatement::IfElse(CheckedIfElse::from_parts((
                checked_condition,
                checked_then_body,
                checked_else_body,
            ))),
            if_else_completion,
        ))
    }

    fn check_while_loop(
        &mut self,
        parsed_while_loop: &ParsedWhileLoop,
    ) -> Result<(CheckedStatement, BodyControlFlows), CompilationProblem> {
        let (checked_condition, condition_type) = {
            let mut expression_checker = ExpressionChecker::from_context(self.check_context);
            match expression_checker.check_expression(parsed_while_loop.condition()) {
                Ok(checked_condition) => checked_condition,
                Err(compilation_problem) => return Err(compilation_problem),
            }
        };
        match ExpressionChecker::require_matching_type((
            condition_type,
            CheckedValueType::Boolean,
            parsed_while_loop.condition_range(),
        )) {
            Ok(()) => {}
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let local_scope_boundary = self.check_context.local_scope_boundary();
        self.loop_nesting += 1;
        let checked_body_and_completion = self.check_function_body(parsed_while_loop.body());
        self.loop_nesting -= 1;
        let (checked_body, body_completion) = match checked_body_and_completion {
            Ok(checked_body) => checked_body,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        self.check_context.end_local_scope(local_scope_boundary);
        let mut while_completion = BodyControlFlows::reaches_end();
        if body_completion.includes(BodyControlFlow::Returns) {
            while_completion.union_with(&BodyControlFlows::returns());
        }
        Ok((
            CheckedStatement::WhileLoop(CheckedWhileLoop::from_parts((
                checked_condition,
                checked_body,
            ))),
            while_completion,
        ))
    }
}
