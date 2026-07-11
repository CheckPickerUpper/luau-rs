use crate::{
    checked_program::{
        CheckedComparisonOperation, CheckedComparisonOperator, CheckedEqualityOperation,
        CheckedEqualityOperator, CheckedExpression, CheckedLogicalNegation,
        CheckedLogicalOperation, CheckedLogicalOperator, CheckedValueType,
    },
    source_language::{
        ParsedComparisonOperation, ParsedComparisonOperator, ParsedEqualityOperation,
        ParsedEqualityOperator, ParsedLogicalNegation, ParsedLogicalOperation,
        ParsedLogicalOperator,
    },
    CompilationProblem, CompilationProblemReason, SourceRange,
};

use super::check_expression::ExpressionChecker;

/// Validates compound expressions whose types depend on their operand relationships.
impl<'context, 'program> ExpressionChecker<'context, 'program> {
    /// Checks a numeric comparison and gives it the boolean type.
    pub(super) fn check_comparison_operation(
        &mut self,
        operation: &ParsedComparisonOperation,
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
            ParsedComparisonOperator::LessThan => CheckedComparisonOperator::LessThan,
            ParsedComparisonOperator::LessThanOrEqual => CheckedComparisonOperator::LessThanOrEqual,
            ParsedComparisonOperator::GreaterThan => CheckedComparisonOperator::GreaterThan,
            ParsedComparisonOperator::GreaterThanOrEqual => {
                CheckedComparisonOperator::GreaterThanOrEqual
            }
        };
        Ok((
            CheckedExpression::ComparisonOperation(CheckedComparisonOperation::from_parts((
                Box::new(checked_left),
                Box::new(checked_right),
                checked_operator,
            ))),
            CheckedValueType::Boolean,
        ))
    }

    /// Checks equality between two values with the same returned value type.
    pub(super) fn check_equality_operation(
        &mut self,
        operation: &ParsedEqualityOperation,
    ) -> Result<(CheckedExpression, CheckedValueType), CompilationProblem> {
        let (checked_left, left_type) = match self.check_expression(operation.left_operand()) {
            Ok(checked_operand) => checked_operand,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        let (checked_right, right_type) = match self.check_expression(operation.right_operand()) {
            Ok(checked_operand) => checked_operand,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match Self::require_matching_equality_operands((
            left_type,
            right_type,
            operation.operator_range(),
        )) {
            Ok(()) => {}
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let checked_operator = match operation.operator() {
            ParsedEqualityOperator::Equal => CheckedEqualityOperator::Equal,
            ParsedEqualityOperator::NotEqual => CheckedEqualityOperator::NotEqual,
        };
        Ok((
            CheckedExpression::EqualityOperation(CheckedEqualityOperation::from_parts((
                Box::new(checked_left),
                Box::new(checked_right),
                checked_operator,
            ))),
            CheckedValueType::Boolean,
        ))
    }

    /// Checks a boolean negation and gives it the boolean type.
    pub(super) fn check_logical_negation(
        &mut self,
        negation: &ParsedLogicalNegation,
    ) -> Result<(CheckedExpression, CheckedValueType), CompilationProblem> {
        let (checked_operand, operand_type) =
            match self.check_expression(negation.negated_expression()) {
                Ok(checked_operand) => checked_operand,
                Err(compilation_problem) => return Err(compilation_problem),
            };
        match Self::require_matching_type((
            operand_type,
            CheckedValueType::Boolean,
            negation.operator_range(),
        )) {
            Ok(()) => {}
            Err(compilation_problem) => return Err(compilation_problem),
        }
        Ok((
            CheckedExpression::LogicalNegation(CheckedLogicalNegation::from_expression(Box::new(
                checked_operand,
            ))),
            CheckedValueType::Boolean,
        ))
    }

    /// Checks a short-circuit boolean operation and gives it the boolean type.
    pub(super) fn check_logical_operation(
        &mut self,
        operation: &ParsedLogicalOperation,
    ) -> Result<(CheckedExpression, CheckedValueType), CompilationProblem> {
        let (checked_left, left_type) = match self.check_expression(operation.left_operand()) {
            Ok(checked_operand) => checked_operand,
            Err(compilation_problem) => return Err(compilation_problem),
        };
        match Self::require_matching_type((
            left_type,
            CheckedValueType::Boolean,
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
            CheckedValueType::Boolean,
            operation.operator_range(),
        )) {
            Ok(()) => {}
            Err(compilation_problem) => return Err(compilation_problem),
        }
        let checked_operator = match operation.operator() {
            ParsedLogicalOperator::Conjunction => CheckedLogicalOperator::Conjunction,
            ParsedLogicalOperator::Disjunction => CheckedLogicalOperator::Disjunction,
        };
        Ok((
            CheckedExpression::LogicalOperation(CheckedLogicalOperation::from_parts((
                Box::new(checked_left),
                Box::new(checked_right),
                checked_operator,
            ))),
            CheckedValueType::Boolean,
        ))
    }

    fn require_matching_equality_operands(
        value_types_at_range: (CheckedValueType, CheckedValueType, SourceRange),
    ) -> Result<(), CompilationProblem> {
        let (left_type, right_type, operator_range) = value_types_at_range;
        match (left_type, right_type) {
            (CheckedValueType::Number, CheckedValueType::Number)
            | (CheckedValueType::String, CheckedValueType::String)
            | (CheckedValueType::Boolean, CheckedValueType::Boolean) => Ok(()),
            (CheckedValueType::NoReturnedValues, CheckedValueType::NoReturnedValues)
            | (CheckedValueType::Number, CheckedValueType::String)
            | (CheckedValueType::Number, CheckedValueType::Boolean)
            | (CheckedValueType::Number, CheckedValueType::NoReturnedValues)
            | (CheckedValueType::String, CheckedValueType::Number)
            | (CheckedValueType::String, CheckedValueType::Boolean)
            | (CheckedValueType::String, CheckedValueType::NoReturnedValues)
            | (CheckedValueType::Boolean, CheckedValueType::Number)
            | (CheckedValueType::Boolean, CheckedValueType::String)
            | (CheckedValueType::Boolean, CheckedValueType::NoReturnedValues)
            | (CheckedValueType::NoReturnedValues, CheckedValueType::Number)
            | (CheckedValueType::NoReturnedValues, CheckedValueType::String)
            | (CheckedValueType::NoReturnedValues, CheckedValueType::Boolean) => {
                Err(CompilationProblem::from_problem_at_range((
                    operator_range,
                    CompilationProblemReason::TypesDoNotMatch,
                )))
            }
        }
    }
}
