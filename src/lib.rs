//! Compiler pipeline for translating wasm modules (compiled from Rust) into
//! strict Luau that runs inside Roblox.
//!
//! The pipeline has three stages: [`wasm::decode_module`] validates and
//! decodes a wasm binary, [`translate::translate_module`] emits one strict
//! Luau module per decoded module, and [`project::compile_project`] assembles
//! multiple modules into a Roblox project layout.

pub mod diagnostics;
pub mod project;
pub mod translate;
pub mod wasm;

pub use diagnostics::{Diagnostic, DiagnosticReport, SourceLocation};
pub use project::{
    compile_project, discover_project_request, CompiledProject, GeneratedProjectModule,
    ModuleExecutionSide, ProjectCompilationOutcome, ProjectCompilationProblem,
    ProjectCompilationRejection, ProjectCompilationRequest, ProjectDiscoveryProblem,
    ProjectManifest, ProjectManifestProblem, ProjectModuleIdentity, ProjectModuleRole,
    ProjectModuleSource, ProjectOutputPath, RobloxService,
};
pub use translate::{
    translate_module, GeneratedLuauText, MainInvocation, TranslateOptions, TranslateOutcome,
    TranslationProblemReason, TranslationRejection,
};
pub use wasm::{
    decode_module, DecodeOutcome, DecodedExport, DecodedFunction, DecodedGlobal,
    DecodedGlobalValue, DecodedImport, DecodedMemory, DecodedModule, StartFunctionPresence,
    WasmDecodeProblemReason, WasmDecodeRejection, WasmValueType,
};
