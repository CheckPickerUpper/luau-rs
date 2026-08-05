use crate::{
    source_language::{ParsedExpression, ParsedFunctionBody},
    SourceRange,
};

/// Retains a conditionally repeated body before its expression and scope are checked.
pub struct ParsedWhileLoop {
    condition: ParsedExpression,
    body: ParsedFunctionBody,
    condition_range: SourceRange,
}

/// Keeps loop structure explicit so later stages can enforce boolean conditions and lexical scope.
impl ParsedWhileLoop {
    /// Joins a loop condition with the body it repeatedly evaluates.
    pub(crate) fn from_parts(
        while_loop_parts: (ParsedExpression, ParsedFunctionBody, SourceRange),
    ) -> Self {
        let (condition, body, condition_range) = while_loop_parts;
        Self {
            condition,
            body,
            condition_range,
        }
    }

    /// Gives semantic checking the expression that determines whether another iteration begins.
    pub(crate) const fn condition(&self) -> &ParsedExpression {
        &self.condition
    }

    /// Gives semantic checking the lexical scope evaluated for each iteration.
    pub(crate) const fn body(&self) -> &ParsedFunctionBody {
        &self.body
    }

    /// Attributes loop-level diagnostics to the condition expression.
    pub(crate) const fn condition_range(&self) -> SourceRange {
        self.condition_range
    }
}
