use crate::checked_program::CheckedValueType;

/// Retains the assignment contract of one checked local binding.
pub enum CheckedLocalBinding {
    /// Prevents assignment after initialization.
    Immutable {
        local_name: String,
        value_type: CheckedValueType,
    },
    /// Permits assignment values of the declared type.
    Mutable {
        local_name: String,
        value_type: CheckedValueType,
    },
}

/// Exposes binding facts without collapsing mutability into a boolean.
impl CheckedLocalBinding {
    /// Provides the source name used for resolution and collision checks.
    pub(super) fn local_name(&self) -> &str {
        match self {
            Self::Immutable { local_name, .. } | Self::Mutable { local_name, .. } => local_name,
        }
    }

    /// Provides the type required by references and assignments.
    pub(super) fn value_type(&self) -> CheckedValueType {
        match self {
            Self::Immutable { value_type, .. } | Self::Mutable { value_type, .. } => {
                value_type.clone()
            }
        }
    }
}
