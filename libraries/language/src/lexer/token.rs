use super::literal::Literal;
use platipus_diagnostics::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Identifier,
    Keyword,
    IntLiteral,
    FloatLiteral,
    StringLiteral,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,

    RustBlock,

    Comma,
    Colon,
    Dot,
    Arrow,
    Question,

    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    Assign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    SlashAssign,
    PercentAssign,

    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    AndAnd,
    OrOr,
    Bang,

    Eof,
}

impl TokenKind {
    pub const fn is_literal(self) -> bool {
        matches!(
            self,
            TokenKind::IntLiteral | TokenKind::FloatLiteral | TokenKind::StringLiteral
        )
    }

    pub const fn is_assignment(self) -> bool {
        matches!(
            self,
            TokenKind::Assign
                | TokenKind::PlusAssign
                | TokenKind::MinusAssign
                | TokenKind::StarAssign
                | TokenKind::SlashAssign
                | TokenKind::PercentAssign
        )
    }

    pub const fn is_trivia(self) -> bool {
        matches!(
            self,
            TokenKind::LeftParen
                | TokenKind::RightParen
                | TokenKind::LeftBrace
                | TokenKind::RightBrace
                | TokenKind::LeftBracket
                | TokenKind::RightBracket
                | TokenKind::Comma
                | TokenKind::Colon
                | TokenKind::Dot
                | TokenKind::Arrow
                | TokenKind::Question
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
    pub literal: Option<Literal>,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            lexeme: lexeme.into(),
            span,
            literal: None,
        }
    }

    pub fn with_literal(mut self, literal: Literal) -> Self {
        self.literal = Some(literal);
        self
    }

    pub fn eof(span: Span) -> Self {
        Self::new(TokenKind::Eof, "", span)
    }

    pub fn is(&self, kind: TokenKind) -> bool {
        self.kind == kind
    }

    pub fn is_identifier(&self) -> bool {
        self.kind == TokenKind::Identifier
    }

    pub fn is_keyword(&self, keyword: super::keyword::Keyword) -> bool {
        self.kind == TokenKind::Keyword && self.lexeme == keyword.as_str()
    }

    pub fn describe(&self) -> String {
        match self.kind {
            TokenKind::RustBlock => "rust block".to_string(),
            TokenKind::Eof => "end of file".to_string(),
            TokenKind::Identifier => format!("identifier `{}`", self.lexeme),
            TokenKind::Keyword => format!("keyword `{}`", self.lexeme),
            TokenKind::StringLiteral => "string literal".to_string(),
            TokenKind::IntLiteral => "integer literal".to_string(),
            TokenKind::FloatLiteral => "float literal".to_string(),
            other => format!("`{}`", other.symbol()),
        }
    }
}

impl TokenKind {
    pub const fn symbol(self) -> &'static str {
        match self {
            TokenKind::LeftParen => "(",
            TokenKind::RightParen => ")",
            TokenKind::LeftBrace => "{",
            TokenKind::RightBrace => "}",
            TokenKind::LeftBracket => "[",
            TokenKind::RightBracket => "]",
            TokenKind::RustBlock => "rust block",
            TokenKind::Comma => ",",
            TokenKind::Colon => ":",
            TokenKind::Dot => ".",
            TokenKind::Arrow => "->",
            TokenKind::Question => "?",
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Percent => "%",
            TokenKind::Assign => "=",
            TokenKind::PlusAssign => "+=",
            TokenKind::MinusAssign => "-=",
            TokenKind::StarAssign => "*=",
            TokenKind::SlashAssign => "/=",
            TokenKind::PercentAssign => "%=",
            TokenKind::Equal => "==",
            TokenKind::NotEqual => "!=",
            TokenKind::Less => "<",
            TokenKind::LessEqual => "<=",
            TokenKind::Greater => ">",
            TokenKind::GreaterEqual => ">=",
            TokenKind::AndAnd => "&&",
            TokenKind::OrOr => "||",
            TokenKind::Bang => "!",
            TokenKind::Identifier => "identifier",
            TokenKind::Keyword => "keyword",
            TokenKind::IntLiteral => "int",
            TokenKind::FloatLiteral => "float",
            TokenKind::StringLiteral => "string",
            TokenKind::Eof => "eof",
        }
    }
}
