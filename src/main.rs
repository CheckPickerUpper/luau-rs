//! Command-line entry point for the luau-rs wasm-to-Luau compiler.

use clap::{Parser, Subcommand};
use luau_rs::{
    compile_project, discover_project_request, ProjectCompilationOutcome,
    ProjectCompilationProblem, ProjectCompilationRequest, ProjectDiscoveryProblem, ProjectManifest,
    ProjectManifestProblem, ProjectModuleIdentity, ProjectModuleRole, ProjectModuleSource,
};
use std::path::PathBuf;
use thiserror::Error;

/// Compiles wasm modules (built from Rust) into strict Luau for Roblox.
#[derive(Parser, Debug)]
#[command(name = "luau-rs", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Build one wasm module into a Roblox project directory.
    Build {
        /// Path to the wasm module produced by `cargo build --target wasm32-unknown-unknown`.
        wasm_path: PathBuf,

        /// Directory that receives the generated Roblox layout.
        #[arg(long, short)]
        out: PathBuf,

        /// Emit the module as an entrypoint script that runs its `main` export.
        #[arg(long)]
        entrypoint: bool,

        /// Execution side for the generated script: server (default) or client.
        #[arg(long, default_value = "server")]
        side: ExecutionSideArg,

        /// Module path within the chosen service, for example `game/main`.
        #[arg(long, default_value = "main")]
        module_path: String,
    },
    /// Validate a manifest-backed project without writing generated output.
    Check {
        /// Path to the project manifest; defaults to `luau-rs.toml`.
        #[arg(long, default_value = "luau-rs.toml")]
        manifest_path: PathBuf,
    },
    /// Compile a manifest-backed project and publish its output atomically.
    Compile {
        /// Path to the project manifest; defaults to `luau-rs.toml`.
        #[arg(long, default_value = "luau-rs.toml")]
        manifest_path: PathBuf,
    },
}

/// Accepted execution-side spellings for the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecutionSideArg {
    Server,
    Client,
}

impl std::str::FromStr for ExecutionSideArg {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "server" => Ok(Self::Server),
            "client" => Ok(Self::Client),
            other => Err(format!(
                "unknown execution side {other:?}; expected \"server\" or \"client\""
            )),
        }
    }
}

fn main() {
    let env_filter = match tracing_subscriber::EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(filter_error) => {
            tracing::warn!(error = %filter_error, "using default log filter");
            tracing_subscriber::EnvFilter::new("luau_rs=info")
        }
    };
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Build {
            wasm_path,
            out,
            entrypoint,
            side,
            module_path,
        } => {
            let wasm_bytes = match fs_err::read(&wasm_path) {
                Ok(bytes) => bytes,
                Err(error) => {
                    tracing::error!(
                        path = %wasm_path.display(),
                        error = %error,
                        "could not read wasm module"
                    );
                    std::process::exit(1);
                }
            };
            let module_identity = match side {
                ExecutionSideArg::Server => ProjectModuleIdentity::Server { module_path },
                ExecutionSideArg::Client => ProjectModuleIdentity::Client { module_path },
            };
            let module_role = if entrypoint {
                ProjectModuleRole::Entrypoint
            } else {
                ProjectModuleRole::Library
            };
            let source_module =
                ProjectModuleSource::from_wasm_parts((module_identity, module_role, wasm_bytes));
            let request = ProjectCompilationRequest::from_source_modules(vec![source_module]);
            match compile_project(request) {
                ProjectCompilationOutcome::Compiled(compiled_project) => {
                    match compiled_project.write_to_directory(&out) {
                        Ok(written_paths) => {
                            for written_path in written_paths {
                                tracing::info!(
                                    path = %written_path.display(),
                                    "wrote generated Luau"
                                );
                            }
                        }
                        Err(error) => {
                            tracing::error!(error = %error, "could not write output");
                            std::process::exit(1);
                        }
                    }
                }
                ProjectCompilationOutcome::Rejected(rejection) => {
                    tracing::error!(
                        problem = ?rejection.problem(),
                        "project compilation rejected"
                    );
                    std::process::exit(1);
                }
            }
        }
        Command::Check { manifest_path } => {
            match run_manifest_command(&manifest_path, ManifestCommandAction::Check) {
                Ok(()) => tracing::info!(path = %manifest_path.display(), "project check passed"),
                Err(error) => {
                    tracing::error!(error = %error, "project check failed");
                    std::process::exit(1);
                }
            }
        }
        Command::Compile { manifest_path } => {
            match run_manifest_command(&manifest_path, ManifestCommandAction::Compile) {
                Ok(()) => tracing::info!(path = %manifest_path.display(), "project compile passed"),
                Err(error) => {
                    tracing::error!(error = %error, "project compile failed");
                    std::process::exit(1);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ManifestCommandAction {
    Check,
    Compile,
}

#[derive(Debug, Error)]
enum ManifestCommandProblem {
    #[error("manifest failed: {0}")]
    Manifest(#[source] ProjectManifestProblem),
    #[error("project discovery failed: {0}")]
    Discovery(#[source] ProjectDiscoveryProblem),
    #[error("project compilation rejected: {0}")]
    Compilation(#[source] ProjectCompilationProblem),
    #[error("could not publish project output {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

fn run_manifest_command(
    manifest_path: &std::path::Path,
    action: ManifestCommandAction,
) -> Result<(), ManifestCommandProblem> {
    let manifest = match ProjectManifest::load_from_path(manifest_path) {
        Ok(manifest) => manifest,
        Err(error) => return Err(ManifestCommandProblem::Manifest(error)),
    };
    let request = match discover_project_request(&manifest) {
        Ok(request) => request,
        Err(error) => return Err(ManifestCommandProblem::Discovery(error)),
    };
    let compiled_project = match compile_project(request) {
        ProjectCompilationOutcome::Compiled(compiled_project) => compiled_project,
        ProjectCompilationOutcome::Rejected(rejection) => {
            return Err(ManifestCommandProblem::Compilation(
                rejection.problem().clone(),
            ));
        }
    };
    match action {
        ManifestCommandAction::Check => {
            tracing::debug!(
                module_count = compiled_project.generated_modules().len(),
                "validated project modules"
            );
            Ok(())
        }
        ManifestCommandAction::Compile => {
            match compiled_project.write_to_directory(manifest.output_root()) {
                Ok(written_paths) => {
                    for written_path in written_paths {
                        tracing::info!(
                            path = %written_path.display(),
                            "published generated Luau"
                        );
                    }
                    Ok(())
                }
                Err(source) => Err(ManifestCommandProblem::Write {
                    path: manifest.output_root().to_path_buf(),
                    source,
                }),
            }
        }
    }
}
