use crate::json_string::append_json_string;
use crate::{CompilationProblem, SourceRange};

/// Identifies one 1-based position in the original source text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiagnosticPosition {
    line: usize,
    column: usize,
    byte: usize,
}

impl DiagnosticPosition {
    /// Gives the 1-based source line.
    #[must_use]
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Gives the 1-based UTF-8 character column.
    #[must_use]
    pub const fn column(&self) -> usize {
        self.column
    }

    /// Gives the original zero-based byte offset.
    #[must_use]
    pub const fn byte(&self) -> usize {
        self.byte
    }
}

/// Preserves the exact half-open source span alongside human-readable coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiagnosticSpan {
    start: DiagnosticPosition,
    end: DiagnosticPosition,
}

impl DiagnosticSpan {
    /// Gives the first position in the diagnostic span.
    #[must_use]
    pub const fn start(&self) -> DiagnosticPosition {
        self.start
    }

    /// Gives the exclusive position immediately after the diagnostic span.
    #[must_use]
    pub const fn end(&self) -> DiagnosticPosition {
        self.end
    }
}

/// Provides one stable diagnostic record for text output, JSON output, and editor consumers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilationDiagnostic {
    file_name: String,
    reason_code: &'static str,
    span: DiagnosticSpan,
}

impl CompilationDiagnostic {
    pub(crate) fn from_parts(diagnostic_parts: (&str, &str, SourceRange, &'static str)) -> Self {
        let (file_name, source_text, source_range, reason_code) = diagnostic_parts;
        Self {
            file_name: file_name.to_owned(),
            reason_code,
            span: diagnostic_span((source_text, source_range)),
        }
    }

    /// Gives the caller-provided source file name.
    #[must_use]
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Gives the stable machine-readable diagnostic code.
    #[must_use]
    pub const fn reason_code(&self) -> &'static str {
        self.reason_code
    }

    /// Gives the byte and line/column span for editor highlighting.
    #[must_use]
    pub const fn span(&self) -> DiagnosticSpan {
        self.span
    }

    /// Formats the diagnostic as stable `file:line:column-line:column: code` text.
    #[must_use]
    pub fn to_text(&self) -> String {
        let start = self.span.start();
        let end = self.span.end();
        format!(
            "{}:{}:{}-{}:{}: {}",
            self.file_name,
            start.line(),
            start.column(),
            end.line(),
            end.column(),
            self.reason_code,
        )
    }

    /// Formats the diagnostic using a stable, dependency-free JSON schema.
    ///
    /// The schema is `{file, code, span: {start: {byte, line, column}, end: {byte, line,
    /// column}}}`.
    #[must_use]
    pub fn to_json(&self) -> String {
        let start = self.span.start();
        let end = self.span.end();
        let mut json = String::new();
        json.push_str("{\"file\":\"");
        append_json_string(&mut json, &self.file_name);
        json.push_str("\",\"code\":\"");
        append_json_string(&mut json, self.reason_code);
        json.push_str("\",\"span\":{\"start\":{\"byte\":");
        json.push_str(&start.byte().to_string());
        json.push_str(",\"line\":");
        json.push_str(&start.line().to_string());
        json.push_str(",\"column\":");
        json.push_str(&start.column().to_string());
        json.push_str("},\"end\":{\"byte\":");
        json.push_str(&end.byte().to_string());
        json.push_str(",\"line\":");
        json.push_str(&end.line().to_string());
        json.push_str(",\"column\":");
        json.push_str(&end.column().to_string());
        json.push_str("}}}");
        json
    }
}

impl CompilationProblem {
    /// Converts this typed rejection into a file-aware diagnostic without parsing debug text.
    #[must_use]
    pub fn diagnostic(&self, diagnostic_parts: (&str, &str)) -> CompilationDiagnostic {
        let (file_name, source_text) = diagnostic_parts;
        CompilationDiagnostic::from_parts((
            file_name,
            source_text,
            self.source_range(),
            self.reason().code(),
        ))
    }
}

fn diagnostic_span(span_parts: (&str, SourceRange)) -> DiagnosticSpan {
    let (source_text, source_range) = span_parts;
    DiagnosticSpan {
        start: diagnostic_position((source_text, source_range.start_byte())),
        end: diagnostic_position((source_text, source_range.end_byte())),
    }
}

fn diagnostic_position(position_parts: (&str, usize)) -> DiagnosticPosition {
    let (source_text, byte) = position_parts;
    let mut line = 1;
    let mut column = 1;
    for (character_byte, character) in source_text.char_indices() {
        if character_byte >= byte {
            break;
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    DiagnosticPosition { line, column, byte }
}
