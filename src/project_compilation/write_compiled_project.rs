use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::{
    CompiledProject, GeneratedProjectModule, ProjectOutputOperation, ProjectOutputRejection,
};

/// Publishes one accepted project as a complete replacement of the destination tree.
///
/// # Errors
///
/// Returns a typed rejection when staging, writing, flushing, replacing, or restoring the
/// destination fails. Compilation must already have succeeded before this function is called.
pub fn write_compiled_project_atomically(
    output_parts: (&CompiledProject, impl AsRef<Path>),
) -> Result<(), ProjectOutputRejection> {
    let (compiled_project, output_directory) = output_parts;
    let output_directory = output_directory.as_ref();
    let staging_directory = match staging_path(output_directory) {
        Ok(path) => path,
        Err(rejection) => return Err(rejection),
    };
    if output_directory.exists() && !output_directory.is_dir() {
        return Err(rejection((
            output_directory,
            ProjectOutputOperation::InspectDestination,
            io::ErrorKind::InvalidInput,
        )));
    }
    if let Err(rejection) = recover_interrupted_publication((output_directory, &staging_directory))
    {
        return Err(rejection);
    }
    if staging_directory.exists() {
        return Err(rejection((
            output_directory,
            ProjectOutputOperation::CreateStagingDirectory,
            io::ErrorKind::AlreadyExists,
        )));
    }
    if let Err(error) = fs::create_dir(&staging_directory) {
        return Err(rejection((
            output_directory,
            ProjectOutputOperation::CreateStagingDirectory,
            error.kind(),
        )));
    }
    if let Err(error) = write_staged_modules((compiled_project, &staging_directory)) {
        let _ = fs::remove_dir_all(&staging_directory);
        return Err(error_for_staged_write((output_directory, error)));
    }
    if !output_directory.exists() {
        if let Err(error) = fs::rename(&staging_directory, output_directory) {
            let _ = fs::remove_dir_all(&staging_directory);
            return Err(rejection((
                output_directory,
                ProjectOutputOperation::PublishStagingDirectory,
                error.kind(),
            )));
        }
        return Ok(());
    }
    if let Err(error_kind) = exchange_directories((&staging_directory, output_directory)) {
        let _ = fs::remove_dir_all(&staging_directory);
        return Err(rejection((
            output_directory,
            ProjectOutputOperation::PublishStagingDirectory,
            error_kind,
        )));
    }
    if let Err(error) = fs::remove_dir_all(&staging_directory) {
        return Err(rejection((
            output_directory,
            ProjectOutputOperation::RemovePreviousOutput,
            error.kind(),
        )));
    }
    Ok(())
}

fn recover_interrupted_publication(
    publication_paths: (&Path, &Path),
) -> Result<(), ProjectOutputRejection> {
    let (output_directory, staging_directory) = publication_paths;
    if staging_directory.exists() {
        if let Err(error) = fs::remove_dir_all(staging_directory) {
            return Err(rejection((
                output_directory,
                ProjectOutputOperation::RecoverInterruptedPublication,
                error.kind(),
            )));
        }
    }
    Ok(())
}

fn staging_path(output_directory: &Path) -> Result<PathBuf, ProjectOutputRejection> {
    if output_directory.as_os_str().is_empty() {
        return Err(rejection((
            output_directory,
            ProjectOutputOperation::InspectDestination,
            io::ErrorKind::InvalidInput,
        )));
    }
    let parent_directory = output_directory
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent_directory.is_dir() {
        return Err(rejection((
            output_directory,
            ProjectOutputOperation::InspectDestination,
            io::ErrorKind::NotFound,
        )));
    }
    let Some(directory_name) = output_directory.file_name() else {
        return Err(rejection((
            output_directory,
            ProjectOutputOperation::InspectDestination,
            io::ErrorKind::InvalidInput,
        )));
    };
    let directory_name = directory_name.to_string_lossy();
    Ok(parent_directory.join(format!(".{directory_name}.luau-rs-staging")))
}

#[cfg(target_os = "linux")]
fn exchange_directories(paths: (&Path, &Path)) -> Result<(), io::ErrorKind> {
    let (staging_directory, output_directory) = paths;
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        staging_directory,
        rustix::fs::CWD,
        output_directory,
        rustix::fs::RenameFlags::EXCHANGE,
    )
    .map_err(|error| io::Error::from(error).kind())
}

#[cfg(not(target_os = "linux"))]
const fn exchange_directories(_paths: (&Path, &Path)) -> Result<(), io::ErrorKind> {
    Err(io::ErrorKind::Unsupported)
}

fn write_staged_modules(module_parts: (&CompiledProject, &Path)) -> Result<(), StagedWriteError> {
    let (compiled_project, staging_directory) = module_parts;
    for generated_module in compiled_project.generated_modules() {
        write_staged_module((generated_module, staging_directory))?;
    }
    Ok(())
}

fn write_staged_module(
    module_parts: (&GeneratedProjectModule, &Path),
) -> Result<(), StagedWriteError> {
    let (generated_module, staging_directory) = module_parts;
    let output_path = staging_directory.join(generated_module.output_path().as_str());
    let Some(output_parent) = output_path.parent() else {
        return Err(StagedWriteError::CreateModuleDirectory(
            io::ErrorKind::InvalidInput,
        ));
    };
    fs::create_dir_all(output_parent)
        .map_err(|error| StagedWriteError::CreateModuleDirectory(error.kind()))?;
    let mut output_file =
        File::create(&output_path).map_err(|error| StagedWriteError::WriteModule(error.kind()))?;
    output_file
        .write_all(generated_module.generated_luau_text().as_text().as_bytes())
        .map_err(|error| StagedWriteError::WriteModule(error.kind()))?;
    output_file
        .sync_all()
        .map_err(|error| StagedWriteError::FlushModule(error.kind()))?;
    Ok(())
}

#[derive(Clone, Copy)]
enum StagedWriteError {
    CreateModuleDirectory(io::ErrorKind),
    WriteModule(io::ErrorKind),
    FlushModule(io::ErrorKind),
}

fn error_for_staged_write(output_parts: (&Path, StagedWriteError)) -> ProjectOutputRejection {
    let (output_path, error) = output_parts;
    let (operation, error_kind) = match error {
        StagedWriteError::CreateModuleDirectory(error_kind) => {
            (ProjectOutputOperation::CreateModuleDirectory, error_kind)
        }
        StagedWriteError::WriteModule(error_kind) => {
            (ProjectOutputOperation::WriteModule, error_kind)
        }
        StagedWriteError::FlushModule(error_kind) => {
            (ProjectOutputOperation::FlushModule, error_kind)
        }
    };
    rejection((output_path, operation, error_kind))
}

fn rejection(
    rejection_parts: (&Path, ProjectOutputOperation, io::ErrorKind),
) -> ProjectOutputRejection {
    let (output_path, operation, error_kind) = rejection_parts;
    ProjectOutputRejection::from_parts((output_path.to_owned(), operation, error_kind))
}
