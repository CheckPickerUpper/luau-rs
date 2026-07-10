use crate::{
    checked_program::{
        CheckedExpression, CheckedFunction, CheckedFunctionCall, CheckedFunctionReturn,
        CheckedParameter, CheckedProgram, CheckedStatement, CheckedValueType,
    },
    generated_luau::{
        LuauExpression, LuauFunction, LuauFunctionCall, LuauFunctionReturn, LuauParameter,
        LuauProgram, LuauStatement, LuauValueType,
    },
};

const ENTRY_FUNCTION_NAME: &str = "main";

/// Lowers a semantically checked program into an owned Luau representation.
pub(crate) fn generate_luau_program(checked_program: &CheckedProgram) -> LuauProgram {
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
        let luau_statements = checked_function
            .statements()
            .iter()
            .map(Self::generate_statement)
            .collect();

        LuauFunction::from_function_parts((
            checked_function.function_name().to_owned(),
            luau_parameters,
            Self::generate_value_type(checked_function.returned_value_type()),
            luau_statements,
            Self::generate_function_return(checked_function.function_return()),
        ))
    }

    fn generate_parameter(checked_parameter: &CheckedParameter) -> LuauParameter {
        LuauParameter::from_name_and_type((
            checked_parameter.parameter_name().to_owned(),
            Self::generate_value_type(checked_parameter.value_type()),
        ))
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
        }
    }

    fn generate_function_return(
        checked_function_return: &CheckedFunctionReturn,
    ) -> LuauFunctionReturn {
        match checked_function_return {
            CheckedFunctionReturn::NoReturn => LuauFunctionReturn::NoReturn,
            CheckedFunctionReturn::ReturnsValue(returned_expression) => {
                LuauFunctionReturn::ReturnsValue(Self::generate_expression(returned_expression))
            }
        }
    }

    fn generate_expression(checked_expression: &CheckedExpression) -> LuauExpression {
        match checked_expression {
            CheckedExpression::NameReference(reference_name) => {
                LuauExpression::NameReference(reference_name.to_owned())
            }
            CheckedExpression::NumberLiteral(number_literal) => {
                LuauExpression::NumberLiteral(number_literal.to_owned())
            }
            CheckedExpression::Addition {
                left_operand,
                right_operand,
            } => LuauExpression::Addition {
                left_operand: Box::new(Self::generate_expression(left_operand)),
                right_operand: Box::new(Self::generate_expression(right_operand)),
            },
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

    fn generate_value_type(checked_value_type: CheckedValueType) -> LuauValueType {
        match checked_value_type {
            CheckedValueType::Number => LuauValueType::Number,
            CheckedValueType::NoReturnedValues => LuauValueType::NoReturnedValues,
        }
    }
}
