/// Classifies a compiler rejection independently from its source location.
#[derive(Debug, PartialEq, Eq)]
pub enum CompilationProblemReason {
    /// A declarative macro definition or its single matcher has invalid shape.
    MacroDefinitionInvalid,
    /// A macro invocation names no definition in the active compilation catalog.
    UnknownMacro,
    /// A macro invocation supplies the wrong token-tree shape for its matcher.
    MacroArgumentShapeMismatch,
    /// Multiple definitions would make a macro lookup ambiguous.
    MacroMatcherAmbiguous,
    /// Expansion nesting exceeded the compiler's fixed safety bound.
    MacroExpansionDepthExceeded,
    /// Expansion output exceeded the compiler's fixed safety bound.
    MacroExpansionOutputLimitExceeded,
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
    /// A type name does not identify a record declared in this source file.
    UnknownRecordType,
    /// A record declaration or literal repeats a field name.
    DuplicateRecordField,
    /// A record literal provides a field outside its declared shape.
    UnknownRecordField,
    /// A record literal omits a field required by its declared shape.
    MissingRecordField,
    /// A record field initializer does not have the field's declared value type.
    RecordFieldInitializerTypeMismatch,
    /// A postfix read names no field declared by its record base type.
    UnknownRecordAccessField,
    /// A postfix field read uses a value that is not a named record.
    FieldAccessRequiresRecord,
    /// A public function exposes a record alias whose visibility ends at this source file.
    FilePrivateRecordTypeCannotBePublic,
    /// An assignment targets a binding whose declaration did not permit updates.
    ImmutableBindingCannotBeAssigned,
    /// A source module uses project-only imports outside the project compiler.
    ProjectImportRequiresProjectCompilation,
    /// An intrinsic names no service in the closed compiler catalog.
    UnknownRobloxService,
    /// A catalogued service was acquired from a module that cannot access it.
    RobloxServiceUnavailableOnModuleExecutionSide,
    /// Service acquisition is only meaningful when a project supplies an execution side.
    RobloxServiceAcquisitionRequiresProjectCompilation,
    /// Service types only describe locals initialized by the service intrinsic.
    RobloxServiceTypeMayOnlyBeUsedForLocalAcquisition,
    /// An intrinsic names no class in the closed Roblox Instance catalog.
    UnknownRobloxInstance,
    /// A typed Instance member is not present in the class catalog.
    UnknownRobloxInstanceMember,
    /// A source acquisition asks the compiler to construct an engine-supplied Instance.
    RobloxInstanceCannotBeConstructed,
    /// Remote operations require project compilation to determine their execution side.
    RobloxRemoteRequiresProjectCompilation,
    /// A shared module cannot select a direction-specific remote operation.
    RobloxRemoteRequiresConcreteExecutionSide,
    /// A remote operation is unavailable on the module's execution side.
    RobloxRemoteWrongExecutionSide,
    /// The operation requires a catalogued `RemoteEvent`.
    RobloxRemoteOperationRequiresRemoteEvent,
    /// The operation requires a catalogued `RemoteFunction`.
    RobloxRemoteOperationRequiresRemoteFunction,
    /// A remote payload contains a value outside the safe wire-data subset.
    RobloxPayloadTypeNotAllowed,
    /// A disconnect operation requires an `RBXScriptConnection` value.
    RobloxConnectionExpected,
}
use crate::ArgumentCount;
