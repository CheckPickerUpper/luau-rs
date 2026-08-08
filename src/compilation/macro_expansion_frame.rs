use crate::SourceRange;

/// Identifies one declarative macro boundary in a compiler diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MacroExpansionFrame {
    macro_name: String,
    definition_module: Option<String>,
    definition_range: SourceRange,
    call_site_module: Option<String>,
    call_site_range: SourceRange,
}

impl MacroExpansionFrame {
    pub(crate) fn from_expansion(
        expansion: (
            String,
            Option<String>,
            SourceRange,
            Option<String>,
            SourceRange,
        ),
    ) -> Self {
        let (macro_name, definition_module, definition_range, call_site_module, call_site_range) =
            expansion;
        Self {
            macro_name,
            definition_module,
            definition_range,
            call_site_module,
            call_site_range,
        }
    }

    /// Returns the macro name that introduced this expansion frame.
    #[must_use]
    pub fn macro_name(&self) -> &str {
        &self.macro_name
    }

    /// Returns the module containing the macro definition when compiling a project.
    #[must_use]
    pub fn definition_module(&self) -> Option<&str> {
        self.definition_module.as_deref()
    }

    /// Returns the source range of the macro definition.
    #[must_use]
    pub const fn definition_range(&self) -> SourceRange {
        self.definition_range
    }

    /// Returns the module containing the invocation when compiling a project.
    #[must_use]
    pub fn call_site_module(&self) -> Option<&str> {
        self.call_site_module.as_deref()
    }

    /// Returns the source range of the macro invocation.
    #[must_use]
    pub const fn call_site_range(&self) -> SourceRange {
        self.call_site_range
    }
}
