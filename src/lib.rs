//! Compiler pipeline for translating a small Rust-like source language into Luau.

mod checked_program;
mod compilation;
mod generated_luau;
mod source_language;

pub use compilation::{
    compile_source, ArgumentCount, CompilationOutcome, CompilationProblem,
    CompilationProblemReason, CompilationRejection, SourceRange,
};
pub use generated_luau::GeneratedLuauText;
