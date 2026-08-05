use crate::source_language::SourceBooleanLiteral;

/// Names every lexical category accepted by the first source-language slice.
#[derive(Clone)]
pub enum SourceTokenKind {
    /// Begins a file-private record type declaration.
    StructKeyword,
    /// Begins a function declaration.
    FunctionKeyword,
    /// Makes the following function available to other project modules.
    PublicKeyword,
    /// Begins a project-module function import.
    UseKeyword,
    /// Begins an immutable local declaration.
    LetKeyword,
    /// Marks a local declaration as assignable after initialization.
    MutKeyword,
    /// Begins a return statement.
    ReturnKeyword,
    /// Exits the innermost enclosing loop.
    BreakKeyword,
    /// Skips to the next iteration of the innermost enclosing loop.
    ContinueKeyword,
    /// Begins a two-branch conditional statement.
    IfKeyword,
    /// Introduces the required alternative branch of a conditional statement.
    ElseKeyword,
    /// Begins a conditionally repeated statement body.
    WhileKeyword,
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
    /// Opens an array type, literal, or indexed access.
    LeftBracket,
    /// Closes an array type, literal, or indexed access.
    RightBracket,
    /// Separates a name from its type.
    Colon,
    /// Separates the fixed Roblox intrinsic namespace from its sole operation.
    DoubleColon,
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
    /// Reads a named field from a record value.
    Dot,
    /// Marks the end of the token stream.
    EndOfSource,
}
