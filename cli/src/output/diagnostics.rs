//! Rendering diagnostics the way a compiler should: one message, a caret under
//! what caused it, notes, then help.

use std::fmt::Write as _;

use platipus_compiler::diagnostics::{Diagnostic, DiagnosticBag, SourceFile, Span};

pub fn render(file: &SourceFile, bag: &DiagnosticBag) -> String {
    let mut out = String::new();
    for diagnostic in bag.sorted() {
        match diagnostic {
            Diagnostic::Error(error) => {
                let _ = writeln!(out, "error: {}", error.label());
                if let Some(span) = error.span {
                    let _ = writeln!(out, "  --> {}", file.describe(span));
                    let _ = writeln!(out, "   |");
                    let _ = writeln!(out, "{}", indent(&file.caret_line(span), 3));
                }
                for note in &error.notes {
                    let _ = writeln!(out, "   = note: {}", note.message);
                }
                if let Some(help) = &error.help {
                    let _ = writeln!(out, "   = help: {help}");
                }
            }
            Diagnostic::Warning(warning) => {
                let _ = writeln!(out, "warning: {}", warning.label());
                if let Some(span) = warning.span {
                    let _ = writeln!(out, "  --> {}", file.describe(span));
                }
                if let Some(help) = &warning.help {
                    let _ = writeln!(out, "   = help: {help}");
                }
            }
        }
        out.push('\n');
    }
    out
}

fn indent(text: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    text.lines()
        .map(|line| format!("{pad}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn span_of(diagnostic: &Diagnostic) -> Option<Span> {
    match diagnostic {
        Diagnostic::Error(error) => error.span,
        Diagnostic::Warning(warning) => warning.span,
    }
}
