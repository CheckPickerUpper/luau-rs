/// Classifies a compiler rejection independently from its source location.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CompilationProblemReason {
    /// The source contains a character outside the accepted language.
    #[error("unsupported character {0:?}")]
    UnsupportedCharacter(char),
    /// The token stream does not form valid source syntax.
    #[error("source does not follow the language rules")]
    SourceDoesNotFollowLanguageRules,
    /// An expression refers to a name that is not in scope.
    #[error("unknown name")]
    UnknownName,
    /// A function call refers to a declaration that appears later in the source file.
    #[error("name is used before its declaration")]
    NameUsedBeforeDeclaration,
    /// A declaration reuses a name already visible in the same program scope.
    #[error("name is already defined in this scope")]
    NameAlreadyDefined,
    /// A source declaration uses a word that cannot form a Luau declaration name.
    #[error("name is not allowed in Luau")]
    NameNotAllowedInLuau,
    /// A source declaration attempts to replace a compiler-provided builtin name.
    #[error("name is reserved for a built-in function")]
    NameReservedForBuiltInFunction,
    /// A value-returning function reaches the end of its body without returning a value.
    #[error("value-returning function must return on every path")]
    MissingReturn,
    /// A source program reaches its end without declaring the required entrypoint function.
    #[error("source must declare an entrypoint function")]
    MissingEntrypoint,
    /// A function call supplies a different number of arguments than its signature requires.
    #[error("function call has {actual} arguments, but the function requires {expected}")]
    WrongArgumentCount {
        /// Records the count required by the resolved function signature.
        expected: ArgumentCount,
        /// Records the count supplied by the source call.
        actual: ArgumentCount,
    },
    /// A checked expression does not satisfy its required type.
    #[error("expression type does not match the required type")]
    TypesDoNotMatch,
    /// A type name does not identify a record declared in this source file.
    #[error("unknown record type")]
    UnknownRecordType,
    /// A record declaration or literal repeats a field name.
    #[error("record field is declared more than once")]
    DuplicateRecordField,
    /// A record literal provides a field outside its declared shape.
    #[error("record literal contains an unknown field")]
    UnknownRecordField,
    /// A record literal omits a field required by its declared shape.
    #[error("record literal is missing a required field")]
    MissingRecordField,
    /// A record field initializer does not have the field's declared value type.
    #[error("record field initializer has the wrong type")]
    RecordFieldInitializerTypeMismatch,
    /// A postfix read names no field declared by its record base type.
    #[error("record access names an unknown field")]
    UnknownRecordAccessField,
    /// A postfix field read uses a value that is not a named record.
    #[error("field access requires a record value")]
    FieldAccessRequiresRecord,
    /// A public function exposes a record alias whose visibility ends at this source file.
    #[error("public function cannot expose a file-private record type")]
    FilePrivateRecordTypeCannotBePublic,
    /// An assignment targets a binding whose declaration did not permit updates.
    #[error("immutable binding cannot be assigned")]
    ImmutableBindingCannotBeAssigned,
    /// A source module uses project-only imports outside the project compiler.
    #[error("project imports require project compilation")]
    ProjectImportRequiresProjectCompilation,
    /// An intrinsic names no service in the closed compiler catalog.
    #[error("unknown Roblox service")]
    UnknownRobloxService,
    /// A catalogued service was acquired from a module that cannot access it.
    #[error("Roblox service is unavailable on this module's execution side")]
    RobloxServiceUnavailableOnModuleExecutionSide,
    /// Service acquisition is only meaningful when a project supplies an execution side.
    #[error("Roblox service acquisition requires project compilation")]
    RobloxServiceAcquisitionRequiresProjectCompilation,
    /// Service types only describe locals initialized by the service intrinsic.
    #[error("Roblox service types may only be used for local acquisition")]
    RobloxServiceTypeMayOnlyBeUsedForLocalAcquisition,
}
use crate::ArgumentCount;
