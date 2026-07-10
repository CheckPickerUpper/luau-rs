use crate::{source_language::ParsedFunctionCall, SourceRange};

/// Represents only grammar forms valid where the source expects a value.
pub(crate) enum ParsedExpression {
    /// Refers to a local value and retains its name location for resolution failures.
    NameReference {
        referenced_name: String,
        name_range: SourceRange,
    },
    /// Preserves a numeric literal exactly as written.
    NumberLiteral {
        number_literal: String,
        literal_range: SourceRange,
    },
    /// Adds two values and retains the operator location for type failures.
    Addition {
        left_operand: Box<ParsedExpression>,
        right_operand: Box<ParsedExpression>,
        operator_range: SourceRange,
        addition_range: SourceRange,
    },
    /// Calls a named function and retains the name location for call failures.
    FunctionCall(ParsedFunctionCall),
}

/// Keeps diagnostic attribution available for every value-producing grammar form.
impl ParsedExpression {
    /// Gives type checking the complete expression range responsible for a mismatch.
    pub(crate) fn source_range(&self) -> SourceRange {
        match self {
            Self::NameReference { name_range, .. } => *name_range,
            Self::NumberLiteral { literal_range, .. } => *literal_range,
            Self::Addition { addition_range, .. } => *addition_range,
            Self::FunctionCall(function_call) => function_call.source_range(),
        }
    }
}
