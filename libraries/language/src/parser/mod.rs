pub mod app;
pub mod component;
pub mod element;
pub mod event;
pub mod expression;
pub mod function;
pub mod state;
pub mod statement;
pub mod style;

pub use crate::ast::Program;
pub use platipus_diagnostics::{Error, ErrorKind, Span};
pub use crate::lexer::{ContextualKeyword, Keyword, Token, TokenKind, tokenize, tokenize_lossy};

#[derive(Debug, Clone, Copy)]
pub struct Checkpoint {
    position: usize,
    errors: usize,
}

impl Checkpoint {
    pub fn position(self) -> usize {
        self.position
    }
}

/// Returned when a token the parser needed was not there. The diagnostic has
/// already been recorded on the parser by the time this is produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mismatch;

#[derive(Debug)]
pub struct Parser {
    source_name: String,
    tokens: Vec<Token>,
    current: usize,
    errors: Vec<Error>,
}

#[derive(Debug)]
pub struct ParseOutcome {
    pub program: Program,
    pub errors: Vec<Error>,
}

impl ParseOutcome {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

impl Parser {
    pub fn new(source_name: impl Into<String>, tokens: Vec<Token>) -> Self {
        Self {
            source_name: source_name.into(),
            tokens,
            current: 0,
            errors: Vec::new(),
        }
    }

    pub fn from_source(source_name: impl Into<String>, source: &str) -> Self {
        let name = source_name.into();
        let (tokens, errors) = tokenize_lossy(source);
        Self {
            source_name: name,
            tokens,
            current: 0,
            errors,
        }
    }

    pub fn source_name(&self) -> &str {
        &self.source_name
    }

    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub fn errors(&self) -> &[Error] {
        &self.errors
    }

    pub fn peek(&self) -> &Token {
        self.tokens
            .get(self.current)
            .or_else(|| self.tokens.last())
            .expect("token stream must contain at least an EOF token")
    }

    pub fn peek_next(&self) -> &Token {
        self.tokens
            .get(self.current + 1)
            .or_else(|| self.tokens.last())
            .expect("token stream must contain at least an EOF token")
    }

    pub fn peek_at(&self, offset: usize) -> &Token {
        self.tokens
            .get(self.current + offset)
            .or_else(|| self.tokens.last())
            .expect("token stream must contain at least an EOF token")
    }

    pub fn previous(&self) -> &Token {
        self.tokens
            .get(self.current.saturating_sub(1))
            .or_else(|| self.tokens.last())
            .expect("token stream must contain at least an EOF token")
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            position: self.current,
            errors: self.errors.len(),
        }
    }

    pub fn restore(&mut self, checkpoint: Checkpoint) {
        self.current = checkpoint.position;
        self.errors.truncate(checkpoint.errors);
    }

    pub fn current(&self) -> usize {
        self.current
    }

    pub fn at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    pub fn advance(&mut self) -> Token {
        if !self.at_end() {
            self.current += 1;
        }
        self.previous().clone()
    }

    pub fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    pub fn check_keyword(&self, keyword: Keyword) -> bool {
        self.peek().is_keyword(keyword)
    }

    pub fn check_contextual(&self, keyword: ContextualKeyword) -> bool {
        self.peek().is_identifier() && self.peek().lexeme == keyword.as_str()
    }

    pub fn match_contextual(&mut self, keyword: ContextualKeyword) -> bool {
        if self.check_contextual(keyword) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn check_identifier(&self) -> bool {
        self.peek().is_identifier()
    }

    pub fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn match_keyword(&mut self, keyword: Keyword) -> bool {
        if self.check_keyword(keyword) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn consume(&mut self, kind: TokenKind, expected: &str) -> Result<Token, Mismatch> {
        if self.check(kind) {
            return Ok(self.advance());
        }
        let found = self.peek().describe();
        let span = self.peek().span;
        self.push_error(
            "unexpected-token",
            format!("expected `{expected}` but found {found}"),
            span,
        );
        Err(Mismatch)
    }

    pub fn consume_keyword(&mut self, keyword: Keyword) -> Result<Token, Mismatch> {
        if self.check_keyword(keyword) {
            return Ok(self.advance());
        }
        let found = self.peek().describe();
        let span = self.peek().span;
        self.push_error(
            "unexpected-token",
            format!("expected `{}` but found {found}", keyword.as_str()),
            span,
        );
        Err(Mismatch)
    }

    pub fn consume_identifier(&mut self, expected: &str) -> Result<Token, Mismatch> {
        if self.check_identifier() {
            return Ok(self.advance());
        }
        let found = self.peek().describe();
        let span = self.peek().span;
        self.push_error(
            "expected-identifier",
            format!("expected {expected} but found {found}"),
            span,
        );
        Err(Mismatch)
    }

    pub fn push_error(&mut self, code: &'static str, message: impl Into<String>, span: Span) {
        self.errors
            .push(Error::new(ErrorKind::Parse, code, message).with_span(span));
    }

    pub fn push_error_with_help(
        &mut self,
        code: &'static str,
        message: impl Into<String>,
        span: Span,
        help: impl Into<String>,
    ) {
        self.errors.push(
            Error::new(ErrorKind::Parse, code, message)
                .with_span(span)
                .with_help(help),
        );
    }

    pub fn synchronize(&mut self) {
        self.advance();
        while !self.at_end() {
            if self.peek().kind == TokenKind::Keyword
                && SYNC_KEYWORDS.contains(&self.peek().lexeme.as_str())
            {
                return;
            }
            if self.check(TokenKind::LeftBrace) {
                return;
            }
            self.advance();
        }
    }
}

pub const SYNC_KEYWORDS: &[&str] = &[
    "app",
    "component",
    "fn",
    "state",
    "derived",
    "input",
    "import",
    "theme",
    "api",
    "test",
    "if",
    "for",
    "return",
    "try",
    "emit",
];

pub fn is_element_name(name: &str) -> bool {
    name.chars().next().is_some_and(char::is_uppercase)
}

pub fn looks_like_element(parser: &Parser) -> bool {
    if !parser.check_identifier() {
        return false;
    }
    if !is_element_name(&parser.peek().lexeme) {
        return false;
    }
    !parser.peek_next().is(TokenKind::LeftParen)
}

pub fn parse(source_name: impl Into<String>, source: &str) -> ParseOutcome {
    let name = source_name.into();
    let mut parser = Parser::from_source(name.clone(), source);
    let program = parser.parse_program();
    ParseOutcome {
        program,
        errors: parser.errors,
    }
}

impl Parser {
    pub fn parse_program(&mut self) -> Program {
        app::parse_program(self)
    }
}
