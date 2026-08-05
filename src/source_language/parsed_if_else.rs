use crate::{
    source_language::{ParsedExpression, ParsedFunctionBody},
    SourceRange,
};

/// Retains an explicit two-branch decision before its condition or scopes are checked.
pub struct ParsedIfElse {
    condition: ParsedExpression,
    then_body: ParsedFunctionBody,
    else_body: ParsedFunctionBody,
    condition_range: SourceRange,
}

/// Keeps branch structure explicit so later stages can prove total returns and lexical scope.
impl ParsedIfElse {
    /// Joins a condition with its required then and else bodies.
    pub(crate) fn from_parts(
        if_else_parts: (
            ParsedExpression,
            ParsedFunctionBody,
            ParsedFunctionBody,
            SourceRange,
        ),
    ) -> Self {
        let (condition, then_body, else_body, condition_range) = if_else_parts;
        Self {
            condition,
            then_body,
            else_body,
            condition_range,
        }
    }

    /// Gives semantic checking the expression that determines the selected branch.
    pub(crate) const fn condition(&self) -> &ParsedExpression {
        &self.condition
    }

    /// Gives semantic checking the scope entered when the condition is true.
    pub(crate) const fn then_body(&self) -> &ParsedFunctionBody {
        &self.then_body
    }

    /// Gives semantic checking the scope entered when the condition is false.
    pub(crate) const fn else_body(&self) -> &ParsedFunctionBody {
        &self.else_body
    }

    /// Attributes decision-level diagnostics to the condition.
    pub(crate) const fn condition_range(&self) -> SourceRange {
        self.condition_range
    }
}
