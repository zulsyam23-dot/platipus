use super::token::TokenKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None,
    Or,
    And,
    Equality,
    Comparison,
    Sum,
    Product,
    Unary,
    Call,
}

#[derive(Debug, Clone, Copy)]
pub struct OperatorInfo {
    pub symbol: &'static str,
    pub kind: TokenKind,
    pub precedence: Precedence,
    pub right_associative: bool,
}

pub const OPERATORS: &[OperatorInfo] = &[
    OperatorInfo {
        symbol: "||",
        kind: TokenKind::OrOr,
        precedence: Precedence::Or,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "&&",
        kind: TokenKind::AndAnd,
        precedence: Precedence::And,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "==",
        kind: TokenKind::Equal,
        precedence: Precedence::Equality,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "!=",
        kind: TokenKind::NotEqual,
        precedence: Precedence::Equality,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "<=",
        kind: TokenKind::LessEqual,
        precedence: Precedence::Comparison,
        right_associative: false,
    },
    OperatorInfo {
        symbol: ">=",
        kind: TokenKind::GreaterEqual,
        precedence: Precedence::Comparison,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "<",
        kind: TokenKind::Less,
        precedence: Precedence::Comparison,
        right_associative: false,
    },
    OperatorInfo {
        symbol: ">",
        kind: TokenKind::Greater,
        precedence: Precedence::Comparison,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "+",
        kind: TokenKind::Plus,
        precedence: Precedence::Sum,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "-",
        kind: TokenKind::Minus,
        precedence: Precedence::Sum,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "*",
        kind: TokenKind::Star,
        precedence: Precedence::Product,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "/",
        kind: TokenKind::Slash,
        precedence: Precedence::Product,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "%",
        kind: TokenKind::Percent,
        precedence: Precedence::Product,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "+=",
        kind: TokenKind::PlusAssign,
        precedence: Precedence::None,
        right_associative: true,
    },
    OperatorInfo {
        symbol: "-=",
        kind: TokenKind::MinusAssign,
        precedence: Precedence::None,
        right_associative: true,
    },
    OperatorInfo {
        symbol: "*=",
        kind: TokenKind::StarAssign,
        precedence: Precedence::None,
        right_associative: true,
    },
    OperatorInfo {
        symbol: "/=",
        kind: TokenKind::SlashAssign,
        precedence: Precedence::None,
        right_associative: true,
    },
    OperatorInfo {
        symbol: "%=",
        kind: TokenKind::PercentAssign,
        precedence: Precedence::None,
        right_associative: true,
    },
    OperatorInfo {
        symbol: "=",
        kind: TokenKind::Assign,
        precedence: Precedence::None,
        right_associative: true,
    },
    OperatorInfo {
        symbol: "!",
        kind: TokenKind::Bang,
        precedence: Precedence::Unary,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "->",
        kind: TokenKind::Arrow,
        precedence: Precedence::None,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "?",
        kind: TokenKind::Question,
        precedence: Precedence::None,
        right_associative: false,
    },
    OperatorInfo {
        symbol: ".",
        kind: TokenKind::Dot,
        precedence: Precedence::Call,
        right_associative: false,
    },
    OperatorInfo {
        symbol: ",",
        kind: TokenKind::Comma,
        precedence: Precedence::None,
        right_associative: false,
    },
    OperatorInfo {
        symbol: ":",
        kind: TokenKind::Colon,
        precedence: Precedence::None,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "(",
        kind: TokenKind::LeftParen,
        precedence: Precedence::None,
        right_associative: false,
    },
    OperatorInfo {
        symbol: ")",
        kind: TokenKind::RightParen,
        precedence: Precedence::None,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "{",
        kind: TokenKind::LeftBrace,
        precedence: Precedence::None,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "}",
        kind: TokenKind::RightBrace,
        precedence: Precedence::None,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "[",
        kind: TokenKind::LeftBracket,
        precedence: Precedence::None,
        right_associative: false,
    },
    OperatorInfo {
        symbol: "]",
        kind: TokenKind::RightBracket,
        precedence: Precedence::None,
        right_associative: false,
    },
];

pub fn lookup(symbol: &str) -> Option<&'static OperatorInfo> {
    OPERATORS.iter().find(|info| info.symbol == symbol)
}

pub fn precedence_of(kind: TokenKind) -> Precedence {
    OPERATORS
        .iter()
        .find(|info| info.kind == kind)
        .map(|info| info.precedence)
        .unwrap_or(Precedence::None)
}

pub fn infix_precedence(kind: TokenKind) -> Precedence {
    precedence_of(kind)
}

pub fn is_right_associative(kind: TokenKind) -> bool {
    OPERATORS
        .iter()
        .find(|info| info.kind == kind)
        .map(|info| info.right_associative)
        .unwrap_or(false)
}
