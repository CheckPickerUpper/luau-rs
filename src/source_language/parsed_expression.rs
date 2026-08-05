use crate::{
    source_language::{
        ParsedArrayLiteral, ParsedArrayRead, ParsedComparisonOperation, ParsedEqualityOperation,
        ParsedFieldRead, ParsedFunctionCall, ParsedLiteral, ParsedLogicalNegation,
        ParsedLogicalOperation, ParsedNumericOperation, ParsedRecordLiteral, SourceBooleanLiteral,
    },
    SourceRange,
};

/// Represents only grammar forms valid where the source expects a value.
pub enum ParsedExpression {
    /// Refers to a local value and retains its name location for resolution failures.
    NameReference {
        referenced_name: String,
        name_range: SourceRange,
    },
    /// Preserves a numeric literal exactly as written.
    NumberLiteral(ParsedLiteral),
    /// Preserves a lexer-validated quoted string exactly as written.
    StringLiteral(ParsedLiteral),
    /// Preserves a tokenizer-classified boolean literal and its source location.
    BooleanLiteral {
        boolean_literal: SourceBooleanLiteral,
        literal_range: SourceRange,
    },
    /// Acquires one catalogued Roblox service through the source language's sole intrinsic.
    RobloxServiceAcquisition {
        service_type_name: String,
        service_type_range: SourceRange,
        expression_range: SourceRange,
    },
    /// Constructs a non-empty homogeneous array.
    ArrayLiteral(ParsedArrayLiteral),
    /// Constructs a named record with a checked set of field initializers.
    RecordLiteral(ParsedRecordLiteral),
    /// Reads a declared field after any primary expression.
    FieldRead(ParsedFieldRead),
    /// Reads one zero-based array element.
    ArrayRead(ParsedArrayRead),
    /// Retains a numeric operation and its operator location for type failures.
    NumericOperation(ParsedNumericOperation),
    /// Retains a numeric comparison and its operator location for type failures.
    ComparisonOperation(ParsedComparisonOperation),
    /// Retains an equality operation and its operator location for type failures.
    EqualityOperation(ParsedEqualityOperation),
    /// Retains a boolean negation and its operator location for type failures.
    LogicalNegation(ParsedLogicalNegation),
    /// Retains a short-circuit logical operation and its operator location for type failures.
    LogicalOperation(ParsedLogicalOperation),
    /// Calls a named function and retains the name location for call failures.
    FunctionCall(ParsedFunctionCall),
}

/// Keeps diagnostic attribution available for every value-producing grammar form.
impl ParsedExpression {
    /// Gives type checking the complete expression range responsible for a mismatch.
    pub(crate) const fn source_range(&self) -> SourceRange {
        match self {
            Self::NameReference { name_range, .. } => *name_range,
            Self::NumberLiteral(parsed_literal) | Self::StringLiteral(parsed_literal) => {
                parsed_literal.literal_range()
            }
            Self::BooleanLiteral { literal_range, .. } => *literal_range,
            Self::RobloxServiceAcquisition {
                expression_range, ..
            } => *expression_range,
            Self::ArrayLiteral(array_literal) => array_literal.literal_range(),
            Self::RecordLiteral(record_literal) => record_literal.literal_range(),
            Self::FieldRead(field_read) => field_read.expression_range(),
            Self::ArrayRead(array_read) => array_read.expression_range(),
            Self::NumericOperation(operation) => operation.expression_range(),
            Self::ComparisonOperation(operation) => operation.expression_range(),
            Self::EqualityOperation(operation) => operation.expression_range(),
            Self::LogicalNegation(negation) => negation.expression_range(),
            Self::LogicalOperation(operation) => operation.expression_range(),
            Self::FunctionCall(function_call) => function_call.source_range(),
        }
    }
}
