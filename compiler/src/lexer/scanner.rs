use super::keyword::Keyword;
use super::literal::{Literal, parse_float_literal, parse_int_literal, unescape};
use super::operator::lookup;
use super::token::{Token, TokenKind};
use crate::diagnostics::{Error, ErrorKind, Span};

#[derive(Debug, Default)]
pub struct ScanResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<Error>,
}

pub struct Scanner<'a> {
    source: &'a str,
    bytes: &'a [u8],
    start: usize,
    current: usize,
    tokens: Vec<Token>,
    errors: Vec<Error>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        let source = strip_bom(source);
        Self {
            source,
            bytes: source.as_bytes(),
            start: 0,
            current: 0,
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn scan(mut self) -> ScanResult {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        let span = Span::new(self.source.len() as u32, self.source.len() as u32);
        self.tokens.push(Token::eof(span));
        ScanResult {
            tokens: self.tokens,
            errors: self.errors,
        }
    }

    fn span(&self) -> Span {
        Span::new(self.start as u32, self.current as u32)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.bytes.len()
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            0
        } else {
            self.bytes[self.current]
        }
    }

    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.bytes.len() {
            0
        } else {
            self.bytes[self.current + 1]
        }
    }

    fn advance(&mut self) -> u8 {
        let byte = self.bytes[self.current];
        self.current += 1;
        byte
    }

    fn matches(&mut self, expected: u8) -> bool {
        if self.is_at_end() || self.bytes[self.current] != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn text(&self) -> &'a str {
        &self.source[self.start..self.current]
    }

    fn char_at(&self, index: usize) -> Option<char> {
        self.source
            .get(index..)
            .and_then(|rest| rest.chars().next())
    }

    fn add(&mut self, kind: TokenKind) {
        let lexeme = self.text().to_string();
        self.tokens.push(Token::new(kind, lexeme, self.span()));
    }

    fn error(&mut self, code: &'static str, message: impl Into<String>) {
        self.errors
            .push(Error::new(ErrorKind::Lex, code, message).with_span(self.span()));
    }

    fn scan_token(&mut self) {
        let byte = self.advance();
        match byte {
            b' ' | b'\t' | b'\r' | b'\n' => {}
            b'/' if self.matches(b'/') => {
                while !self.is_at_end() && self.peek() != b'\n' {
                    self.advance();
                }
            }
            b'/' if self.matches(b'*') => self.scan_block_comment(),
            b'"' => self.scan_string(),
            b'0'..=b'9' => self.scan_number(),
            b if is_identifier_start(b) => self.scan_identifier(),
            _ if !byte.is_ascii() => {
                let first = self.char_at(self.current - 1).unwrap_or('\u{fffd}');
                if is_identifier_start_char(first) {
                    for _ in 1..first.len_utf8() {
                        self.advance();
                    }
                    self.scan_identifier();
                } else {
                    self.scan_symbol(byte);
                }
            }
            _ => self.scan_symbol(byte),
        }
    }

    fn scan_block_comment(&mut self) {
        let open = self.span();
        loop {
            if self.is_at_end() {
                self.errors.push(
                    Error::new(
                        ErrorKind::Lex,
                        "unterminated-comment",
                        "block comment is never closed",
                    )
                    .with_span(open),
                );
                return;
            }
            if self.advance() == b'*' && self.matches(b'/') {
                return;
            }
        }
    }

    fn scan_string(&mut self) {
        let open = self.span();
        let mut value = String::new();
        let mut terminated = false;
        while !self.is_at_end() {
            match self.advance() {
                b'"' => {
                    terminated = true;
                    break;
                }
                b'\n' => break,
                b'\\' => {
                    if self.is_at_end() {
                        break;
                    }
                    let start = self.current;
                    self.advance();
                    if self.peek() == b'{' {
                        while !self.is_at_end() && self.peek() != b'}' {
                            self.advance();
                        }
                        if !self.is_at_end() {
                            self.advance();
                        }
                    }
                    value.push_str(&self.source[start - 1..self.current]);
                }
                _ => {
                    let start = self.current - 1;
                    while !self.is_at_end() && !is_escape(self.peek()) && self.peek() != b'\n' {
                        self.advance();
                    }
                    value.push_str(&self.source[start..self.current]);
                }
            }
        }
        if !terminated {
            self.errors.push(
                Error::new(
                    ErrorKind::Lex,
                    "unterminated-string",
                    "string literal is never closed with `\"`",
                )
                .with_span(open),
            );
            return;
        }
        self.current = self.current.max(self.start + 2);
        let literal_span = self.span();
        match unescape(&value) {
            Ok(text) => self.tokens.push(
                Token::new(TokenKind::StringLiteral, text.clone(), literal_span)
                    .with_literal(Literal::Str(text)),
            ),
            Err(message) => self.errors.push(
                Error::new(ErrorKind::Lex, "invalid-escape", message).with_span(literal_span),
            ),
        }
    }

    fn scan_number(&mut self) {
        while !self.is_at_end() && (self.peek().is_ascii_digit() || self.peek() == b'_') {
            self.advance();
        }
        let mut is_float = false;
        if self.peek() == b'.' && self.peek_next().is_ascii_digit() {
            is_float = true;
            self.advance();
            while !self.is_at_end() && (self.peek().is_ascii_digit() || self.peek() == b'_') {
                self.advance();
            }
        }
        if self.peek() == b'e' || self.peek() == b'E' {
            let save = self.current;
            self.advance();
            if self.peek() == b'+' || self.peek() == b'-' {
                self.advance();
            }
            if self.peek().is_ascii_digit() {
                is_float = true;
                while !self.is_at_end() && self.peek().is_ascii_digit() {
                    self.advance();
                }
            } else {
                self.current = save;
            }
        }
        let digits = self.text().to_string();
        let span = self.span();
        if is_float {
            match parse_float_literal(&digits) {
                Ok(value) => self.tokens.push(
                    Token::new(TokenKind::FloatLiteral, digits, span)
                        .with_literal(Literal::Float(value)),
                ),
                Err(message) => self
                    .errors
                    .push(Error::new(ErrorKind::Lex, "invalid-float", message).with_span(span)),
            }
        } else {
            match parse_int_literal(&digits) {
                Ok(value) => self.tokens.push(
                    Token::new(TokenKind::IntLiteral, digits, span)
                        .with_literal(Literal::Int(value)),
                ),
                Err(message) => self
                    .errors
                    .push(Error::new(ErrorKind::Lex, "invalid-int", message).with_span(span)),
            }
        }
    }

    fn scan_identifier(&mut self) {
        while !self.is_at_end() {
            let Some(ch) = self.char_at(self.current) else {
                break;
            };
            if !is_identifier_continue_char(ch) {
                break;
            }
            for _ in 0..ch.len_utf8() {
                self.advance();
            }
        }
        let text = self.text();
        if let Some(keyword) = Keyword::from_ident(text) {
            let span = self.span();
            let literal = match keyword {
                Keyword::True => Some(Literal::Bool(true)),
                Keyword::False => Some(Literal::Bool(false)),
                Keyword::Null => Some(Literal::Null),
                _ => None,
            };
            let mut token = Token::new(TokenKind::Keyword, keyword.as_str(), span);
            if let Some(literal) = literal {
                token = token.with_literal(literal);
            }
            self.tokens.push(token);
            return;
        }
        self.add(TokenKind::Identifier);
    }

    fn scan_symbol(&mut self, byte: u8) {
        if !byte.is_ascii() {
            self.report_unknown_character();
            return;
        }
        let mut single = String::new();
        single.push(byte as char);
        if is_pair_candidate(byte) {
            let next = self.peek();
            if next.is_ascii() {
                let mut pair = single.clone();
                pair.push(next as char);
                if let Some(info) = lookup(&pair) {
                    let symbol = info.symbol;
                    let kind = info.kind;
                    self.advance();
                    self.tokens
                        .push(Token::new(kind, symbol.to_string(), self.span()));
                    return;
                }
            }
        }
        if let Some(info) = lookup(&single) {
            let symbol = info.symbol;
            let kind = info.kind;
            self.tokens
                .push(Token::new(kind, symbol.to_string(), self.span()));
            return;
        }
        self.error(
            "unexpected-character",
            format!("unexpected character `{}` in source", byte as char),
        );
    }

    fn report_unknown_character(&mut self) {
        let rest = &self.source[self.current - 1..];
        let found = rest.chars().next().unwrap_or('\u{fffd}');
        for _ in 1..found.len_utf8() {
            if self.is_at_end() {
                break;
            }
            self.advance();
        }
        self.error(
            "unexpected-character",
            format!("unexpected character `{found}` in source"),
        );
    }
}

fn is_pair_candidate(byte: u8) -> bool {
    matches!(
        byte,
        b'>' | b'<' | b'!' | b'=' | b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|'
    )
}

pub fn strip_bom(source: &str) -> &str {
    source.strip_prefix('\u{feff}').unwrap_or(source)
}

pub fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

pub fn is_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

pub fn is_identifier_start_char(ch: char) -> bool {
    ch == '_' || ch.is_alphabetic()
}

pub fn is_identifier_continue_char(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric()
}

fn is_escape(byte: u8) -> bool {
    matches!(byte, b'"' | b'\\' | b'\n' | b'\r' | b'\t')
}
