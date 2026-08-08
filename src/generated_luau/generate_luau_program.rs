use crate::{
    checked_program::{
        CheckedBooleanLiteral, CheckedExpression, CheckedFunction, CheckedFunctionBody,
        CheckedFunctionCall, CheckedFunctionLiteral, CheckedIfElse, CheckedParameter,
        CheckedProgram, CheckedRecordDeclaration, CheckedRobloxRemoteOperation, CheckedStatement,
        CheckedValueType, CheckedWhileLoop,
    },
    generated_luau::{
        LuauArrayLiteral, LuauArrayRead, LuauBooleanLiteral, LuauComparisonOperation,
        LuauComparisonOperator, LuauEqualityOperation, LuauEqualityOperator, LuauExpression,
        LuauFieldRead, LuauFunction, LuauFunctionBody, LuauFunctionCall, LuauFunctionLiteral,
        LuauIfElse, LuauInstanceLookup, LuauLogicalNegation, LuauLogicalOperation,
        LuauLogicalOperator, LuauNumericOperation, LuauNumericOperator, LuauParameter,
        LuauPlaceAssignment, LuauPlaceStep, LuauProgram, LuauRecordAlias, LuauRecordField,
        LuauRecordFieldInitializer, LuauRecordLiteral, LuauRobloxRemoteOperation, LuauStatement,
        LuauValueType, LuauWhileLoop,
    },
};

const ENTRY_FUNCTION_NAME: &str = "main";

/// Lowers a semantically checked program into an owned Luau representation.
pub fn generate_luau_program(checked_program: &CheckedProgram) -> LuauProgram {
    LuauProgramGenerator::generate_executable(checked_program)
}

/// Lowers a checked library module without adding an eager entrypoint call.
pub fn generate_luau_library(checked_program: &CheckedProgram) -> LuauProgram {
    LuauProgramGenerator::generate_library(checked_program)
}

struct LuauProgramGenerator;

/// Isolates recursive lowering from the target model and text writer.
impl LuauProgramGenerator {
    fn generate_executable(checked_program: &CheckedProgram) -> LuauProgram {
        let luau_functions = Self::generate_functions(checked_program);
        let entry_function_call = LuauExpression::FunctionCall(LuauFunctionCall::from_call((
            ENTRY_FUNCTION_NAME.to_owned(),
            Vec::new(),
        )));

        LuauProgram::from_program_parts((
            Self::generate_record_aliases(checked_program),
            luau_functions,
            entry_function_call,
        ))
    }

    fn generate_library(checked_program: &CheckedProgram) -> LuauProgram {
        LuauProgram::from_library_declarations((
            Self::generate_record_aliases(checked_program),
            Self::generate_functions(checked_program),
        ))
    }

    fn generate_record_aliases(checked_program: &CheckedProgram) -> Vec<LuauRecordAlias> {
        checked_program
            .records()
            .iter()
            .map(Self::generate_record_alias)
            .collect()
    }

    fn generate_record_alias(checked_record: &CheckedRecordDeclaration) -> LuauRecordAlias {
        LuauRecordAlias::from_alias((
            checked_record.record_name().to_owned(),
            checked_record
                .record_fields()
                .iter()
                .map(|field| {
                    LuauRecordField::from_field((
                        field.field_name().to_owned(),
                        Self::generate_value_type(field.value_type().clone()),
                    ))
                })
                .collect(),
        ))
    }

    fn generate_functions(checked_program: &CheckedProgram) -> Vec<LuauFunction> {
        let luau_functions = checked_program
            .functions()
            .iter()
            .map(Self::generate_function)
            .collect();
        luau_functions
    }

    fn generate_function(checked_function: &CheckedFunction) -> LuauFunction {
        let luau_parameters = checked_function
            .function_parameters()
            .iter()
            .map(Self::generate_parameter)
            .collect();
        LuauFunction::from_function_parts((
            checked_function.function_name().to_owned(),
            luau_parameters,
            Self::generate_value_type(checked_function.returned_value_type()),
            Self::generate_function_body(checked_function.function_body()),
        ))
    }

    fn generate_parameter(checked_parameter: &CheckedParameter) -> LuauParameter {
        LuauParameter::from_name_and_type((
            checked_parameter.parameter_name().to_owned(),
            Self::generate_value_type(checked_parameter.value_type()),
        ))
    }

    fn generate_function_body(checked_function_body: &CheckedFunctionBody) -> LuauFunctionBody {
        LuauFunctionBody::from_statements(
            checked_function_body
                .body_statements()
                .iter()
                .map(Self::generate_statement)
                .collect(),
        )
    }

    fn generate_statement(checked_statement: &CheckedStatement) -> LuauStatement {
        match checked_statement {
            CheckedStatement::ImmutableLocal {
                local_name,
                value_type,
                initial_value,
            } => LuauStatement::ImmutableLocal {
                local_name: local_name.to_owned(),
                value_type: Self::generate_value_type(value_type.clone()),
                initial_value: Self::generate_expression(initial_value),
            },
            CheckedStatement::MutableLocal {
                local_name,
                value_type,
                initial_value,
            } => LuauStatement::MutableLocal {
                local_name: local_name.to_owned(),
                value_type: Self::generate_value_type(value_type.clone()),
                initial_value: Self::generate_expression(initial_value),
            },
            CheckedStatement::AssignLocal {
                local_name,
                assigned_value,
            } => LuauStatement::AssignLocal {
                local_name: local_name.to_owned(),
                assigned_value: Self::generate_expression(assigned_value),
            },
            CheckedStatement::AssignPlace(place_assignment) => {
                LuauStatement::AssignPlace(LuauPlaceAssignment::from_parts((
                    place_assignment.root_binding_name().to_owned(),
                    place_assignment
                        .steps()
                        .iter()
                        .map(|step| match step {
                            crate::checked_program::CheckedPlaceStep::Field(field_name) => {
                                LuauPlaceStep::Field(field_name.to_owned())
                            }
                            crate::checked_program::CheckedPlaceStep::Index(index_expression) => {
                                LuauPlaceStep::Index(Self::generate_expression(index_expression))
                            }
                        })
                        .collect(),
                    Self::generate_expression(place_assignment.assigned_value()),
                )))
            }
            CheckedStatement::CallFunctionAndIgnoreResult(checked_function_call) => {
                LuauStatement::CallFunctionAndIgnoreResult(Self::generate_function_call(
                    checked_function_call,
                ))
            }
            CheckedStatement::RobloxRemoteOperation(operation) => {
                LuauStatement::RobloxRemoteOperation(Self::generate_remote_operation(operation))
            }
            CheckedStatement::ReturnsValue(checked_expression) => {
                LuauStatement::ReturnsValue(Self::generate_expression(checked_expression))
            }
            CheckedStatement::BreaksLoop => LuauStatement::BreaksLoop,
            CheckedStatement::ContinuesLoop => LuauStatement::ContinuesLoop,
            CheckedStatement::IfElse(checked_if_else) => {
                LuauStatement::IfElse(Self::generate_if_else(checked_if_else))
            }
            CheckedStatement::WhileLoop(checked_while_loop) => {
                LuauStatement::WhileLoop(Self::generate_while_loop(checked_while_loop))
            }
        }
    }

    fn generate_if_else(checked_if_else: &CheckedIfElse) -> LuauIfElse {
        LuauIfElse::from_parts((
            Self::generate_expression(checked_if_else.condition()),
            Self::generate_function_body(checked_if_else.then_body()),
            Self::generate_function_body(checked_if_else.else_body()),
        ))
    }

    fn generate_while_loop(checked_while_loop: &CheckedWhileLoop) -> LuauWhileLoop {
        LuauWhileLoop::from_parts((
            Self::generate_expression(checked_while_loop.condition()),
            Self::generate_function_body(checked_while_loop.body()),
        ))
    }

    fn generate_expression(checked_expression: &CheckedExpression) -> LuauExpression {
        match checked_expression {
            CheckedExpression::NameReference(reference_name) => {
                LuauExpression::NameReference(reference_name.to_owned())
            }
            CheckedExpression::FunctionReference(function_name) => {
                LuauExpression::FunctionReference(function_name.to_owned())
            }
            CheckedExpression::NumberLiteral(number_literal) => {
                LuauExpression::NumberLiteral(number_literal.to_owned())
            }
            CheckedExpression::StringLiteral(string_literal) => {
                LuauExpression::StringLiteral(string_literal.to_owned())
            }
            CheckedExpression::BooleanLiteral(checked_boolean_literal) => {
                let luau_boolean_literal = match checked_boolean_literal {
                    CheckedBooleanLiteral::True => LuauBooleanLiteral::True,
                    CheckedBooleanLiteral::False => LuauBooleanLiteral::False,
                };
                LuauExpression::BooleanLiteral(luau_boolean_literal)
            }
            CheckedExpression::RobloxServiceAcquisition(roblox_service) => {
                LuauExpression::RobloxServiceAcquisition(roblox_service.canonical_name().to_owned())
            }
            CheckedExpression::RobloxInstanceAcquisition(construction) => {
                LuauExpression::RobloxInstanceAcquisition(
                    crate::generated_luau::LuauInstanceConstruction::from_parts((
                        construction.instance().canonical_name().to_owned(),
                        construction
                            .parent_expression()
                            .map(|parent| Box::new(Self::generate_expression(parent))),
                    )),
                )
            }
            CheckedExpression::RobloxInstanceWaitForChild(lookup) => {
                LuauExpression::RobloxInstanceWaitForChild(LuauInstanceLookup::from_parts((
                    lookup.instance().canonical_name().to_owned(),
                    Box::new(Self::generate_expression(lookup.parent_expression())),
                    Box::new(Self::generate_expression(lookup.child_name_expression())),
                )))
            }
            CheckedExpression::ArrayLiteral(array_literal) => {
                LuauExpression::ArrayLiteral(LuauArrayLiteral::from_elements(
                    array_literal
                        .element_expressions()
                        .iter()
                        .map(Self::generate_expression)
                        .collect(),
                ))
            }
            CheckedExpression::RecordLiteral(record_literal) => {
                LuauExpression::RecordLiteral(LuauRecordLiteral::from_initializers(
                    record_literal
                        .field_initializers()
                        .iter()
                        .map(|initializer| {
                            LuauRecordFieldInitializer::from_initializer((
                                initializer.field_name().to_owned(),
                                Self::generate_expression(initializer.initialized_value()),
                            ))
                        })
                        .collect(),
                ))
            }
            CheckedExpression::FieldRead(field_read) => {
                LuauExpression::FieldRead(LuauFieldRead::from_read((
                    Box::new(Self::generate_expression(field_read.base_expression())),
                    field_read.field_name().to_owned(),
                )))
            }
            CheckedExpression::ArrayRead(array_read) => {
                LuauExpression::ArrayRead(LuauArrayRead::from_read((
                    Box::new(Self::generate_expression(array_read.base_expression())),
                    Box::new(Self::generate_expression(array_read.index_expression())),
                )))
            }
            CheckedExpression::NumericOperation(operation) => {
                let generated_operator = match operation.operator() {
                    crate::checked_program::CheckedNumericOperator::Addition => {
                        LuauNumericOperator::Addition
                    }
                    crate::checked_program::CheckedNumericOperator::Subtraction => {
                        LuauNumericOperator::Subtraction
                    }
                    crate::checked_program::CheckedNumericOperator::Multiplication => {
                        LuauNumericOperator::Multiplication
                    }
                    crate::checked_program::CheckedNumericOperator::Division => {
                        LuauNumericOperator::Division
                    }
                };
                LuauExpression::NumericOperation(LuauNumericOperation::from_parts((
                    Box::new(Self::generate_expression(operation.left_operand())),
                    Box::new(Self::generate_expression(operation.right_operand())),
                    generated_operator,
                )))
            }
            CheckedExpression::ComparisonOperation(operation) => {
                let generated_operator = match operation.operator() {
                    crate::checked_program::CheckedComparisonOperator::LessThan => {
                        LuauComparisonOperator::LessThan
                    }
                    crate::checked_program::CheckedComparisonOperator::LessThanOrEqual => {
                        LuauComparisonOperator::LessThanOrEqual
                    }
                    crate::checked_program::CheckedComparisonOperator::GreaterThan => {
                        LuauComparisonOperator::GreaterThan
                    }
                    crate::checked_program::CheckedComparisonOperator::GreaterThanOrEqual => {
                        LuauComparisonOperator::GreaterThanOrEqual
                    }
                };
                LuauExpression::ComparisonOperation(LuauComparisonOperation::from_parts((
                    Box::new(Self::generate_expression(operation.left_operand())),
                    Box::new(Self::generate_expression(operation.right_operand())),
                    generated_operator,
                )))
            }
            CheckedExpression::EqualityOperation(operation) => {
                let generated_operator = match operation.operator() {
                    crate::checked_program::CheckedEqualityOperator::Equal => {
                        LuauEqualityOperator::Equal
                    }
                    crate::checked_program::CheckedEqualityOperator::NotEqual => {
                        LuauEqualityOperator::NotEqual
                    }
                };
                LuauExpression::EqualityOperation(LuauEqualityOperation::from_parts((
                    Box::new(Self::generate_expression(operation.left_operand())),
                    Box::new(Self::generate_expression(operation.right_operand())),
                    generated_operator,
                )))
            }
            CheckedExpression::LogicalNegation(negation) => {
                LuauExpression::LogicalNegation(LuauLogicalNegation::from_expression(Box::new(
                    Self::generate_expression(negation.negated_expression()),
                )))
            }
            CheckedExpression::LogicalOperation(operation) => {
                let generated_operator = match operation.operator() {
                    crate::checked_program::CheckedLogicalOperator::Conjunction => {
                        LuauLogicalOperator::Conjunction
                    }
                    crate::checked_program::CheckedLogicalOperator::Disjunction => {
                        LuauLogicalOperator::Disjunction
                    }
                };
                LuauExpression::LogicalOperation(LuauLogicalOperation::from_parts((
                    Box::new(Self::generate_expression(operation.left_operand())),
                    Box::new(Self::generate_expression(operation.right_operand())),
                    generated_operator,
                )))
            }
            CheckedExpression::FunctionCall(checked_function_call) => {
                LuauExpression::FunctionCall(Self::generate_function_call(checked_function_call))
            }
            CheckedExpression::FunctionLiteral(function_literal) => {
                LuauExpression::FunctionLiteral(Self::generate_function_literal(function_literal))
            }
            CheckedExpression::RobloxRemoteOperation(operation) => {
                LuauExpression::RobloxRemoteOperation(Self::generate_remote_operation(operation))
            }
        }
    }

    fn generate_remote_operation(
        checked_operation: &CheckedRobloxRemoteOperation,
    ) -> LuauRobloxRemoteOperation {
        match checked_operation {
            CheckedRobloxRemoteOperation::Connect {
                remote_expression,
                callback_expression,
                execution_side,
            } => LuauRobloxRemoteOperation::Connect {
                remote_expression: Box::new(Self::generate_expression(remote_expression)),
                callback_expression: Box::new(Self::generate_expression(callback_expression)),
                execution_side: *execution_side,
            },
            CheckedRobloxRemoteOperation::Disconnect {
                connection_expression,
            } => LuauRobloxRemoteOperation::Disconnect {
                connection_expression: Box::new(Self::generate_expression(connection_expression)),
            },
            CheckedRobloxRemoteOperation::FireServer {
                remote_expression,
                payload_expression,
            } => LuauRobloxRemoteOperation::FireServer {
                remote_expression: Box::new(Self::generate_expression(remote_expression)),
                payload_expression: Box::new(Self::generate_expression(payload_expression)),
            },
            CheckedRobloxRemoteOperation::FireClient {
                remote_expression,
                player_expression,
                payload_expression,
            } => LuauRobloxRemoteOperation::FireClient {
                remote_expression: Box::new(Self::generate_expression(remote_expression)),
                player_expression: Box::new(Self::generate_expression(player_expression)),
                payload_expression: Box::new(Self::generate_expression(payload_expression)),
            },
            CheckedRobloxRemoteOperation::FireAllClients {
                remote_expression,
                payload_expression,
            } => LuauRobloxRemoteOperation::FireAllClients {
                remote_expression: Box::new(Self::generate_expression(remote_expression)),
                payload_expression: Box::new(Self::generate_expression(payload_expression)),
            },
            CheckedRobloxRemoteOperation::InvokeServer {
                remote_expression,
                payload_expression,
            } => LuauRobloxRemoteOperation::InvokeServer {
                remote_expression: Box::new(Self::generate_expression(remote_expression)),
                payload_expression: Box::new(Self::generate_expression(payload_expression)),
            },
            CheckedRobloxRemoteOperation::InvokeClient {
                remote_expression,
                player_expression,
                payload_expression,
            } => LuauRobloxRemoteOperation::InvokeClient {
                remote_expression: Box::new(Self::generate_expression(remote_expression)),
                player_expression: Box::new(Self::generate_expression(player_expression)),
                payload_expression: Box::new(Self::generate_expression(payload_expression)),
            },
            CheckedRobloxRemoteOperation::SetCallback {
                remote_expression,
                callback_expression,
                execution_side,
            } => LuauRobloxRemoteOperation::SetCallback {
                remote_expression: Box::new(Self::generate_expression(remote_expression)),
                callback_expression: Box::new(Self::generate_expression(callback_expression)),
                execution_side: *execution_side,
            },
        }
    }

    fn generate_function_literal(
        checked_function_literal: &CheckedFunctionLiteral,
    ) -> LuauFunctionLiteral {
        LuauFunctionLiteral::from_parts((
            checked_function_literal
                .function_parameters()
                .iter()
                .map(Self::generate_parameter)
                .collect(),
            Self::generate_value_type(checked_function_literal.returned_value_type()),
            Self::generate_function_body(checked_function_literal.function_body()),
        ))
    }

    fn generate_function_call(checked_function_call: &CheckedFunctionCall) -> LuauFunctionCall {
        LuauFunctionCall::from_call((
            checked_function_call.function_name().to_owned(),
            checked_function_call
                .function_arguments()
                .iter()
                .map(Self::generate_expression)
                .collect(),
        ))
    }

    fn generate_value_type(checked_value_type: CheckedValueType) -> LuauValueType {
        match checked_value_type {
            CheckedValueType::Number => LuauValueType::Number,
            CheckedValueType::String => LuauValueType::String,
            CheckedValueType::Boolean => LuauValueType::Boolean,
            CheckedValueType::Array(element_type) => {
                LuauValueType::Array(Box::new(Self::generate_value_type(*element_type)))
            }
            CheckedValueType::Function {
                parameter_types,
                returned_value_type,
            } => LuauValueType::Function {
                parameter_types: parameter_types
                    .into_iter()
                    .map(Self::generate_value_type)
                    .collect(),
                returned_value_type: Box::new(Self::generate_value_type(*returned_value_type)),
            },
            CheckedValueType::NamedRecord(record_name) => LuauValueType::NamedRecord(record_name),
            CheckedValueType::RobloxService(roblox_service) => {
                LuauValueType::RobloxService(roblox_service.canonical_name().to_owned())
            }
            CheckedValueType::RobloxInstance(roblox_instance) => {
                LuauValueType::RobloxInstance(roblox_instance.canonical_name().to_owned())
            }
            CheckedValueType::RobloxConnection => LuauValueType::RobloxConnection,
            CheckedValueType::NoReturnedValues => LuauValueType::NoReturnedValues,
        }
    }
}
