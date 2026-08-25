//! Translation of a decoded wasm module into strict Luau text.

mod emitter;
mod function;
mod helpers;
mod ops;
mod problem;
mod writer;

pub use emitter::{MainInvocation, TranslateOptions};
pub use problem::{TranslationProblemReason, TranslationRejection};

use crate::wasm::DecodedModule;

/// The outcome of translating one decoded wasm module.
#[derive(Debug)]
pub enum TranslateOutcome {
    /// The module translated into one strict Luau artifact.
    Translated(GeneratedLuauText),
    /// Translation stopped before any artifact was accepted.
    Rejected(TranslationRejection),
}

/// Owns complete strict Luau emitted only after translation succeeds.
#[derive(Debug, PartialEq, Eq)]
pub struct GeneratedLuauText {
    text: String,
}

impl GeneratedLuauText {
    /// Restricts construction to the translation pipeline.
    pub(crate) const fn from_text(text: String) -> Self {
        Self { text }
    }

    /// @why Transfers the validated artifact so callers can write or execute it.
    #[must_use]
    pub fn into_text(self) -> String {
        self.text
    }

    /// @why Lets project compilation assemble artifacts without copying text.
    #[must_use]
    pub fn as_text(&self) -> &str {
        &self.text
    }
}

/// Translates one decoded wasm module into strict Luau.
///
/// # Errors
///
/// Returns a typed rejection naming the first unsupported instruction instead
/// of emitting a module that would misbehave.
#[must_use]
pub fn translate_module(decoded: &DecodedModule, options: TranslateOptions) -> TranslateOutcome {
    emitter::emit_module(decoded, options).map_or_else(
        |reason| TranslateOutcome::Rejected(TranslationRejection::from(reason)),
        |text| TranslateOutcome::Translated(GeneratedLuauText::from_text(text)),
    )
}
