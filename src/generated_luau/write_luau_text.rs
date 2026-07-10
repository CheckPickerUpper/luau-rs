use crate::generated_luau::{
    GeneratedLuauText, LuauBooleanLiteral, LuauExpression, LuauFunction, LuauFunctionCall,
    LuauFunctionReturn, LuauNumericOperation, LuauNumericOperator, LuauParameter, LuauProgram,
    LuauStatement, LuauValueType,
};

const STATEMENT_INDENTATION: &str = "    ";

/// Serializes the owned target model into deterministic strict Luau text.
pub(crate) fn write_luau_text(luau_program: &LuauProgram) -> GeneratedLuauText {
    let mut luau_text_writer = LuauTextWriter::new();
    luau_text_writer.write_program(luau_program);
    GeneratedLuauText::from_text(luau_text_writer.finish())
}

struct LuauTextWriter {
    luau_text: String,
}

/// Centralizes layout and precedence decisions for generated Luau.
impl LuauTextWriter {
    fn new() -> Self {
        Self {
            luau_text: String::new(),
        }
    }

    fn finish(self) -> String {
        self.luau_text
    }

    fn write_program(&mut self, luau_program: &LuauProgram) {
        self.luau_text.push_str("--!strict\n\n");
        for luau_function in luau_program.program_functions() {
            self.write_function(luau_function);
            self.luau_text.push('\n');
        }
        self.write_expression(luau_program.entry_function_call());
        self.luau_text.push('\n');
    }

    fn write_function(&mut self, luau_function: &LuauFunction) {
        self.luau_text.push_str("local function ");
        self.luau_text.push_str(luau_function.function_name());
        self.luau_text.push('(');
        self.write_parameters(luau_function.function_parameters());
        self.luau_text.push_str("): ");
        self.write_value_type(luau_function.returned_value_type());
        self.luau_text.push('\n');

        for luau_statement in luau_function.function_statements() {
            self.luau_text.push_str(STATEMENT_INDENTATION);
            self.write_statement(luau_statement);
            self.luau_text.push('\n');
        }

        match luau_function.function_return() {
            LuauFunctionReturn::NoReturn => {}
            LuauFunctionReturn::ReturnsValue(returned_expression) => {
                self.luau_text.push_str(STATEMENT_INDENTATION);
                self.luau_text.push_str("return ");
                self.write_expression(returned_expression);
                self.luau_text.push('\n');
            }
        }

        self.luau_text.push_str("end\n");
    }

    fn write_parameters(&mut self, luau_parameters: &[LuauParameter]) {
        let (first_parameter, remaining_parameters) = match luau_parameters.split_first() {
            Some(parameter_sequence) => parameter_sequence,
            None => return,
        };
        self.write_parameter(first_parameter);
        for luau_parameter in remaining_parameters {
            self.luau_text.push_str(", ");
            self.write_parameter(luau_parameter);
        }
    }

    fn write_parameter(&mut self, luau_parameter: &LuauParameter) {
        self.luau_text.push_str(luau_parameter.parameter_name());
        self.luau_text.push_str(": ");
        self.write_value_type(luau_parameter.value_type());
    }

    fn write_statement(&mut self, luau_statement: &LuauStatement) {
        match luau_statement {
            LuauStatement::ImmutableLocal {
                local_name,
                value_type,
                initial_value,
            } => {
                self.luau_text.push_str("const ");
                self.luau_text.push_str(local_name);
                self.luau_text.push_str(": ");
                self.write_value_type(*value_type);
                self.luau_text.push_str(" = ");
                self.write_expression(initial_value);
            }
            LuauStatement::CallFunctionAndIgnoreResult(function_call) => {
                self.write_function_call(function_call);
            }
        }
    }

    fn write_expression(&mut self, luau_expression: &LuauExpression) {
        self.write_expression_in((
            luau_expression,
            LuauTextWriterExpressionPosition::Unrestricted,
        ));
    }

    fn write_expression_in(
        &mut self,
        expression_in_position: (&LuauExpression, LuauTextWriterExpressionPosition),
    ) {
        let (luau_expression, expression_position) = expression_in_position;
        match (luau_expression, expression_position) {
            (LuauExpression::NumericOperation(operation), expression_position) => {
                self.write_numeric_operation((operation, expression_position));
            }
            (LuauExpression::NameReference(reference_name), _) => {
                self.luau_text.push_str(reference_name);
            }
            (LuauExpression::NumberLiteral(number_literal), _) => {
                self.luau_text.push_str(number_literal);
            }
            (LuauExpression::StringLiteral(string_literal), _) => {
                self.luau_text.push_str(string_literal);
            }
            (LuauExpression::BooleanLiteral(boolean_literal), _) => match boolean_literal {
                LuauBooleanLiteral::True => self.luau_text.push_str("true"),
                LuauBooleanLiteral::False => self.luau_text.push_str("false"),
            },
            (LuauExpression::FunctionCall(function_call), _) => {
                self.write_function_call(function_call);
            }
        }
    }

    fn write_function_call(&mut self, function_call: &LuauFunctionCall) {
        self.luau_text.push_str(function_call.function_name());
        self.luau_text.push('(');
        self.write_call_arguments(function_call.function_arguments());
        self.luau_text.push(')');
    }

    fn write_numeric_operation(
        &mut self,
        operation_and_position: (&LuauNumericOperation, LuauTextWriterExpressionPosition),
    ) {
        let (operation, expression_position) = operation_and_position;
        let needs_parentheses = match expression_position {
            LuauTextWriterExpressionPosition::NumericOperationRightOperand => true,
            LuauTextWriterExpressionPosition::Unrestricted
            | LuauTextWriterExpressionPosition::NumericOperationLeftOperand
            | LuauTextWriterExpressionPosition::FunctionArgument => false,
        };
        if needs_parentheses {
            self.luau_text.push('(');
        }
        self.write_expression_in((
            operation.left_operand(),
            LuauTextWriterExpressionPosition::NumericOperationLeftOperand,
        ));
        match operation.operator() {
            LuauNumericOperator::Addition => self.luau_text.push_str(" + "),
            LuauNumericOperator::Subtraction => self.luau_text.push_str(" - "),
            LuauNumericOperator::Multiplication => self.luau_text.push_str(" * "),
            LuauNumericOperator::Division => self.luau_text.push_str(" / "),
        }
        self.write_expression_in((
            operation.right_operand(),
            LuauTextWriterExpressionPosition::NumericOperationRightOperand,
        ));
        if needs_parentheses {
            self.luau_text.push(')');
        }
    }

    fn write_call_arguments(&mut self, call_arguments: &[LuauExpression]) {
        let (first_argument, remaining_arguments) = match call_arguments.split_first() {
            Some(argument_sequence) => argument_sequence,
            None => return,
        };
        self.write_expression_in((
            first_argument,
            LuauTextWriterExpressionPosition::FunctionArgument,
        ));
        for call_argument in remaining_arguments {
            self.luau_text.push_str(", ");
            self.write_expression_in((
                call_argument,
                LuauTextWriterExpressionPosition::FunctionArgument,
            ));
        }
    }

    fn write_value_type(&mut self, luau_value_type: LuauValueType) {
        match luau_value_type {
            LuauValueType::Number => self.luau_text.push_str("number"),
            LuauValueType::String => self.luau_text.push_str("string"),
            LuauValueType::Boolean => self.luau_text.push_str("boolean"),
            LuauValueType::NoReturnedValues => self.luau_text.push_str("()"),
        }
    }
}

#[derive(Clone, Copy)]
enum LuauTextWriterExpressionPosition {
    Unrestricted,
    NumericOperationLeftOperand,
    NumericOperationRightOperand,
    FunctionArgument,
}
