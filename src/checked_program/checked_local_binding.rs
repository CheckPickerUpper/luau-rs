use crate::{
    checked_program::{roblox_service::RobloxService, CheckedExpression, CheckedValueType},
    CompilationProblem, CompilationProblemReason, SourceRange,
};

/// Retains the assignment contract of one checked local binding.
pub struct CheckedLocalBinding {
    local_name: String,
    binding_kind: CheckedLocalBindingKind,
}

enum CheckedLocalBindingKind {
    Immutable(CheckedValueType),
    Mutable(CheckedValueType),
    AcquiredService(RobloxService),
}

/// Names whether assignment is representable for a checked local.
pub enum LocalAssignmentContract {
    /// The binding cannot receive another value after construction.
    Forbidden,
    /// The binding accepts values of the carried checked type.
    Allowed(CheckedValueType),
}

/// Exposes binding facts without permitting a copied service handle to be represented.
impl CheckedLocalBinding {
    pub(super) fn from_parameter(
        parameter_parts: (String, CheckedValueType, SourceRange),
    ) -> Result<Self, CompilationProblem> {
        let (local_name, value_type, parameter_range) = parameter_parts;
        if matches!(value_type, CheckedValueType::RobloxService(_)) {
            return Err(CompilationProblem::from_problem_at_range((
                parameter_range,
                CompilationProblemReason::RobloxServiceTypeMayOnlyBeUsedForLocalAcquisition,
            )));
        }
        Ok(Self {
            local_name,
            binding_kind: CheckedLocalBindingKind::Immutable(value_type),
        })
    }

    pub(super) fn from_immutable_declaration(
        declaration_parts: (String, CheckedValueType, &CheckedExpression, SourceRange),
    ) -> Result<Self, CompilationProblem> {
        let (local_name, value_type, initial_value, initial_value_range) = declaration_parts;
        let binding_kind = match (value_type, initial_value) {
            (
                CheckedValueType::RobloxService(expected_service),
                CheckedExpression::RobloxServiceAcquisition(actual_service),
            ) if expected_service == *actual_service => {
                CheckedLocalBindingKind::AcquiredService(expected_service)
            }
            (CheckedValueType::RobloxService(_), _) => {
                return Err(CompilationProblem::from_problem_at_range((
                    initial_value_range,
                    CompilationProblemReason::RobloxServiceTypeMayOnlyBeUsedForLocalAcquisition,
                )));
            }
            (ordinary_type, _) => CheckedLocalBindingKind::Immutable(ordinary_type),
        };
        Ok(Self {
            local_name,
            binding_kind,
        })
    }

    pub(super) fn from_mutable_declaration(
        declaration_parts: (String, CheckedValueType, SourceRange),
    ) -> Result<Self, CompilationProblem> {
        let (local_name, value_type, declaration_range) = declaration_parts;
        if matches!(value_type, CheckedValueType::RobloxService(_)) {
            return Err(CompilationProblem::from_problem_at_range((
                declaration_range,
                CompilationProblemReason::RobloxServiceTypeMayOnlyBeUsedForLocalAcquisition,
            )));
        }
        Ok(Self {
            local_name,
            binding_kind: CheckedLocalBindingKind::Mutable(value_type),
        })
    }

    /// Provides the source name used for resolution and collision checks.
    pub(super) fn local_name(&self) -> &str {
        &self.local_name
    }

    /// Provides the type required by references.
    pub(super) fn value_type(&self) -> CheckedValueType {
        match &self.binding_kind {
            CheckedLocalBindingKind::Immutable(value_type)
            | CheckedLocalBindingKind::Mutable(value_type) => value_type.clone(),
            CheckedLocalBindingKind::AcquiredService(service) => {
                CheckedValueType::RobloxService(*service)
            }
        }
    }

    pub(super) fn assignment_contract(&self) -> LocalAssignmentContract {
        match &self.binding_kind {
            CheckedLocalBindingKind::Mutable(value_type) => {
                LocalAssignmentContract::Allowed(value_type.clone())
            }
            CheckedLocalBindingKind::Immutable(_) | CheckedLocalBindingKind::AcquiredService(_) => {
                LocalAssignmentContract::Forbidden
            }
        }
    }
}
