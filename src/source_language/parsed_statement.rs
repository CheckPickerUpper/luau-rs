use crate::{
    source_language::{
        ParsedExpression, ParsedFunctionCall, ParsedIfElse, ParsedRobloxRemoteOperation,
        ParsedValueType, ParsedWhileLoop,
    },
    SourceRange,
};

/// Represents only grammar forms valid inside a parsed function body.
pub enum ParsedStatement {
    /// Introduces an immutable typed local value.
    ImmutableLocal {
        local_name: String,
        local_name_range: crate::SourceRange,
        value_type: ParsedValueType,
        initial_value: ParsedExpression,
    },
    /// Introduces a typed local that later assignment may update.
    MutableLocal {
        local_name: String,
        local_name_range: crate::SourceRange,
        value_type: ParsedValueType,
        initial_value: ParsedExpression,
    },
    /// Replaces a mutable local value while retaining the target name location.
    AssignLocal {
        local_name: String,
        local_name_range: crate::SourceRange,
        assigned_value: ParsedExpression,
    },
    /// Replaces a value reached from a mutable identifier through fields or indexes.
    AssignPlace(crate::source_language::ParsedPlaceAssignment),
    /// Invokes a function where the source discards any returned value.
    CallFunctionAndIgnoreResult(ParsedFunctionCall),
    /// Performs an explicit remote operation where the source discards any returned value.
    RobloxRemoteOperation(ParsedRobloxRemoteOperation),
    /// Returns one expression from the enclosing function or branch.
    ReturnsValue(ParsedExpression),
    /// Exits the innermost enclosing loop at this source location.
    BreaksLoop(SourceRange),
    /// Starts the next iteration of the innermost enclosing loop at this source location.
    ContinuesLoop(SourceRange),
    /// Chooses between two independently scoped bodies.
    IfElse(ParsedIfElse),
    /// Repeats an independently scoped body while its boolean condition remains true.
    WhileLoop(ParsedWhileLoop),
}

/// Keeps unreachable-statement diagnostics tied to the first unreachable source construct.
impl ParsedStatement {
    /// Gives control-flow checking the statement location that follows a guaranteed return.
    pub(crate) const fn source_range(&self) -> SourceRange {
        match self {
            Self::ImmutableLocal {
                local_name_range, ..
            }
            | Self::MutableLocal {
                local_name_range, ..
            }
            | Self::AssignLocal {
                local_name_range, ..
            } => *local_name_range,
            Self::AssignPlace(place_assignment) => place_assignment.source_range(),
            Self::CallFunctionAndIgnoreResult(function_call) => function_call.source_range(),
            Self::RobloxRemoteOperation(operation) => operation.expression_range(),
            Self::ReturnsValue(returned_expression) => returned_expression.source_range(),
            Self::BreaksLoop(keyword_range) | Self::ContinuesLoop(keyword_range) => *keyword_range,
            Self::IfElse(parsed_if_else) => parsed_if_else.condition_range(),
            Self::WhileLoop(parsed_while_loop) => parsed_while_loop.condition_range(),
        }
    }
}
