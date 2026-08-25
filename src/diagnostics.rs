//! Structured source diagnostics for rejected WebAssembly modules.

use gimli::{Dwarf, EndianSlice, LittleEndian, Reader};
use serde::Serialize;
use std::borrow::Cow;
use walrus::Module;
use wasmparser::{Parser, Payload};

/// A source location attached to a compiler diagnostic.
///
/// The fields are stable and optional because optimized or stripped wasm can
/// legitimately omit source information. The wasm offset and function name
/// remain available as the deterministic fallback in that case.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SourceLocation {
    /// The Rust source file from DWARF, when present.
    pub file: Option<String>,
    /// One-based Rust source line, when present.
    pub line: Option<u64>,
    /// One-based source column; zero means the line's left edge.
    pub column: Option<u64>,
    /// The wasm function name or name-section fallback.
    pub function: Option<String>,
    /// The code-section instruction offset, when known.
    pub wasm_offset: Option<usize>,
    /// Whether a source file and line were recovered from DWARF.
    pub source_available: bool,
    /// Actionable guidance when source information is absent.
    pub hint: Option<String>,
}

/// One rejection with a stable machine-readable code and location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Diagnostic {
    /// Stable identifier for the rejection category.
    pub code: String,
    /// Human-readable explanation of the rejected construct.
    pub message: String,
    /// Source and wasm fallback location.
    pub location: SourceLocation,
}

/// A stable collection of diagnostics for one rejected module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DiagnosticReport {
    /// Diagnostics in deterministic rejection order, with exact duplicates removed.
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticReport {
    /// Creates a report from diagnostics collected during one compiler stage.
    #[must_use]
    pub const fn new(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }

    /// Returns the structured diagnostics without exposing report internals.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Creates a report for a stage that has no wasm metadata context.
    #[must_use]
    pub fn without_locations(messages: impl IntoIterator<Item = (String, String)>) -> Self {
        let diagnostics = messages
            .into_iter()
            .map(|(code, message)| Diagnostic {
                code,
                message,
                location: SourceLocation {
                    file: None,
                    line: None,
                    column: None,
                    function: None,
                    wasm_offset: None,
                    source_available: false,
                    hint: Some(
                        "rebuild the Rust crate with debug information to recover a source line"
                            .into(),
                    ),
                },
            })
            .collect();
        Self::new(diagnostics)
    }
}

/// Resolves a wasm function/offset into the best source location available.
pub(crate) fn source_location(
    raw_wasm: &[u8],
    function: Option<String>,
    wasm_offset: Option<usize>,
) -> SourceLocation {
    let source = wasm_offset.and_then(|offset| lookup_line(raw_wasm, offset as u64));
    match source {
        Some((file, line, column)) => SourceLocation {
            file: Some(file),
            line: Some(line),
            column: Some(column),
            function,
            wasm_offset,
            source_available: true,
            hint: None,
        },
        None => SourceLocation {
            file: None,
            line: None,
            column: None,
            function,
            wasm_offset,
            source_available: false,
            hint: Some(
                "source locations were unavailable; rebuild the Rust crate with debug information"
                    .into(),
            ),
        },
    }
}

fn lookup_line(raw_wasm: &[u8], address: u64) -> Option<(String, u64, u64)> {
    let mut sections = std::collections::HashMap::new();
    for payload in Parser::new(0).parse_all(raw_wasm) {
        let Payload::CustomSection(section) = payload.ok()? else {
            continue;
        };
        if section.name().starts_with(".debug") {
            sections.insert(section.name().to_owned(), section.data());
        }
    }
    let dwarf = Dwarf::load(|section| {
        Ok::<_, gimli::Error>(EndianSlice::new(
            sections.get(section.name()).copied().unwrap_or_default(),
            LittleEndian,
        ))
    })
    .ok()?;
    let mut units = dwarf.units();
    while let Some(header) = units.next().ok()? {
        let unit = dwarf.unit(header).ok()?;
        let Some(line_program) = unit.line_program.clone() else {
            continue;
        };
        let (program, sequences) = line_program.sequences().ok()?;
        for sequence in sequences {
            if address < sequence.start || address > sequence.end {
                continue;
            }
            let mut rows = program.resume_from(&sequence);
            let mut best = None;
            while let Some((header, row)) = rows.next_row().ok()? {
                if row.end_sequence() {
                    continue;
                }
                if row.address() > address {
                    break;
                }
                let Some(file) = row.file(header) else {
                    continue;
                };
                let file_name = dwarf
                    .attr_string(&unit, file.path_name())
                    .ok()
                    .and_then(|reader| reader_text(&reader))?;
                let directory = file
                    .directory(header)
                    .and_then(|directory| dwarf.attr_string(&unit, directory).ok())
                    .and_then(|reader| reader_text(&reader));
                let path = match directory {
                    Some(directory) if !directory.is_empty() && !file_name.starts_with('/') => {
                        format!("{directory}/{file_name}")
                    }
                    _ => file_name,
                };
                let line = row.line()?.get();
                let column = match row.column() {
                    gimli::ColumnType::Column(column) => column.get(),
                    gimli::ColumnType::LeftEdge => 0,
                };
                best = Some((path, line, column));
            }
            if best.is_some() {
                return best;
            }
        }
    }
    None
}

fn reader_text<R>(reader: &R) -> Option<String>
where
    R: Reader,
{
    let bytes: Cow<'_, [u8]> = reader.to_slice().ok()?;
    String::from_utf8(bytes.into_owned()).ok()
}

pub(crate) fn function_context(
    module: &Module,
    preferred: impl Fn(&walrus::Function) -> bool,
) -> Option<(Option<String>, Option<usize>)> {
    module
        .funcs
        .iter()
        .filter(|function| matches!(function.kind, walrus::FunctionKind::Local(_)))
        .find(|function| preferred(function))
        .or_else(|| {
            module
                .funcs
                .iter()
                .find(|function| matches!(function.kind, walrus::FunctionKind::Local(_)))
        })
        .map(|function| {
            let offset = match &function.kind {
                walrus::FunctionKind::Local(local) => local
                    .instruction_mapping
                    .first()
                    .map(|(offset, _location)| *offset),
                walrus::FunctionKind::Import(_) | walrus::FunctionKind::Uninitialized(_) => None,
            };
            (function.name.clone(), offset)
        })
}
