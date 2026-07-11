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
    /// Begins a two-branch conditional statement.
    IfKeyword,
    /// Introduces the required alternative branch of a conditional statement.
    ElseKeyword,
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
    /// Tests equality between two expressions.
    EqualEqual,
    /// Tests inequality between two expressions.
    BangEqual,
    /// Negates one boolean expression.
    Bang,
    /// Adds numeric expressions.
    Plus,
    /// Subtracts numeric expressions.
    Minus,
    /// Multiplies numeric expressions.
    Star,
    /// Divides numeric expressions.
    Slash,
    /// Tests whether the left number is smaller than the right number.
    LessThan,
    /// Tests whether the left number is no greater than the right number.
    LessThanOrEqual,
    /// Tests whether the left number is greater than the right number.
    GreaterThan,
    /// Tests whether the left number is at least the right number.
    GreaterThanOrEqual,
    /// Conjoins boolean expressions with short-circuit evaluation.
    AmpersandAmpersand,
    /// Disjoins boolean expressions with short-circuit evaluation.
    PipePipe,
    /// Marks the end of the token stream.
    EndOfSource,
}
