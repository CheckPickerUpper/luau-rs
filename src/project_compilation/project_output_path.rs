/// Names one deterministic location inside the generated Roblox project tree.
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd)]
pub struct ProjectOutputPath {
    path_text: String,
}

/// Prevents target placement from being reconstructed differently by separate project consumers.
impl ProjectOutputPath {
    pub(crate) const fn from_path_text(path_text: String) -> Self {
        Self { path_text }
    }

    /// @why Lets build tools materialize the exact compiler-selected Roblox location without recalculating naming conventions.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.path_text
    }
}
