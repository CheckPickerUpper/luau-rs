use crate::{
    source_language::{
        ParsedFunctionCall, ParsedLiteral, ParsedNumericOperation, SourceBooleanLiteral,
    },
    SourceRange,
};

/// Represents only grammar forms valid where the source expects a value.
pub(crate) enum ParsedExpression {
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
    /// Retains a numeric operation and its operator location for type failures.
    NumericOperation(ParsedNumericOperation),
    /// Calls a named function and retains the name location for call failures.
    FunctionCall(ParsedFunctionCall),
}

/// Keeps diagnostic attribution available for every value-producing grammar form.
impl ParsedExpression {
    /// Gives type checking the complete expression range responsible for a mismatch.
    pub(crate) fn source_range(&self) -> SourceRange {
        match self {
            Self::NameReference { name_range, .. } => *name_range,
            Self::NumberLiteral(parsed_literal) | Self::StringLiteral(parsed_literal) => {
                parsed_literal.literal_range()
            }
            Self::BooleanLiteral { literal_range, .. } => *literal_range,
            Self::NumericOperation(operation) => operation.expression_range(),
            Self::FunctionCall(function_call) => function_call.source_range(),
        }
    }
}
