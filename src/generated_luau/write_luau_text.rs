use crate::generated_luau::{
    GeneratedLuauText, LuauBooleanLiteral, LuauComparisonOperation, LuauComparisonOperator,
    LuauEqualityOperation, LuauEqualityOperator, LuauExpression, LuauExpressionEmbedding,
    LuauExpressionPrecedence, LuauFunction, LuauFunctionBody, LuauFunctionCall,
    LuauFunctionLiteral, LuauIfElse, LuauLogicalNegation, LuauLogicalOperation,
    LuauLogicalOperator, LuauNumericOperation, LuauNumericOperator, LuauOperationOperandSide,
    LuauParameter, LuauProgram, LuauProgramEnding, LuauRecordAlias, LuauRecordLiteral,
    LuauRobloxRemoteOperation, LuauStatement, LuauValueType, LuauWhileLoop,
};

const STATEMENT_INDENTATION: &str = "    ";

/// Serializes the owned target model into deterministic strict Luau text.
pub fn write_luau_text(luau_program: &LuauProgram) -> GeneratedLuauText {
    let mut luau_text_writer = LuauTextWriter::new();
    luau_text_writer.write_program(luau_program);
    GeneratedLuauText::from_text(luau_text_writer.finish())
}

struct LuauTextWriter {
    luau_text: String,
    current_indentation_level: usize,
}

/// Centralizes layout and precedence decisions for generated Luau.
impl LuauTextWriter {
    const fn new() -> Self {
        Self {
            luau_text: String::new(),
            current_indentation_level: 0,
        }
    }

    fn finish(self) -> String {
        self.luau_text
    }

    fn write_program(&mut self, luau_program: &LuauProgram) {
        self.luau_text.push_str("--!strict\n\n");
        for record_alias in luau_program.record_aliases() {
            self.write_record_alias(record_alias);
            self.luau_text.push('\n');
        }
        for luau_function in luau_program.program_functions() {
            self.write_function(luau_function);
            self.luau_text.push('\n');
        }
        match luau_program.program_ending() {
            LuauProgramEnding::EntrypointCall(entry_function_call) => {
                self.write_expression(entry_function_call);
                self.luau_text.push('\n');
            }
            LuauProgramEnding::NoEntrypointCall => {}
        }
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
            let previous_indentation_level = self.current_indentation_level;
            self.current_indentation_level = indentation_level;
            self.write_statement((luau_statement, indentation_level));
            self.current_indentation_level = previous_indentation_level;
            self.luau_text.push('\n');
        }
    }

    fn write_indentation(&mut self, indentation_level: usize) {
        for _ in 0..indentation_level {
            self.luau_text.push_str(STATEMENT_INDENTATION);
        }
    }

    fn write_parameters(&mut self, luau_parameters: &[LuauParameter]) {
        let Some((first_parameter, remaining_parameters)) = luau_parameters.split_first() else {
            return;
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
                self.write_value_type(value_type.clone());
                self.luau_text.push_str(" = ");
                self.write_expression(initial_value);
            }
            LuauStatement::MutableLocal {
                local_name,
                value_type,
                initial_value,
            } => {
                self.luau_text.push_str("local ");
                self.luau_text.push_str(local_name);
                self.luau_text.push_str(": ");
                self.write_value_type(value_type.clone());
                self.luau_text.push_str(" = ");
                self.write_expression(initial_value);
            }
            LuauStatement::AssignLocal {
                local_name,
                assigned_value,
            } => {
                self.luau_text.push_str(local_name);
                self.luau_text.push_str(" = ");
                self.write_expression(assigned_value);
            }
            LuauStatement::AssignPlace(place_assignment) => {
                self.luau_text
                    .push_str(place_assignment.root_binding_name());
                for step in place_assignment.steps() {
                    match step {
                        crate::generated_luau::LuauPlaceStep::Field(field_name) => {
                            self.luau_text.push('.');
                            self.luau_text.push_str(field_name);
                        }
                        crate::generated_luau::LuauPlaceStep::Index(index_expression) => {
                            self.luau_text.push_str("[(");
                            self.write_expression(index_expression);
                            self.luau_text.push_str(") + 1]");
                        }
                    }
                }
                self.luau_text.push_str(" = ");
                self.write_expression(place_assignment.assigned_value());
            }
            LuauStatement::CallFunctionAndIgnoreResult(function_call) => {
                self.write_function_call(function_call);
            }
            LuauStatement::RobloxRemoteOperation(operation) => {
                self.write_remote_operation(operation);
            }
            LuauStatement::ReturnsValue(returned_expression) => {
                self.luau_text.push_str("return ");
                self.write_expression(returned_expression);
            }
            LuauStatement::BreaksLoop => self.luau_text.push_str("break"),
            LuauStatement::ContinuesLoop => self.luau_text.push_str("continue"),
            LuauStatement::IfElse(luau_if_else) => {
                self.write_if_else((luau_if_else, indentation_level));
            }
            LuauStatement::WhileLoop(luau_while_loop) => {
                self.write_while_loop((luau_while_loop, indentation_level));
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

    fn write_while_loop(&mut self, while_loop_at_indentation: (&LuauWhileLoop, usize)) {
        let (luau_while_loop, indentation_level) = while_loop_at_indentation;
        self.luau_text.push_str("while ");
        self.write_expression(luau_while_loop.condition());
        self.luau_text.push_str(" do\n");
        self.write_function_body((luau_while_loop.body(), indentation_level + 1));
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
        let needs_parentheses = Self::needs_parentheses((luau_expression, expression_embedding));
        if needs_parentheses {
            self.luau_text.push('(');
        }
        match luau_expression {
            LuauExpression::NumericOperation(operation) => self.write_numeric_operation(operation),
            LuauExpression::ComparisonOperation(operation) => {
                self.write_comparison_operation(operation);
            }
            LuauExpression::EqualityOperation(operation) => {
                self.write_equality_operation(operation);
            }
            LuauExpression::LogicalNegation(negation) => self.write_logical_negation(negation),
            LuauExpression::LogicalOperation(operation) => self.write_logical_operation(operation),
            LuauExpression::NameReference(reference_name) => {
                self.luau_text.push_str(reference_name);
            }
            LuauExpression::FunctionReference(function_name) => {
                self.luau_text.push_str(function_name);
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
            LuauExpression::RobloxServiceAcquisition(service_name) => {
                self.luau_text.push_str("game:GetService(\"");
                self.luau_text.push_str(service_name);
                self.luau_text.push_str("\")");
            }
            LuauExpression::RobloxInstanceAcquisition(construction) => {
                match construction.parent_expression() {
                    None => {
                        self.luau_text.push_str("Instance.new(\"");
                        self.luau_text.push_str(construction.instance_name());
                        self.luau_text.push_str("\")");
                    }
                    Some(parent_expression) => {
                        self.luau_text.push_str("(function(__parent: Instance): ");
                        self.luau_text.push_str(construction.instance_name());
                        self.luau_text.push_str(" local __instance: ");
                        self.luau_text.push_str(construction.instance_name());
                        self.luau_text.push_str(" = Instance.new(\"");
                        self.luau_text.push_str(construction.instance_name());
                        self.luau_text
                            .push_str("\"); __instance.Parent = __parent; return __instance end)(");
                        self.write_expression(parent_expression);
                        self.luau_text.push(')');
                    }
                }
            }
            LuauExpression::RobloxInstanceWaitForChild(lookup) => {
                self.write_expression_in((
                    lookup.parent_expression(),
                    LuauExpressionEmbedding::OperationOperand {
                        parent_precedence: LuauExpressionPrecedence::Primary,
                        operand_side: LuauOperationOperandSide::Left,
                    },
                ));
                self.luau_text.push_str(":WaitForChild(");
                self.write_expression(lookup.child_name_expression());
                self.luau_text.push_str(") :: ");
                self.luau_text.push_str(lookup.instance_name());
            }
            LuauExpression::ArrayLiteral(array_literal) => self.write_array_literal(array_literal),
            LuauExpression::RecordLiteral(record_literal) => {
                self.write_record_literal(record_literal);
            }
            LuauExpression::FieldRead(field_read) => {
                match field_read.base_expression() {
                    LuauExpression::RecordLiteral(record_literal) => {
                        self.luau_text.push('(');
                        self.write_record_literal(record_literal);
                        self.luau_text.push(')');
                    }
                    _ => self.write_expression_in((
                        field_read.base_expression(),
                        LuauExpressionEmbedding::OperationOperand {
                            parent_precedence: LuauExpressionPrecedence::Primary,
                            operand_side: LuauOperationOperandSide::Left,
                        },
                    )),
                }
                self.luau_text.push('.');
                self.luau_text.push_str(field_read.field_name());
            }
            LuauExpression::ArrayRead(array_read) => {
                self.write_expression_in((
                    array_read.base_expression(),
                    LuauExpressionEmbedding::OperationOperand {
                        parent_precedence: LuauExpressionPrecedence::Primary,
                        operand_side: LuauOperationOperandSide::Left,
                    },
                ));
                self.luau_text.push_str("[(");
                self.write_expression(array_read.index_expression());
                self.luau_text.push_str(") + 1]");
            }
            LuauExpression::FunctionCall(function_call) => {
                self.write_function_call(function_call);
            }
            LuauExpression::FunctionLiteral(function_literal) => {
                self.write_function_literal(function_literal);
            }
            LuauExpression::RobloxRemoteOperation(operation) => {
                self.write_remote_operation(operation);
            }
        }
        if needs_parentheses {
            self.luau_text.push(')');
        }
    }

    fn write_function_literal(&mut self, function_literal: &LuauFunctionLiteral) {
        self.luau_text.push_str("function(");
        self.write_parameters(function_literal.function_parameters());
        self.luau_text.push_str("): ");
        self.write_value_type(function_literal.returned_value_type());
        self.luau_text.push('\n');
        self.write_function_body((
            function_literal.function_body(),
            self.current_indentation_level + 1,
        ));
        self.write_indentation(self.current_indentation_level);
        self.luau_text.push_str("end");
    }

    fn write_function_call(&mut self, function_call: &LuauFunctionCall) {
        self.luau_text.push_str(function_call.function_name());
        self.luau_text.push('(');
        self.write_call_arguments(function_call.function_arguments());
        self.luau_text.push(')');
    }

    fn write_remote_operation(&mut self, operation: &LuauRobloxRemoteOperation) {
        match operation {
            LuauRobloxRemoteOperation::Connect {
                remote_expression,
                callback_expression,
                execution_side,
            } => {
                self.write_expression_in((
                    remote_expression,
                    LuauExpressionEmbedding::OperationOperand {
                        parent_precedence: LuauExpressionPrecedence::Primary,
                        operand_side: LuauOperationOperandSide::Left,
                    },
                ));
                self.luau_text.push_str(match execution_side {
                    crate::RemoteExecutionSide::Client => ".OnClientEvent:Connect(",
                    crate::RemoteExecutionSide::Server => ".OnServerEvent:Connect(",
                });
                self.write_expression(callback_expression);
                self.luau_text.push(')');
            }
            LuauRobloxRemoteOperation::Disconnect {
                connection_expression,
            } => {
                self.write_expression_in((
                    connection_expression,
                    LuauExpressionEmbedding::OperationOperand {
                        parent_precedence: LuauExpressionPrecedence::Primary,
                        operand_side: LuauOperationOperandSide::Left,
                    },
                ));
                self.luau_text.push_str(":Disconnect()");
            }
            LuauRobloxRemoteOperation::FireServer {
                remote_expression,
                payload_expression,
            } => {
                self.write_expression_in((
                    remote_expression,
                    LuauExpressionEmbedding::OperationOperand {
                        parent_precedence: LuauExpressionPrecedence::Primary,
                        operand_side: LuauOperationOperandSide::Left,
                    },
                ));
                self.luau_text.push_str(":FireServer(");
                self.write_expression(payload_expression);
                self.luau_text.push(')');
            }
            LuauRobloxRemoteOperation::FireClient {
                remote_expression,
                player_expression,
                payload_expression,
            } => {
                self.write_expression_in((
                    remote_expression,
                    LuauExpressionEmbedding::OperationOperand {
                        parent_precedence: LuauExpressionPrecedence::Primary,
                        operand_side: LuauOperationOperandSide::Left,
                    },
                ));
                self.luau_text.push_str(":FireClient(");
                self.write_expression(player_expression);
                self.luau_text.push_str(", ");
                self.write_expression(payload_expression);
                self.luau_text.push(')');
            }
            LuauRobloxRemoteOperation::FireAllClients {
                remote_expression,
                payload_expression,
            } => {
                self.write_expression_in((
                    remote_expression,
                    LuauExpressionEmbedding::OperationOperand {
                        parent_precedence: LuauExpressionPrecedence::Primary,
                        operand_side: LuauOperationOperandSide::Left,
                    },
                ));
                self.luau_text.push_str(":FireAllClients(");
                self.write_expression(payload_expression);
                self.luau_text.push(')');
            }
            LuauRobloxRemoteOperation::InvokeServer {
                remote_expression,
                payload_expression,
            } => {
                self.write_expression_in((
                    remote_expression,
                    LuauExpressionEmbedding::OperationOperand {
                        parent_precedence: LuauExpressionPrecedence::Primary,
                        operand_side: LuauOperationOperandSide::Left,
                    },
                ));
                self.luau_text.push_str(":InvokeServer(");
                self.write_expression(payload_expression);
                self.luau_text.push(')');
            }
            LuauRobloxRemoteOperation::InvokeClient {
                remote_expression,
                player_expression,
                payload_expression,
            } => {
                self.write_expression_in((
                    remote_expression,
                    LuauExpressionEmbedding::OperationOperand {
                        parent_precedence: LuauExpressionPrecedence::Primary,
                        operand_side: LuauOperationOperandSide::Left,
                    },
                ));
                self.luau_text.push_str(":InvokeClient(");
                self.write_expression(player_expression);
                self.luau_text.push_str(", ");
                self.write_expression(payload_expression);
                self.luau_text.push(')');
            }
            LuauRobloxRemoteOperation::SetCallback {
                remote_expression,
                callback_expression,
                execution_side,
            } => {
                self.write_expression_in((
                    remote_expression,
                    LuauExpressionEmbedding::OperationOperand {
                        parent_precedence: LuauExpressionPrecedence::Primary,
                        operand_side: LuauOperationOperandSide::Left,
                    },
                ));
                self.luau_text.push_str(match execution_side {
                    crate::RemoteExecutionSide::Client => ".OnClientInvoke = ",
                    crate::RemoteExecutionSide::Server => ".OnServerInvoke = ",
                });
                self.write_expression(callback_expression);
            }
        }
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

    const fn requires_numeric_precedence_parentheses(
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

    const fn expression_precedence(luau_expression: &LuauExpression) -> LuauExpressionPrecedence {
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
            | LuauExpression::FunctionReference(_)
            | LuauExpression::NumberLiteral(_)
            | LuauExpression::StringLiteral(_)
            | LuauExpression::BooleanLiteral(_)
            | LuauExpression::RobloxServiceAcquisition(_)
            | LuauExpression::RobloxInstanceAcquisition(_)
            | LuauExpression::RobloxInstanceWaitForChild(_)
            | LuauExpression::FunctionCall(_)
            | LuauExpression::RecordLiteral(_)
            | LuauExpression::FieldRead(_)
            | LuauExpression::ArrayLiteral(_)
            | LuauExpression::ArrayRead(_)
            | LuauExpression::FunctionLiteral(_)
            | LuauExpression::RobloxRemoteOperation(_) => LuauExpressionPrecedence::Primary,
        }
    }

    fn write_call_arguments(&mut self, call_arguments: &[LuauExpression]) {
        let Some((first_argument, remaining_arguments)) = call_arguments.split_first() else {
            return;
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
            LuauValueType::Array(element_type) => {
                self.luau_text.push('{');
                self.write_value_type(*element_type);
                self.luau_text.push('}');
            }
            LuauValueType::Function {
                parameter_types,
                returned_value_type,
            } => {
                self.luau_text.push('(');
                self.write_value_types(&parameter_types);
                self.luau_text.push_str(") -> ");
                self.write_value_type(*returned_value_type);
            }
            LuauValueType::NamedRecord(record_name) => self.luau_text.push_str(&record_name),
            LuauValueType::RobloxService(service_name) => self.luau_text.push_str(&service_name),
            LuauValueType::RobloxInstance(instance_name) => self.luau_text.push_str(&instance_name),
            LuauValueType::RobloxConnection => self.luau_text.push_str("RBXScriptConnection"),
            LuauValueType::NoReturnedValues => self.luau_text.push_str("()"),
        }
    }

    fn write_value_types(&mut self, value_types: &[LuauValueType]) {
        let Some((first_value_type, remaining_value_types)) = value_types.split_first() else {
            return;
        };
        self.write_value_type(first_value_type.clone());
        for value_type in remaining_value_types {
            self.luau_text.push_str(", ");
            self.write_value_type(value_type.clone());
        }
    }

    fn write_record_alias(&mut self, record_alias: &LuauRecordAlias) {
        self.luau_text.push_str("type ");
        self.luau_text.push_str(record_alias.record_name());
        self.luau_text.push_str(" = {\n");
        for record_field in record_alias.record_fields() {
            self.luau_text.push_str(STATEMENT_INDENTATION);
            self.luau_text.push_str(record_field.field_name());
            self.luau_text.push_str(": ");
            self.write_value_type(record_field.value_type().clone());
            self.luau_text.push_str(",\n");
        }
        self.luau_text.push_str("}\n");
    }

    fn write_record_literal(&mut self, record_literal: &LuauRecordLiteral) {
        self.luau_text.push('{');
        let Some((first_initializer, remaining_initializers)) =
            record_literal.field_initializers().split_first()
        else {
            self.luau_text.push('}');
            return;
        };
        self.luau_text.push_str(first_initializer.field_name());
        self.luau_text.push_str(" = ");
        self.write_expression(first_initializer.initialized_value());
        for initializer in remaining_initializers {
            self.luau_text.push_str(", ");
            self.luau_text.push_str(initializer.field_name());
            self.luau_text.push_str(" = ");
            self.write_expression(initializer.initialized_value());
        }
        self.luau_text.push('}');
    }

    fn write_array_literal(&mut self, array_literal: &crate::generated_luau::LuauArrayLiteral) {
        self.luau_text.push('{');
        let Some((first_element, remaining_elements)) =
            array_literal.element_expressions().split_first()
        else {
            self.luau_text.push('}');
            return;
        };
        self.write_expression(first_element);
        for element in remaining_elements {
            self.luau_text.push_str(", ");
            self.write_expression(element);
        }
        self.luau_text.push('}');
    }
}
