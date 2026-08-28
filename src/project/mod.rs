//! Compiling one or more wasm modules into a strict Luau Roblox project.

mod discovery;
mod layout;
mod manifest;
mod problem;

pub use discovery::{discover_project_request, ProjectDiscoveryProblem};
pub use layout::{ModuleExecutionSide, ProjectModuleIdentity, ProjectModuleRole};
pub use manifest::{ProjectManifest, ProjectManifestProblem};
pub use problem::{ProjectCompilationProblem, ProjectCompilationRejection};

use crate::translate::{GeneratedLuauText, MainInvocation, TranslateOptions, TranslateOutcome};
use crate::wasm::{decode_module, DecodeOutcome};
use atomic_write_file::AtomicWriteFile;
use std::ffi::OsStr;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The outcome of compiling one project.
#[derive(Debug)]
pub enum ProjectCompilationOutcome {
    /// Every module compiled into a strict Luau artifact with a unique destination.
    Compiled(CompiledProject),
    /// Compilation stopped before accepting any project artifact.
    Rejected(ProjectCompilationRejection),
}

/// Couples wasm bytes with the target identity and initialization contract.
#[derive(Debug)]
pub struct ProjectModuleSource {
    module_identity: ProjectModuleIdentity,
    module_role: ProjectModuleRole,
    wasm_bytes: Vec<u8>,
}

/// Keeps caller-owned wasm inputs immutable after the project compiler takes ownership.
impl ProjectModuleSource {
    /// @why Keeps the module's location and initialization contract inseparable.
    #[must_use]
    pub fn from_wasm_parts(
        wasm_parts: (ProjectModuleIdentity, ProjectModuleRole, Vec<u8>),
    ) -> Self {
        let (module_identity, module_role, wasm_bytes) = wasm_parts;
        Self {
            module_identity,
            module_role,
            wasm_bytes,
        }
    }

    /// @why Lets validation and layout read the module identity.
    #[must_use]
    pub const fn module_identity(&self) -> &ProjectModuleIdentity {
        &self.module_identity
    }

    /// @why Lets compilation pick entrypoint behavior.
    #[must_use]
    pub const fn module_role(&self) -> ProjectModuleRole {
        self.module_role
    }

    /// @why Lets the compiler hand the module bytes to the decoder.
    #[must_use]
    pub fn wasm_bytes(&self) -> &[u8] {
        &self.wasm_bytes
    }
}

/// Defines every wasm module that must be compiled into one Roblox project layout.
#[derive(Debug)]
pub struct ProjectCompilationRequest {
    source_modules: Vec<ProjectModuleSource>,
}

/// Preserves the caller's complete source set so ordering cannot change the emitted layout.
impl ProjectCompilationRequest {
    /// @why Requires all source modules up front so identity conflicts and missing
    /// entrypoints reject the project before any artifact is accepted.
    #[must_use]
    pub const fn from_source_modules(source_modules: Vec<ProjectModuleSource>) -> Self {
        Self { source_modules }
    }

    /// @why Lets the compiler consume the source set after validation.
    #[must_use]
    pub fn into_source_modules(self) -> Vec<ProjectModuleSource> {
        self.source_modules
    }
}

/// The destination path of one generated artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectOutputPath {
    /// A slash-separated Roblox service path such as
    /// `ServerScriptService/game.server.luau`.
    path_text: String,
}

impl ProjectOutputPath {
    /// @why Lets materialization write the artifact under its compiler-owned destination.
    #[must_use]
    pub fn path_text(&self) -> &str {
        &self.path_text
    }
}

/// One strict Luau artifact attached to its Roblox destination.
#[derive(Debug)]
pub struct GeneratedProjectModule {
    module_identity: ProjectModuleIdentity,
    output_path: ProjectOutputPath,
    artifact: GeneratedLuauText,
}

impl GeneratedProjectModule {
    /// @why Gives diagnostics and build tools the source identity that produced this artifact.
    #[must_use]
    pub const fn module_identity(&self) -> &ProjectModuleIdentity {
        &self.module_identity
    }

    /// @why Lets materialization write the artifact under its compiler-owned destination.
    #[must_use]
    pub const fn output_path(&self) -> &ProjectOutputPath {
        &self.output_path
    }

    /// @why Lets callers write or execute the strict emitted text.
    #[must_use]
    pub const fn artifact(&self) -> &GeneratedLuauText {
        &self.artifact
    }
}

/// The complete set of generated project artifacts.
#[derive(Debug)]
pub struct CompiledProject {
    generated_modules: Vec<GeneratedProjectModule>,
}

impl CompiledProject {
    /// @why Gives materialization the full artifact set in deterministic order.
    #[must_use]
    pub fn generated_modules(&self) -> &[GeneratedProjectModule] {
        &self.generated_modules
    }

    /// Writes every artifact beneath the given project root directory.
    ///
    /// # Errors
    ///
    /// Returns the first I/O error encountered while staging or publishing an
    /// artifact. The previous output directory remains in place until the
    /// complete staged tree is ready to replace it.
    pub fn write_to_directory(&self, project_root: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
        let staging_directory = create_staging_directory(project_root)?;
        match self.write_to_staging_directory(&staging_directory) {
            Ok(()) => {}
            Err(write_error) => {
                remove_staging_directory(&staging_directory);
                return Err(write_error);
            }
        }
        match publish_staging_directory(&staging_directory, project_root) {
            Ok(()) => {}
            Err(publish_error) => return Err(publish_error),
        }
        let mut written_paths = Vec::with_capacity(self.generated_modules.len());
        for generated_module in &self.generated_modules {
            written_paths.push(generated_output_path(project_root, generated_module)?);
        }
        Ok(written_paths)
    }

    fn write_to_staging_directory(&self, staging_directory: &Path) -> Result<(), std::io::Error> {
        for generated_module in &self.generated_modules {
            let output_path = generated_output_path(staging_directory, generated_module)?;
            if let Some(parent_directory) = output_path.parent() {
                fs_err::create_dir_all(parent_directory)?;
            }
            let mut atomic_file = AtomicWriteFile::open(&output_path)?;
            match atomic_file.write_all(generated_module.artifact().as_text().as_bytes()) {
                Ok(()) => {}
                Err(write_error) => {
                    return match atomic_file.discard() {
                        Ok(()) => Err(write_error),
                        Err(discard_error) => Err(std::io::Error::new(
                            write_error.kind(),
                            format!(
                                "could not write {}: {write_error}; could not discard staged file: {discard_error}",
                                output_path.display()
                            ),
                        )),
                    };
                }
            }
            atomic_file.commit()?;
        }
        Ok(())
    }
}

const MAX_STAGING_DIRECTORY_ATTEMPTS: usize = 16;
const INITIAL_STAGING_DIRECTORY_ATTEMPT: usize = 0;
const FALLBACK_TIMESTAMP_NANOS: u128 = 0;

fn create_staging_directory(project_root: &Path) -> Result<PathBuf, std::io::Error> {
    if let Some(parent_directory) = project_root.parent() {
        if !parent_directory.as_os_str().is_empty() {
            fs_err::create_dir_all(parent_directory)?;
        }
    }
    let mut attempt = INITIAL_STAGING_DIRECTORY_ATTEMPT;
    loop {
        let staging_directory = output_sibling_path(project_root, "staging", attempt);
        match fs_err::create_dir(&staging_directory) {
            Ok(()) => return Ok(staging_directory),
            Err(error) => match error.kind() {
                std::io::ErrorKind::AlreadyExists => {
                    attempt += 1;
                    if attempt >= MAX_STAGING_DIRECTORY_ATTEMPTS {
                        return Err(std::io::Error::new(
                            error.kind(),
                            format!(
                                "could not create a unique staging directory after {attempt} attempts: {error}"
                            ),
                        ));
                    }
                }
                _ => return Err(error),
            },
        }
    }
}

fn generated_output_path(
    project_root: &Path,
    generated_module: &GeneratedProjectModule,
) -> Result<PathBuf, std::io::Error> {
    let relative_path = Path::new(generated_module.output_path().path_text());
    for component in relative_path.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "generated output path is not relative and normalized: {}",
                        relative_path.display()
                    ),
                ));
            }
        }
    }
    Ok(project_root.join(relative_path))
}

fn publish_staging_directory(
    staging_directory: &Path,
    project_root: &Path,
) -> Result<(), std::io::Error> {
    match fs_err::symlink_metadata(project_root) {
        Ok(metadata) if metadata.is_dir() => {
            let backup_directory = match move_output_to_backup(project_root) {
                Ok(path) => path,
                Err(error) => {
                    remove_staging_directory(staging_directory);
                    return Err(error);
                }
            };
            match fs_err::rename(staging_directory, project_root) {
                Ok(()) => {
                    remove_staging_directory(&backup_directory);
                    Ok(())
                }
                Err(publish_error) => match fs_err::rename(&backup_directory, project_root) {
                    Ok(()) => {
                        remove_staging_directory(staging_directory);
                        Err(publish_error)
                    }
                    Err(restore_error) => Err(std::io::Error::other(
                        format!(
                            "could not publish {}: {publish_error}; could not restore previous output {}: {restore_error}",
                            project_root.display(),
                            project_root.display()
                        ),
                    )),
                },
            }
        }
        Ok(_) => {
            remove_staging_directory(staging_directory);
            Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!(
                    "project output {} is not a directory",
                    project_root.display()
                ),
            ))
        }
        Err(error) => {
            if matches!(error.kind(), std::io::ErrorKind::NotFound) {
                match fs_err::rename(staging_directory, project_root) {
                    Ok(()) => Ok(()),
                    Err(rename_error) => {
                        remove_staging_directory(staging_directory);
                        Err(std::io::Error::new(
                            rename_error.kind(),
                            format!(
                                "could not publish staged output {}: {rename_error}",
                                project_root.display()
                            ),
                        ))
                    }
                }
            } else {
                remove_staging_directory(staging_directory);
                Err(error)
            }
        }
    }
}

fn remove_staging_directory(directory: &Path) {
    match fs_err::remove_dir_all(directory) {
        Ok(()) => {}
        Err(error) => tracing::warn!(
            path = %directory.display(),
            error = %error,
            "could not remove temporary project directory"
        ),
    }
}

fn output_sibling_path(project_root: &Path, purpose: &str, attempt: usize) -> PathBuf {
    let parent_directory = match project_root.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        Some(_) | None => Path::new("."),
    };
    let project_name = match project_root.file_name().and_then(OsStr::to_str) {
        Some(project_name) if !project_name.is_empty() => project_name,
        Some(_) | None => "output",
    };
    let timestamp_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(FALLBACK_TIMESTAMP_NANOS, |duration| duration.as_nanos());
    parent_directory.join(format!(
        ".{project_name}.luau-rs-{purpose}-{}-{timestamp_nanos}-{attempt}",
        std::process::id()
    ))
}

fn move_output_to_backup(project_root: &Path) -> Result<PathBuf, std::io::Error> {
    let mut attempt = INITIAL_STAGING_DIRECTORY_ATTEMPT;
    loop {
        let backup_directory = output_sibling_path(project_root, "backup", attempt);
        match fs_err::rename(project_root, &backup_directory) {
            Ok(()) => return Ok(backup_directory),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                attempt += 1;
                if attempt >= MAX_STAGING_DIRECTORY_ATTEMPTS {
                    return Err(std::io::Error::new(
                        error.kind(),
                        format!(
                            "could not create a unique backup directory after {attempt} attempts: {error}"
                        ),
                    ));
                }
            }
            Err(error) => return Err(error),
        }
    }
}

/// Compiles a complete project after validating every module before any output is accepted.
///
/// # Errors
///
/// Returns a typed rejection for the first validation, decode, or translation
/// failure instead of accepting a partial project.
#[must_use]
pub fn compile_project(request: ProjectCompilationRequest) -> ProjectCompilationOutcome {
    let mut source_modules = request.into_source_modules();
    source_modules.sort_by(|left, right| left.module_identity().cmp(right.module_identity()));

    let mut previous_identity: Option<ProjectModuleIdentity> = None;
    let mut has_entrypoint = false;
    for source_module in &source_modules {
        let module_identity = source_module.module_identity();
        match &previous_identity {
            Some(previous) if previous == module_identity => {
                return ProjectCompilationOutcome::Rejected(
                    ProjectCompilationProblem::DuplicateModuleIdentity(module_identity.clone())
                        .into(),
                );
            }
            Some(_) | None => {}
        }
        match source_module.module_role() {
            ProjectModuleRole::Entrypoint => has_entrypoint = true,
            ProjectModuleRole::Library => {}
        }
        if source_module.module_role() == ProjectModuleRole::Entrypoint
            && source_module
                .module_identity()
                .output_path_text(source_module.module_role())
                .is_none()
        {
            return ProjectCompilationOutcome::Rejected(
                ProjectCompilationProblem::SharedEntrypoint(module_identity.clone()).into(),
            );
        }
        previous_identity = Some(module_identity.clone());
    }
    if !has_entrypoint {
        return ProjectCompilationOutcome::Rejected(
            ProjectCompilationProblem::MissingEntrypointModule.into(),
        );
    }

    let mut generated_modules = Vec::new();
    for source_module in source_modules {
        let module_identity = source_module.module_identity().clone();
        let module_role = source_module.module_role();
        let Some(output_path_text) = module_identity.output_path_text(module_role) else {
            return ProjectCompilationOutcome::Rejected(
                ProjectCompilationProblem::SharedEntrypoint(module_identity).into(),
            );
        };
        let artifact = match translate_wasm_module(source_module.wasm_bytes(), module_role) {
            Ok(artifact) => artifact,
            Err(problem) => {
                return ProjectCompilationOutcome::Rejected(problem);
            }
        };
        generated_modules.push(GeneratedProjectModule {
            module_identity,
            output_path: ProjectOutputPath {
                path_text: output_path_text,
            },
            artifact,
        });
    }

    ProjectCompilationOutcome::Compiled(CompiledProject { generated_modules })
}

fn translate_wasm_module(
    wasm_bytes: &[u8],
    module_role: ProjectModuleRole,
) -> Result<GeneratedLuauText, ProjectCompilationRejection> {
    let decoded = match decode_module(wasm_bytes) {
        DecodeOutcome::Decoded(decoded) => decoded,
        DecodeOutcome::Rejected(rejection) => {
            return Err(ProjectCompilationProblem::DecodeFailed(format!("{rejection:?}")).into());
        }
    };
    let options = match module_role {
        ProjectModuleRole::Entrypoint => {
            TranslateOptions::with_main_invocation(MainInvocation::InvokeMain)
        }
        ProjectModuleRole::Library => {
            TranslateOptions::with_main_invocation(MainInvocation::LeaveIdle)
        }
    };
    match crate::translate::translate_module(&decoded, options) {
        TranslateOutcome::Translated(artifact) => Ok(artifact),
        TranslateOutcome::Rejected(rejection) => {
            Err(ProjectCompilationProblem::TranslateFailed(format!("{rejection:?}")).into())
        }
    }
}
