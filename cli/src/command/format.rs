use std::process::ExitCode;

use platipus_compiler::diagnostics::SourceFile;
use platipus_compiler::lexer::{tokenize, Token, TokenKind};

use crate::config::Options;
use crate::output::{diagnostics, path_label, read_entry};
use crate::CliError;

const INDENT: &str = "    ";

/// Rewrites the file with normalised indentation and spacing.
///
/// This is deliberately layout-only. The lexer discards comments, so a
/// formatter that rebuilt the file from tokens would silently delete every
/// comment in it. Instead each token is copied back out of the original bytes
/// using its span, and only the whitespace *between* tokens is replaced. Token
/// text, string contents, and comments therefore survive untouched.
pub fn format_source(source: &str) -> Result<String, Vec<platipus_compiler::diagnostics::Error>> {
    // The scanner strips a byte order mark before it starts, which leaves every
    // span pointing one byte early in a source that still has one. The mark is
    // taken off here so the spans line up, and put back on the way out.
    let (bom, source) = match source.strip_prefix('\u{feff}') {
        Some(stripped) => ("\u{feff}", stripped),
        None => ("", source),
    };
    let tokens = tokenize(source)?;
    let real: Vec<&Token> = tokens
        .iter()
        .filter(|token| !token.is(TokenKind::Eof))
        .collect();
    let mut out = String::with_capacity(source.len());
    let mut indent = 0usize;
    // Whatever precedes the first token, which is only ever a comment.
    let head = &source[..real.first().map(|token| token.span.start as usize).unwrap_or(0)];
    for line in comment_lines(head) {
        pad(&mut out, indent);
        out.push_str(line);
        newline(&mut out);
    }
    for (position, token) in real.iter().enumerate() {
        let previous = position.checked_sub(1).map(|index| real[index]);
        if token.is(TokenKind::RightBrace) {
            // Dedented before the line is written, so `}` lands on the level of
            // the block it closes rather than inside it.
            indent = indent.saturating_sub(1);
        }
        if let Some(previous) = previous {
            let gap = gap_of(source, previous, token);
            emit_gap(&mut out, &gap, previous, token, indent);
        }
        out.push_str(token_text(source, token));
        if token.is(TokenKind::LeftBrace) {
            indent += 1;
        }
    }
    // Anything after the last token can only be whitespace or a comment.
    let tail = &source[real.last().map(|token| token.span.end as usize).unwrap_or(0)..];
    emit_tail(&mut out, tail, indent);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    let mut result = String::with_capacity(out.len() + bom.len());
    result.push_str(bom);
    result.push_str(&out);
    Ok(result)
}

fn token_text<'a>(source: &'a str, token: &Token) -> &'a str {
    &source[token.span.start as usize..token.span.end as usize]
}

fn gap_of(source: &str, previous: &Token, token: &Token) -> String {
    source[previous.span.end as usize..token.span.start as usize].to_string()
}

/// Decides what goes between two tokens.
///
/// A line break the author wrote is kept, because deciding where a program
/// should break from tokens alone means re-deriving structure the parser
/// already knows. What this normalises is horizontal spacing and indentation.
fn emit_gap(out: &mut String, gap: &str, previous: &Token, token: &Token, indent: usize) {
    if gap.contains("//") {
        // A comment that already shares a line with the token before it stays
        // there, and the next token moves down.
        let before = gap.split("//").next().unwrap_or("");
        let trailing = !before.contains('\n');
        if !trailing {
            break_line(out, indent);
        } else {
            out.push(' ');
        }
        out.push_str(&comment_lines(gap).join("\n"));
        break_line(out, indent);
        return;
    }
    if gap.contains("/*") {
        // The compiler accepts block comments, but nothing in the repository
        // uses one, so they are kept verbatim on their own line.
        break_line(out, indent);
        out.push_str(gap.trim());
        break_line(out, indent);
        return;
    }
    if previous.is(TokenKind::LeftBrace)
        && token.is(TokenKind::RightBrace)
        && gap.trim().is_empty()
    {
        out.push(' ');
        return;
    }
    if previous.is(TokenKind::LeftBrace) || token.is(TokenKind::RightBrace) {
        break_line(out, indent);
        return;
    }
    if previous.is(TokenKind::RightBrace) && token.lexeme != "else" {
        break_line(out, indent);
        return;
    }
    let breaks = gap.matches('\n').count();
    if breaks == 0 {
        out.push_str(separator(previous, token));
        return;
    }
    if breaks >= 2 {
        // An author who left a blank line meant it. The indentation is written
        // after the blank line, not before it, or the blank line keeps it.
        newline(out);
        out.push('\n');
        pad(out, indent);
        return;
    }
    break_line(out, indent);
}

fn emit_tail(out: &mut String, tail: &str, indent: usize) {
    for line in comment_lines(tail) {
        break_line(out, indent);
        out.push_str(line);
    }
}

/// Starts a new line, if the output is not already at the start of one.
fn newline(out: &mut String) {
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
}

/// Writes the indentation for a line that is about to receive text.
///
/// Indentation is written here rather than at the end of the previous line so a
/// blank line stays genuinely blank instead of carrying trailing spaces.
fn pad(out: &mut String, indent: usize) {
    if out.is_empty() || out.ends_with('\n') {
        for _ in 0..indent {
            out.push_str(INDENT);
        }
    }
}

/// A line break on its own, with the indentation for whatever follows it.
fn break_line(out: &mut String, indent: usize) {
    newline(out);
    pad(out, indent);
}

/// The comment lines in a chunk of source, trimmed.
///
/// The `//` is kept: dropping the marker would not be a comment any more.
fn comment_lines(text: &str) -> Vec<&str> {
    text.lines().map(str::trim).filter(|line| !line.is_empty()).collect()
}

fn separator(previous: &Token, token: &Token) -> &'static str {
    if previous.is(TokenKind::LeftParen)
        || previous.is(TokenKind::LeftBracket)
        || previous.is(TokenKind::Dot)
        || previous.is(TokenKind::Bang)
    {
        return "";
    }
    if token.is(TokenKind::RightParen)
        || token.is(TokenKind::RightBracket)
        || token.is(TokenKind::Comma)
        || token.is(TokenKind::Colon)
        || token.is(TokenKind::Dot)
        || token.is(TokenKind::LeftParen)
        || token.is(TokenKind::LeftBracket)
    {
        return "";
    }
    " "
}

pub fn run(options: &Options) -> Result<ExitCode, CliError> {
    let source = read_entry(&options.entry)?;
    let label = path_label(&options.entry);
    let formatted = match format_source(&source) {
        Ok(formatted) => formatted,
        Err(errors) => {
            let mut bag = platipus_compiler::diagnostics::DiagnosticBag::new();
            for error in errors {
                bag.error(error);
            }
            let file = SourceFile::new(label, source);
            eprint!("{}", diagnostics::render(&file, &bag));
            return Err(CliError::Io("cannot format a file that does not lex".to_string()));
        }
    };
    if formatted == source {
        if !options.quiet {
            println!("{label} is already formatted");
        }
        return Ok(ExitCode::SUCCESS);
    }
    std::fs::write(&options.entry, &formatted).map_err(|error| {
        CliError::Io(format!("cannot write `{}`: {error}", options.entry.display()))
    })?;
    if !options.quiet {
        println!("formatted {label}");
    }
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format(source: &str) -> String {
        format_source(source).expect("lexes")
    }

    #[test]
    fn comments_survive() {
        let source = "// a leading note\napp Main {\n    // inside\n    Column { }\n}\n";
        let out = format(source);
        assert!(out.contains("// a leading note"), "{out}");
        assert!(out.contains("// inside"), "{out}");
    }

    #[test]
    fn a_trailing_comment_stays_on_its_line() {
        let out = format("app Main { // the root\n    Column { }\n}\n");
        assert!(out.contains("app Main { // the root"), "{out}");
    }

    #[test]
    fn an_empty_block_stays_on_one_line() {
        assert_eq!(format("app Main { Column { } }\n"), "app Main {\n    Column { }\n}\n");
    }

    #[test]
    fn formatting_is_idempotent() {
        let source = "// note\napp Counter {\n    state count = 0\n    derived doubled = count * 2\n\n    Column {\n        Text count\n        Button \"+\" {\n            on click {\n                count += 1\n            }\n        }\n    }\n}\n\ntest t {\n    click \"+\"\n    expect count == 1\n}\n";
        let once = format(source);
        assert_eq!(format(&once), once, "not idempotent");
    }

    #[test]
    fn spacing_is_normalised() {
        let out = format("app Main{state count=0\nColumn{Text count\nButton \"+\"{on click{count+=1}}}}\n");
        assert_eq!(
            out,
            "app Main {\n    state count = 0\n    Column {\n        Text count\n        Button \"+\" {\n            on click {\n                count += 1\n            }\n        }\n    }\n}\n"
        );
    }

    #[test]
    fn else_stays_with_the_brace_it_closes() {
        let out = format("app Main {\n    if count > 0 {\n        Text \"hi\"\n    } else {\n        Text \"bye\"\n    }\n}\n");
        assert!(out.contains("    } else {"), "{out}");
    }

    #[test]
    fn a_blank_line_between_statements_is_kept() {
        let out = format("app Main {\n    state count = 0\n\n\n    state other = 1\n    Column { }\n}\n");
        assert!(out.contains("state count = 0\n\n    state other = 1"), "{out}");
    }

    #[test]
    fn string_contents_are_untouched() {
        let out = format("app Main {\n    Text \"a   b\"\n}\n");
        assert!(out.contains("Text \"a   b\""), "{out}");
    }

    #[test]
    fn a_file_that_does_not_lex_is_reported() {
        assert!(format_source("app Main {\n    Text \"unterminated\n}\n").is_err());
    }

    #[test]
    fn a_byte_order_mark_survives_and_does_not_shift_the_source() {
        // The scanner strips the mark before it starts, so without handling it
        // here every span would point one byte early and drop characters.
        let out = format("\u{feff}app Main {\n    Column { }\n}\n");
        assert_eq!(out, "\u{feff}app Main {\n    Column { }\n}\n");
    }
}
