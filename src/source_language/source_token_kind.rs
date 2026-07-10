use crate::source_language::SourceBooleanLiteral;

/// Names every lexical category accepted by the first source-language slice.
#[derive(Clone)]
pub(crate) enum SourceTokenKind {
    /// Begins a function declaration.
    FunctionKeyword,
    /// Begins an immutable local declaration.
    LetKeyword,
    /// Begins a return statement.
    ReturnKeyword,
    /// Preserves an identifier spelling.
    IdentifierName(String),
    /// Preserves a numeric literal spelling.
    NumberLiteral(String),
    /// Preserves a validated one-line quoted string spelling.
    StringLiteral(String),
    /// Preserves a tokenizer-classified boolean literal.
    BooleanLiteral(SourceBooleanLiteral),
    /// Opens a parameter or argument list.
    LeftParenthesis,
    /// Closes a parameter or argument list.
    RightParenthesis,
    /// Opens a function body.
    LeftBrace,
    /// Closes a function body.
    RightBrace,
    /// Separates a name from its type.
    Colon,
    /// Separates ordered parameters or arguments.
    Comma,
    /// Terminates a source statement.
    Semicolon,
    /// Introduces a function return type.
    Arrow,
    /// Separates a local binding from its initializer.
    Equals,
    /// Adds numeric expressions.
    Plus,
    /// Subtracts numeric expressions.
    Minus,
    /// Multiplies numeric expressions.
    Star,
    /// Divides numeric expressions.
    Slash,
    /// Marks the end of the token stream.
    EndOfSource,
}
