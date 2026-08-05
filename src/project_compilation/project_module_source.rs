use crate::{ProjectModuleIdentity, ProjectModuleRole};

/// Couples source text with the target identity and initialization contract it must satisfy.
#[derive(Debug)]
pub struct ProjectModuleSource {
    module_identity: ProjectModuleIdentity,
    module_role: ProjectModuleRole,
    source_text: String,
}

/// Keeps caller-owned source inputs immutable after the project compiler takes ownership.
impl ProjectModuleSource {
    /// @why Keeps the source's location and initialization contract inseparable, preventing a module from being emitted into the wrong Roblox realm.
    #[must_use]
    pub fn from_source_parts(
        source_parts: (ProjectModuleIdentity, ProjectModuleRole, String),
    ) -> Self {
        let (module_identity, module_role, source_text) = source_parts;
        Self {
            module_identity,
            module_role,
            source_text,
        }
    }

    pub(crate) const fn module_identity(&self) -> &ProjectModuleIdentity {
        &self.module_identity
    }

    pub(crate) const fn module_role(&self) -> ProjectModuleRole {
        self.module_role
    }

    pub(crate) fn source_text(&self) -> &str {
        &self.source_text
    }
}
