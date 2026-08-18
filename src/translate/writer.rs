//! A minimal indentation-aware Luau text builder and literal formatting.

use super::problem::TranslationProblemReason;
use std::fmt::Write as _;

/// A named constant for the wasm page size in bytes (64 KiB).
pub const WASM_PAGE_SIZE_BYTES: u32 = 65_536;
/// Luau tables are one-indexed; wasm indices shift by this amount.
pub const LUAU_INDEX_OFFSET: usize = 1;
/// The stack slot array name inside every generated function.
pub const STACK_NAME: &str = "stack";
/// The stack pointer name inside every generated function.
pub const SP_NAME: &str = "sp";
/// The first printable ASCII byte kept verbatim in escaped byte strings.
const PRINTABLE_ASCII_START: u8 = 0x20;
/// The last printable ASCII byte kept verbatim in escaped byte strings.
const PRINTABLE_ASCII_END: u8 = 0x7E;

/// A minimal indentation-aware Luau text builder.
pub struct TextWriter {
    text: String,
    indentation: usize,
}

impl TextWriter {
    pub const fn new() -> Self {
        Self {
            text: String::new(),
            indentation: 0,
        }
    }

    pub fn finish(self) -> String {
        self.text
    }

    pub fn line(&mut self, content: &str) {
        for _ in 0..self.indentation {
            self.text.push_str("    ");
        }
        self.text.push_str(content);
        self.text.push('\n');
    }

    pub fn raw(&mut self, content: &str) {
        self.text.push_str(content);
    }

    pub const fn push_indent(&mut self) {
        self.indentation += 1;
    }

    pub const fn pop_indent(&mut self) {
        self.indentation = self.indentation.saturating_sub(1);
    }
}

/// Converts a `u32` to `usize` without an unchecked cast.
pub fn usize_from_u32(value: u32) -> Result<usize, TranslationProblemReason> {
    usize::try_from(value).map_err(|error| TranslationProblemReason::Internal(error.to_string()))
}

/// Formats an `f64` as a Luau number literal, naming NaN and infinities.
pub fn luau_number_literal(value: f64) -> String {
    if value.is_nan() {
        return "(0 / 0)".into();
    }
    if value == f64::INFINITY {
        return "math.huge".into();
    }
    if value == f64::NEG_INFINITY {
        return "-math.huge".into();
    }
    value.to_string()
}

/// Formats a string as a Luau string literal.
pub fn luau_string_literal(value: &str) -> String {
    let mut escaped = String::new();
    escaped.push('"');
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}

/// Formats raw bytes as a Luau string literal, escaping every byte verbatim.
pub fn luau_byte_string(bytes: &[u8]) -> String {
    let mut escaped = String::new();
    escaped.push('"');
    for byte in bytes {
        match *byte {
            b'"' => escaped.push_str("\\\""),
            b'\\' => escaped.push_str("\\\\"),
            PRINTABLE_ASCII_START..=PRINTABLE_ASCII_END => escaped.push(char::from(*byte)),
            _ => {
                let _ = write!(escaped, "\\{byte:03}");
            }
        }
    }
    escaped.push('"');
    escaped
}
