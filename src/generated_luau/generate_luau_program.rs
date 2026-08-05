use crate::{
    checked_program::{
        CheckedBooleanLiteral, CheckedExpression, CheckedFunction, CheckedFunctionBody,
        CheckedFunctionCall, CheckedIfElse, CheckedParameter, CheckedProgram, CheckedStatement,
        CheckedValueType,
    },
    generated_luau::{
        LuauBooleanLiteral, LuauComparisonOperation, LuauComparisonOperator, LuauEqualityOperation,
        LuauEqualityOperator, LuauExpression, LuauFunction, LuauFunctionBody, LuauFunctionCall,
        LuauIfElse, LuauLogicalNegation, LuauLogicalOperation, LuauLogicalOperator,
        LuauNumericOperation, LuauNumericOperator, LuauParameter, LuauProgram, LuauStatement,
        LuauValueType,
    },
};

const ENTRY_FUNCTION_NAME: &str = "main";

/// Lowers a semantically checked program into an owned Luau representation.
pub fn generate_luau_program(checked_program: &CheckedProgram) -> LuauProgram {
    LuauProgramGenerator::generate(checked_program)
}

struct LuauProgramGenerator;

/// Isolates recursive lowering from the target model and text writer.
impl LuauProgramGenerator {
    fn generate(checked_program: &CheckedProgram) -> LuauProgram {
        let luau_functions = checked_program
            .functions()
            .iter()
            .map(Self::generate_function)
            .collect();
        let entry_function_call = LuauExpression::FunctionCall(LuauFunctionCall::from_call((
            ENTRY_FUNCTION_NAME.to_owned(),
            Vec::new(),
        )));

        LuauProgram::from_program_parts((luau_functions, entry_function_call))
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
                value_type: Self::generate_value_type(*value_type),
                initial_value: Self::generate_expression(initial_value),
            },
            CheckedStatement::CallFunctionAndIgnoreResult(checked_function_call) => {
                LuauStatement::CallFunctionAndIgnoreResult(Self::generate_function_call(
                    checked_function_call,
                ))
            }
            CheckedStatement::ReturnsValue(checked_expression) => {
                LuauStatement::ReturnsValue(Self::generate_expression(checked_expression))
            }
            CheckedStatement::IfElse(checked_if_else) => {
                LuauStatement::IfElse(Self::generate_if_else(checked_if_else))
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

    fn generate_expression(checked_expression: &CheckedExpression) -> LuauExpression {
        match checked_expression {
            CheckedExpression::NameReference(reference_name) => {
                LuauExpression::NameReference(reference_name.to_owned())
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
        }
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

    const fn generate_value_type(checked_value_type: CheckedValueType) -> LuauValueType {
        match checked_value_type {
            CheckedValueType::Number => LuauValueType::Number,
            CheckedValueType::String => LuauValueType::String,
            CheckedValueType::Boolean => LuauValueType::Boolean,
            CheckedValueType::NoReturnedValues => LuauValueType::NoReturnedValues,
        }
    }
}
