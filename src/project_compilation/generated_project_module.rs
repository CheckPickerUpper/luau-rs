use crate::{GeneratedLuauText, ProjectModuleIdentity, ProjectOutputPath};

/// Owns one strict Luau artifact and the Roblox location that receives it.
#[derive(Debug)]
pub struct GeneratedProjectModule {
    module_identity: ProjectModuleIdentity,
    output_path: ProjectOutputPath,
    generated_luau_text: GeneratedLuauText,
}

/// Keeps generated text attached to its destination, preventing callers from writing an artifact under a different module identity.
impl GeneratedProjectModule {
    pub(crate) fn from_generated_parts(
        generated_parts: (ProjectModuleIdentity, ProjectOutputPath, GeneratedLuauText),
    ) -> Self {
        let (module_identity, output_path, generated_luau_text) = generated_parts;
        Self {
            module_identity,
            output_path,
            generated_luau_text,
        }
    }

    /// @why Gives diagnostics and build tools the source identity that produced this artifact.
    #[must_use]
    pub const fn module_identity(&self) -> &ProjectModuleIdentity {
        &self.module_identity
    }

    /// @why Gives materialization code the compiler-owned destination rather than making callers repeat Roblox layout rules.
    #[must_use]
    pub const fn output_path(&self) -> &ProjectOutputPath {
        &self.output_path
    }

    /// @why Lets callers validate or write strict emitted text while preserving its identity and destination metadata.
    #[must_use]
    pub const fn generated_luau_text(&self) -> &GeneratedLuauText {
        &self.generated_luau_text
    }
}
