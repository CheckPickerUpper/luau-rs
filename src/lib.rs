//! Compiler pipeline for translating a small Rust-like source language into Luau.

mod checked_program;
mod compilation;
mod generated_luau;
mod project_compilation;
mod remote_payload_shape;
mod source_language;

pub use compilation::{
    compile_library_source, compile_source, ArgumentCount, CompilationOutcome, CompilationProblem,
    CompilationProblemReason, CompilationRejection, SourceRange,
};
pub use generated_luau::GeneratedLuauText;
pub use project_compilation::{
    compile_project, CompiledProject, GeneratedProjectModule, ModuleExecutionSide,
    ProjectCompilationOutcome, ProjectCompilationProblem, ProjectCompilationRejection,
    ProjectCompilationRequest, ProjectModuleIdentity, ProjectModuleRole, ProjectModuleSource,
    ProjectOutputPath, RemoteExecutionSide,
};
