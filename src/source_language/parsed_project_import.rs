use crate::{ProjectModuleIdentity, SourceRange};

/// Preserves one project-only function import with the locations needed for project diagnostics.
pub struct ParsedProjectImport {
    target_module_identity: ProjectModuleIdentity,
    imported_function_name: String,
    imported_function_range: SourceRange,
    import_range: SourceRange,
}

/// Keeps module addressing separate from function-binding identity until project resolution.
impl ParsedProjectImport {
    pub(crate) fn from_import_parts(
        import_parts: (ProjectModuleIdentity, String, SourceRange, SourceRange),
    ) -> Self {
        let (target_module_identity, imported_function_name, imported_function_range, import_range) =
            import_parts;
        Self {
            target_module_identity,
            imported_function_name,
            imported_function_range,
            import_range,
        }
    }

    pub(crate) const fn target_module_identity(&self) -> &ProjectModuleIdentity {
        &self.target_module_identity
    }

    pub(crate) fn imported_function_name(&self) -> &str {
        &self.imported_function_name
    }

    pub(crate) const fn imported_function_range(&self) -> SourceRange {
        self.imported_function_range
    }

    pub(crate) const fn import_range(&self) -> SourceRange {
        self.import_range
    }
}
