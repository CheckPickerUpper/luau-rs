use crate::GeneratedProjectModule;

/// Owns every deterministic Luau artifact accepted for one Roblox project.
#[derive(Debug)]
pub struct CompiledProject {
    generated_modules: Vec<GeneratedProjectModule>,
}

/// Keeps successful project output ordered by source identity for stable build results.
impl CompiledProject {
    pub(crate) const fn from_generated_modules(
        generated_modules: Vec<GeneratedProjectModule>,
    ) -> Self {
        Self { generated_modules }
    }

    /// @why Lets project writers materialize all accepted artifacts in the exact order used for deterministic compiler output.
    #[must_use]
    pub fn generated_modules(&self) -> &[GeneratedProjectModule] {
        &self.generated_modules
    }
}
