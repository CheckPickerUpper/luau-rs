/// Classifies a compiler rejection independently from its source location.
#[derive(Debug, PartialEq, Eq)]
pub enum CompilationProblemReason {
    /// The source contains a character outside the accepted language.
    UnsupportedCharacter(char),
    /// The token stream does not form valid source syntax.
    SourceDoesNotFollowLanguageRules,
    /// An expression refers to a name that is not in scope.
    UnknownName,
    /// A function call refers to a declaration that appears later in the source file.
    NameUsedBeforeDeclaration,
    /// A declaration reuses a name already visible in the same program scope.
    NameAlreadyDefined,
    /// A source declaration uses a word that cannot form a Luau declaration name.
    NameNotAllowedInLuau,
    /// A source declaration attempts to replace a compiler-provided builtin name.
    NameReservedForBuiltInFunction,
    /// A value-returning function reaches the end of its body without returning a value.
    MissingReturn,
    /// A source program reaches its end without declaring the required entrypoint function.
    MissingEntrypoint,
    /// A function call supplies a different number of arguments than its signature requires.
    WrongArgumentCount {
        /// Records the count required by the resolved function signature.
        expected: ArgumentCount,
        /// Records the count supplied by the source call.
        actual: ArgumentCount,
    },
    /// A checked expression does not satisfy its required type.
    TypesDoNotMatch,
}
use crate::ArgumentCount;
