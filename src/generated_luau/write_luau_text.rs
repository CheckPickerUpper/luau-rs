use crate::generated_luau::{
    GeneratedLuauText, LuauBooleanLiteral, LuauComparisonOperation, LuauComparisonOperator,
    LuauEqualityOperation, LuauEqualityOperator, LuauExpression, LuauExpressionEmbedding,
    LuauExpressionPrecedence, LuauFunction, LuauFunctionBody, LuauFunctionCall, LuauIfElse,
    LuauLogicalNegation, LuauLogicalOperation, LuauLogicalOperator, LuauNumericOperation,
    LuauNumericOperator, LuauOperationOperandSide, LuauParameter, LuauProgram, LuauStatement,
    LuauValueType,
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

        self.write_function_body((luau_function.function_body(), 1));

        self.luau_text.push_str("end\n");
    }

    fn write_function_body(&mut self, body_at_indentation: (&LuauFunctionBody, usize)) {
        let (luau_function_body, indentation_level) = body_at_indentation;
        for luau_statement in luau_function_body.body_statements() {
            self.write_indentation(indentation_level);
            self.write_statement((luau_statement, indentation_level));
            self.luau_text.push('\n');
        }
    }

    fn write_indentation(&mut self, indentation_level: usize) {
        for _ in 0..indentation_level {
            self.luau_text.push_str(STATEMENT_INDENTATION);
        }
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

    fn write_statement(&mut self, statement_at_indentation: (&LuauStatement, usize)) {
        let (luau_statement, indentation_level) = statement_at_indentation;
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
            LuauStatement::ReturnsValue(returned_expression) => {
                self.luau_text.push_str("return ");
                self.write_expression(returned_expression);
            }
            LuauStatement::IfElse(luau_if_else) => {
                self.write_if_else((luau_if_else, indentation_level));
            }
        }
    }

    fn write_if_else(&mut self, if_else_at_indentation: (&LuauIfElse, usize)) {
        let (luau_if_else, indentation_level) = if_else_at_indentation;
        self.luau_text.push_str("if ");
        self.write_expression(luau_if_else.condition());
        self.luau_text.push_str(" then\n");
        self.write_function_body((luau_if_else.then_body(), indentation_level + 1));
        self.write_indentation(indentation_level);
        self.luau_text.push_str("else\n");
        self.write_function_body((luau_if_else.else_body(), indentation_level + 1));
        self.write_indentation(indentation_level);
        self.luau_text.push_str("end");
    }

    fn write_expression(&mut self, luau_expression: &LuauExpression) {
        self.write_expression_in((luau_expression, LuauExpressionEmbedding::Unrestricted));
    }

    fn write_expression_in(
        &mut self,
        expression_in_position: (&LuauExpression, LuauExpressionEmbedding),
    ) {
        let (luau_expression, expression_embedding) = expression_in_position;
        let needs_parentheses = self.needs_parentheses((luau_expression, expression_embedding));
        if needs_parentheses {
            self.luau_text.push('(');
        }
        match luau_expression {
            LuauExpression::NumericOperation(operation) => self.write_numeric_operation(operation),
            LuauExpression::ComparisonOperation(operation) => {
                self.write_comparison_operation(operation)
            }
            LuauExpression::EqualityOperation(operation) => {
                self.write_equality_operation(operation)
            }
            LuauExpression::LogicalNegation(negation) => self.write_logical_negation(negation),
            LuauExpression::LogicalOperation(operation) => self.write_logical_operation(operation),
            LuauExpression::NameReference(reference_name) => {
                self.luau_text.push_str(reference_name);
            }
            LuauExpression::NumberLiteral(number_literal) => {
                self.luau_text.push_str(number_literal);
            }
            LuauExpression::StringLiteral(string_literal) => {
                self.luau_text.push_str(string_literal);
            }
            LuauExpression::BooleanLiteral(boolean_literal) => match boolean_literal {
                LuauBooleanLiteral::True => self.luau_text.push_str("true"),
                LuauBooleanLiteral::False => self.luau_text.push_str("false"),
            },
            LuauExpression::FunctionCall(function_call) => {
                self.write_function_call(function_call);
            }
        }
        if needs_parentheses {
            self.luau_text.push(')');
        }
    }

    fn write_function_call(&mut self, function_call: &LuauFunctionCall) {
        self.luau_text.push_str(function_call.function_name());
        self.luau_text.push('(');
        self.write_call_arguments(function_call.function_arguments());
        self.luau_text.push(')');
    }

    fn write_numeric_operation(&mut self, operation: &LuauNumericOperation) {
        let operator_spelling = match operation.operator() {
            LuauNumericOperator::Addition => " + ",
            LuauNumericOperator::Subtraction => " - ",
            LuauNumericOperator::Multiplication => " * ",
            LuauNumericOperator::Division => " / ",
        };
        let operation_precedence = match operation.operator() {
            LuauNumericOperator::Addition | LuauNumericOperator::Subtraction => {
                LuauExpressionPrecedence::Additive
            }
            LuauNumericOperator::Multiplication | LuauNumericOperator::Division => {
                LuauExpressionPrecedence::Multiplicative
            }
        };
        self.write_binary_operation((
            operation.left_operand(),
            operation.right_operand(),
            operator_spelling,
            operation_precedence,
        ));
    }

    fn write_comparison_operation(&mut self, operation: &LuauComparisonOperation) {
        let operator_spelling = match operation.operator() {
            LuauComparisonOperator::LessThan => " < ",
            LuauComparisonOperator::LessThanOrEqual => " <= ",
            LuauComparisonOperator::GreaterThan => " > ",
            LuauComparisonOperator::GreaterThanOrEqual => " >= ",
        };
        self.write_binary_operation((
            operation.left_operand(),
            operation.right_operand(),
            operator_spelling,
            LuauExpressionPrecedence::Comparison,
        ));
    }

    fn write_equality_operation(&mut self, operation: &LuauEqualityOperation) {
        let operator_spelling = match operation.operator() {
            LuauEqualityOperator::Equal => " == ",
            LuauEqualityOperator::NotEqual => " ~= ",
        };
        self.write_binary_operation((
            operation.left_operand(),
            operation.right_operand(),
            operator_spelling,
            LuauExpressionPrecedence::Comparison,
        ));
    }

    fn write_logical_negation(&mut self, negation: &LuauLogicalNegation) {
        self.luau_text.push_str("not ");
        self.write_expression_in((
            negation.negated_expression(),
            LuauExpressionEmbedding::OperationOperand {
                parent_precedence: LuauExpressionPrecedence::Negation,
                operand_side: LuauOperationOperandSide::Right,
            },
        ));
    }

    fn write_logical_operation(&mut self, operation: &LuauLogicalOperation) {
        let (operator_spelling, operation_precedence) = match operation.operator() {
            LuauLogicalOperator::Conjunction => (" and ", LuauExpressionPrecedence::Conjunction),
            LuauLogicalOperator::Disjunction => (" or ", LuauExpressionPrecedence::Disjunction),
        };
        self.write_binary_operation((
            operation.left_operand(),
            operation.right_operand(),
            operator_spelling,
            operation_precedence,
        ));
    }

    fn write_binary_operation(
        &mut self,
        binary_operation_parts: (
            &LuauExpression,
            &LuauExpression,
            &str,
            LuauExpressionPrecedence,
        ),
    ) {
        let (left_operand, right_operand, operator_spelling, operation_precedence) =
            binary_operation_parts;
        self.write_expression_in((
            left_operand,
            LuauExpressionEmbedding::OperationOperand {
                parent_precedence: operation_precedence,
                operand_side: LuauOperationOperandSide::Left,
            },
        ));
        self.luau_text.push_str(operator_spelling);
        self.write_expression_in((
            right_operand,
            LuauExpressionEmbedding::OperationOperand {
                parent_precedence: operation_precedence,
                operand_side: LuauOperationOperandSide::Right,
            },
        ));
    }

    fn needs_parentheses(
        &self,
        expression_in_embedding: (&LuauExpression, LuauExpressionEmbedding),
    ) -> bool {
        let (luau_expression, expression_embedding) = expression_in_embedding;
        match expression_embedding {
            LuauExpressionEmbedding::Unrestricted | LuauExpressionEmbedding::FunctionArgument => {
                false
            }
            LuauExpressionEmbedding::OperationOperand {
                parent_precedence,
                operand_side,
            } => {
                let child_precedence = Self::expression_precedence(luau_expression);
                match child_precedence.cmp(&parent_precedence) {
                    std::cmp::Ordering::Less => true,
                    std::cmp::Ordering::Equal => match operand_side {
                        LuauOperationOperandSide::Left => false,
                        LuauOperationOperandSide::Right => true,
                    },
                    std::cmp::Ordering::Greater => Self::requires_numeric_precedence_parentheses((
                        parent_precedence,
                        child_precedence,
                        operand_side,
                    )),
                }
            }
        }
    }

    fn requires_numeric_precedence_parentheses(
        expression_precedence_relationship: (
            LuauExpressionPrecedence,
            LuauExpressionPrecedence,
            LuauOperationOperandSide,
        ),
    ) -> bool {
        let (parent_precedence, child_precedence, operand_side) =
            expression_precedence_relationship;
        match parent_precedence {
            LuauExpressionPrecedence::Additive => match child_precedence {
                LuauExpressionPrecedence::Multiplicative => match operand_side {
                    LuauOperationOperandSide::Left => false,
                    LuauOperationOperandSide::Right => true,
                },
                LuauExpressionPrecedence::Disjunction
                | LuauExpressionPrecedence::Conjunction
                | LuauExpressionPrecedence::Comparison
                | LuauExpressionPrecedence::Additive
                | LuauExpressionPrecedence::Negation
                | LuauExpressionPrecedence::Primary => false,
            },
            LuauExpressionPrecedence::Disjunction
            | LuauExpressionPrecedence::Conjunction
            | LuauExpressionPrecedence::Comparison
            | LuauExpressionPrecedence::Multiplicative
            | LuauExpressionPrecedence::Negation
            | LuauExpressionPrecedence::Primary => false,
        }
    }

    fn expression_precedence(luau_expression: &LuauExpression) -> LuauExpressionPrecedence {
        match luau_expression {
            LuauExpression::LogicalOperation(operation) => match operation.operator() {
                LuauLogicalOperator::Conjunction => LuauExpressionPrecedence::Conjunction,
                LuauLogicalOperator::Disjunction => LuauExpressionPrecedence::Disjunction,
            },
            LuauExpression::ComparisonOperation(_) | LuauExpression::EqualityOperation(_) => {
                LuauExpressionPrecedence::Comparison
            }
            LuauExpression::NumericOperation(operation) => match operation.operator() {
                LuauNumericOperator::Addition | LuauNumericOperator::Subtraction => {
                    LuauExpressionPrecedence::Additive
                }
                LuauNumericOperator::Multiplication | LuauNumericOperator::Division => {
                    LuauExpressionPrecedence::Multiplicative
                }
            },
            LuauExpression::LogicalNegation(_) => LuauExpressionPrecedence::Negation,
            LuauExpression::NameReference(_)
            | LuauExpression::NumberLiteral(_)
            | LuauExpression::StringLiteral(_)
            | LuauExpression::BooleanLiteral(_)
            | LuauExpression::FunctionCall(_) => LuauExpressionPrecedence::Primary,
        }
    }

    fn write_call_arguments(&mut self, call_arguments: &[LuauExpression]) {
        let (first_argument, remaining_arguments) = match call_arguments.split_first() {
            Some(argument_sequence) => argument_sequence,
            None => return,
        };
        self.write_expression_in((first_argument, LuauExpressionEmbedding::FunctionArgument));
        for call_argument in remaining_arguments {
            self.luau_text.push_str(", ");
            self.write_expression_in((call_argument, LuauExpressionEmbedding::FunctionArgument));
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
