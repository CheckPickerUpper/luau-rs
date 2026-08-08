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
}

/// Gives every source rejection one stable machine-readable code.
impl CompilationProblemReason {
    /// Gives the stable machine-readable code for this rejection reason.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedCharacter(_) => "unsupported_character",
            Self::SourceDoesNotFollowLanguageRules => "source_does_not_follow_language_rules",
            Self::UnknownName => "unknown_name",
            Self::NameUsedBeforeDeclaration => "name_used_before_declaration",
            Self::NameAlreadyDefined => "name_already_defined",
            Self::NameNotAllowedInLuau => "name_not_allowed_in_luau",
            Self::NameReservedForBuiltInFunction => "name_reserved_for_built_in_function",
            Self::MissingReturn => "missing_return",
            Self::MissingEntrypoint => "missing_entrypoint",
            Self::WrongArgumentCount { .. } => "wrong_argument_count",
            Self::TypesDoNotMatch => "types_do_not_match",
            Self::UnknownRecordType => "unknown_record_type",
            Self::DuplicateRecordField => "duplicate_record_field",
            Self::UnknownRecordField => "unknown_record_field",
            Self::MissingRecordField => "missing_record_field",
            Self::RecordFieldInitializerTypeMismatch => "record_field_initializer_type_mismatch",
            Self::UnknownRecordAccessField => "unknown_record_access_field",
            Self::FieldAccessRequiresRecord => "field_access_requires_record",
            Self::FilePrivateRecordTypeCannotBePublic => {
                "file_private_record_type_cannot_be_public"
            }
            Self::ImmutableBindingCannotBeAssigned => "immutable_binding_cannot_be_assigned",
            Self::ProjectImportRequiresProjectCompilation => {
                "project_import_requires_project_compilation"
            }
            Self::UnknownRobloxService => "unknown_roblox_service",
            Self::RobloxServiceUnavailableOnModuleExecutionSide => {
                "roblox_service_unavailable_on_module_execution_side"
            }
            Self::RobloxServiceAcquisitionRequiresProjectCompilation => {
                "roblox_service_acquisition_requires_project_compilation"
            }
            Self::RobloxServiceTypeMayOnlyBeUsedForLocalAcquisition => {
                "roblox_service_type_may_only_be_used_for_local_acquisition"
            }
        }
    }
}
use crate::ArgumentCount;
