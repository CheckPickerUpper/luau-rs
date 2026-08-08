//! Compiler pipeline for translating a small Rust-like source language into Luau.

mod checked_program;
mod compilation;
mod generated_luau;
mod project_compilation;
mod source_language;

pub use compilation::{
    compile_library_source, compile_source, ArgumentCount, CompilationDiagnostic,
    CompilationOutcome, CompilationProblem, CompilationProblemReason, CompilationRejection,
    DiagnosticPosition, DiagnosticSpan, SourceRange,
};
pub use generated_luau::GeneratedLuauText;
pub use project_compilation::{
    compile_project, write_compiled_project_atomically, CompiledProject, GeneratedProjectModule,
    ModuleExecutionSide, ProjectCompilationOutcome, ProjectCompilationProblem,
    ProjectCompilationRejection, ProjectCompilationRequest, ProjectModuleIdentity,
    ProjectModuleRole, ProjectModuleSource, ProjectOutputOperation, ProjectOutputPath,
    ProjectOutputRejection,
};
