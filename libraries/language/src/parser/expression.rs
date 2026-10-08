use super::Parser;
use crate::ast::{
    AssignOp, Assignment, BinaryOp, Block, Expression, Identifier, LogicalOp, ObjectEntry,
    PropertyKey, TypeExpr, UnaryOp,
};
use crate::lexer::{Keyword, Literal, TokenKind, operator::Precedence};

use super::app::identifier;

pub fn parse_expression(parser: &mut Parser) -> Expression {
    let expression = parse_expression_raw(parser);
    // `a..b` is only meaningful in a `for` header; everywhere else it would
    // otherwise be reported as a stray token far from its cause.
    if matches!(
        parser.peek().kind,
        TokenKind::DotDot | TokenKind::DotDotEqual
    ) {
        let span = parser.peek().span;
        parser.push_error_with_help(
            "range-outside-for",
            "`..` ranges may only appear in a `for` loop header",
            span,
            "write `for i in start..end { ... }`",
        );
        parser.advance();
        let _ = parse_precedence(parser, Precedence::None);
    }
    expression
}

/// An expression with no trailing range check; `for` headers use it so they can
/// claim `..` for themselves.
pub fn parse_expression_raw(parser: &mut Parser) -> Expression {
    parse_precedence(parser, Precedence::None)
}

pub fn parse_precedence(parser: &mut Parser, limit: Precedence) -> Expression {
    let mut left = parse_unary(parser);
    while let Some(info) = binary_info(parser.peek().kind) {
        if info.0 <= limit {
            break;
        }
        let op = binary_operator(parser.advance().kind);
        let right = parse_precedence(parser, info.1);
        let span = left.span().join(right.span());
        left = match op {
            Some(LogicalOperator::Logical(logical)) => Expression::Logical {
                op: logical,
                left: Box::new(left),
                right: Box::new(right),
                span,
            },
            Some(LogicalOperator::Binary(binary)) => Expression::Binary {
                op: binary,
                left: Box::new(left),
                right: Box::new(right),
                span,
            },
            None => left,
        };
    }
    left
}

enum LogicalOperator {
    Logical(LogicalOp),
    Binary(BinaryOp),
}

fn binary_info(kind: TokenKind) -> Option<(Precedence, Precedence)> {
    use TokenKind as T;
    let precedence = match kind {
        T::OrOr => Precedence::Or,
        T::AndAnd => Precedence::And,
        T::Equal | T::NotEqual => Precedence::Equality,
        T::Less | T::LessEqual | T::Greater | T::GreaterEqual => Precedence::Comparison,
        T::Pipe => Precedence::BitOr,
        T::Caret => Precedence::BitXor,
        T::Amp => Precedence::BitAnd,
        T::Shl | T::Shr => Precedence::Shift,
        T::Plus | T::Minus => Precedence::Sum,
        T::Star | T::Slash | T::Percent => Precedence::Product,
        _ => return None,
    };
    // Left-associative: the right operand stops as soon as it meets an
    // operator of equal precedence, so `a - b - c` stays `(a - b) - c`
    // while a tighter operator is still absorbed (`a + b * c`).
    Some((precedence, precedence))
}

fn binary_operator(kind: TokenKind) -> Option<LogicalOperator> {
    use TokenKind as T;
    match kind {
        T::OrOr => Some(LogicalOperator::Logical(LogicalOp::Or)),
        T::AndAnd => Some(LogicalOperator::Logical(LogicalOp::And)),
        T::Equal => Some(LogicalOperator::Binary(BinaryOp::Eq)),
        T::NotEqual => Some(LogicalOperator::Binary(BinaryOp::Ne)),
        T::Less => Some(LogicalOperator::Binary(BinaryOp::Lt)),
        T::LessEqual => Some(LogicalOperator::Binary(BinaryOp::Le)),
        T::Greater => Some(LogicalOperator::Binary(BinaryOp::Gt)),
        T::GreaterEqual => Some(LogicalOperator::Binary(BinaryOp::Ge)),
        T::Pipe => Some(LogicalOperator::Binary(BinaryOp::Or)),
        T::Caret => Some(LogicalOperator::Binary(BinaryOp::Xor)),
        T::Amp => Some(LogicalOperator::Binary(BinaryOp::And)),
        T::Shl => Some(LogicalOperator::Binary(BinaryOp::Shl)),
        T::Shr => Some(LogicalOperator::Binary(BinaryOp::Shr)),
        T::Plus => Some(LogicalOperator::Binary(BinaryOp::Add)),
        T::Minus => Some(LogicalOperator::Binary(BinaryOp::Sub)),
        T::Star => Some(LogicalOperator::Binary(BinaryOp::Mul)),
        T::Slash => Some(LogicalOperator::Binary(BinaryOp::Div)),
        T::Percent => Some(LogicalOperator::Binary(BinaryOp::Rem)),
        _ => None,
    }
}

fn parse_unary(parser: &mut Parser) -> Expression {
    match parser.peek().kind {
        TokenKind::Bang | TokenKind::Minus | TokenKind::Tilde => {
            let op = match parser.advance().kind {
                TokenKind::Bang => UnaryOp::Not,
                TokenKind::Minus => UnaryOp::Negate,
                TokenKind::Tilde => UnaryOp::Negate,
                _ => UnaryOp::Negate,
            };
            let operand = parse_unary(parser);
            let span = operand.span();
            Expression::Unary {
                op,
                operand: Box::new(operand),
                span,
            }
        }
        _ if parser.check_keyword(Keyword::Await) => {
            let start = parser.advance().span.start;
            let operand = parse_unary(parser);
            let span = platipus_diagnostics::Span::new(start, operand.span().end);
            Expression::Await {
                operand: Box::new(operand),
                span,
            }
        }
        _ => parse_postfix(parser),
    }
}

fn parse_postfix(parser: &mut Parser) -> Expression {
    let mut expression = parse_primary(parser);
    loop {
        if parser.check(TokenKind::Dot) {
            parser.advance();
            let Some(property) = identifier(parser, "a property name") else {
                return expression;
            };
            let span = expression.span().join(property.span);
            expression = Expression::Member {
                object: Box::new(expression),
                property,
                span,
            };
            continue;
        }
        if parser.check(TokenKind::LeftBracket) {
            parser.advance();
            let index = parse_expression(parser);
            parser.consume(TokenKind::RightBracket, "]").ok();
            let span = expression.span().join(index.span());
            expression = Expression::Index {
                object: Box::new(expression),
                index: Box::new(index),
                span,
            };
            continue;
        }
        if parser.check(TokenKind::LeftParen) {
            parser.advance();
            let arguments = parse_arguments(parser);
            let span = expression.span().join(parser.previous().span);
            expression = Expression::Call {
                callee: Box::new(expression),
                arguments,
                span,
            };
            continue;
        }
        return expression;
    }
}

pub fn parse_arguments(parser: &mut Parser) -> Vec<Expression> {
    let mut arguments = Vec::new();
    while !parser.check(TokenKind::RightParen) && !parser.at_end() {
        arguments.push(parse_expression(parser));
        if !parser.match_token(TokenKind::Comma) {
            break;
        }
    }
    parser.consume(TokenKind::RightParen, ")").ok();
    arguments
}

fn parse_primary(parser: &mut Parser) -> Expression {
    let token = parser.peek().clone();
    match token.kind {
        TokenKind::IntLiteral => {
            parser.advance();
            match token.literal {
                Some(Literal::Int(value)) => Expression::IntLiteral(value, token.span),
                _ => Expression::IntLiteral(0, token.span),
            }
        }
        TokenKind::FloatLiteral => {
            parser.advance();
            match token.literal {
                Some(Literal::Float(value)) => Expression::FloatLiteral(value, token.span),
                _ => Expression::FloatLiteral(0.0, token.span),
            }
        }
        TokenKind::StringLiteral => {
            parser.advance();
            match token.literal {
                Some(Literal::Str(value)) => Expression::StringLiteral(value, token.span),
                _ => Expression::StringLiteral(token.lexeme, token.span),
            }
        }
        TokenKind::LeftBracket => parse_array(parser),
        TokenKind::LeftBrace => parse_object(parser),
        TokenKind::LeftParen => {
            let start = parser.advance().span.start;
            if parser.check(TokenKind::RightParen) {
                let end = parser.advance().span;
                if matches!(parser.peek().kind, TokenKind::FatArrow) {
                    parser.advance();
                    let body = parse_lambda_body(parser);
                    let span = platipus_diagnostics::Span::new(start, body.span().end);
                    return Expression::Lambda {
                        parameters: vec![],
                        body,
                        span,
                    };
                }
                parser.push_error("unexpected-token", "expected \"=>\" after ()", end);
                return Expression::NullLiteral(platipus_diagnostics::Span::new(start, end.end));
            }
            let inner = parse_expression(parser);
            parser.consume(TokenKind::RightParen, ")").ok();
            inner
        }
        TokenKind::Identifier => {
            if token.lexeme == "event" {
                parser.advance();
                return Expression::Event(token.span);
            }
            let checkpoint = parser.checkpoint();
            parser.advance();
            if matches!(parser.peek().kind, TokenKind::FatArrow) {
                let params = vec![Identifier::new(token.lexeme.clone(), token.span)];
                let _ = parser.advance();
                let body = parse_lambda_body(parser);
                let span = token.span.join(body.span());
                return Expression::Lambda {
                    parameters: params,
                    body,
                    span,
                };
            }
            parser.restore(checkpoint);
            parser.advance();
            Expression::Identifier(Identifier::new(token.lexeme, token.span))
        }
        TokenKind::Keyword => match token.lexeme.as_str() {
            "true" => {
                parser.advance();
                Expression::BoolLiteral(true, token.span)
            }
            "false" => {
                parser.advance();
                Expression::BoolLiteral(false, token.span)
            }
            "null" => {
                parser.advance();
                Expression::NullLiteral(token.span)
            }
            _ => {
                parser.advance();
                parser.push_error(
                    "unexpected-keyword",
                    format!("`{}` cannot start an expression", token.lexeme),
                    token.span,
                );
                Expression::NullLiteral(token.span)
            }
        },
        _ => {
            parser.advance();
            parser.push_error(
                "expected-expression",
                format!("expected an expression but found {}", token.describe()),
                token.span,
            );
            Expression::NullLiteral(token.span)
        }
    }
}

fn parse_array(parser: &mut Parser) -> Expression {
    let open = parser.advance().span;
    let mut items = Vec::new();
    while !parser.check(TokenKind::RightBracket) && !parser.at_end() {
        items.push(parse_expression(parser));
        if !parser.match_token(TokenKind::Comma) {
            break;
        }
    }
    parser.consume(TokenKind::RightBracket, "]").ok();
    Expression::ArrayLiteral(items, open.join(parser.previous().span))
}

fn parse_object(parser: &mut Parser) -> Expression {
    let open = parser.advance().span;
    let mut entries = Vec::new();
    while !parser.check(TokenKind::RightBrace) && !parser.at_end() {
        let start = parser.peek().span.start;
        let key = if parser.check(TokenKind::StringLiteral) {
            let token = parser.advance();
            PropertyKey::String(token.lexeme, token.span)
        } else if let Some(name) = identifier(parser, "an object key") {
            PropertyKey::Named(name)
        } else {
            parser.synchronize();
            continue;
        };
        parser.consume(TokenKind::Colon, ":").ok();
        let value = parse_expression(parser);
        let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
        entries.push(ObjectEntry { key, value, span });
        if !parser.match_token(TokenKind::Comma) {
            break;
        }
    }
    parser.consume(TokenKind::RightBrace, "}").ok();
    Expression::ObjectLiteral(entries, open.join(parser.previous().span))
}

pub fn parse_type(parser: &mut Parser) -> TypeExpr {
    let Some(name) = identifier(parser, "a type name") else {
        return TypeExpr::Named {
            name: Identifier::new("", platipus_diagnostics::Span::empty()),
            arguments: Vec::new(),
            span: platipus_diagnostics::Span::empty(),
        };
    };
    let mut arguments = Vec::new();
    let mut span = name.span;
    if parser.check(TokenKind::Less) {
        parser.advance();
        while !parser.check(TokenKind::Greater) && !parser.at_end() {
            arguments.push(parse_type(parser));
            if !parser.match_token(TokenKind::Comma) {
                break;
            }
        }
        if parser.consume(TokenKind::Greater, ">").is_err() {
            parser.push_error_with_help(
                "unclosed-type-arguments",
                "type argument list is never closed with `>`",
                parser.peek().span,
                "close the list with `>`",
            );
        } else {
            span = span.join(parser.previous().span);
        }
    }
    if parser.check(TokenKind::Question) {
        parser.advance();
        span = span.join(parser.previous().span);
        return TypeExpr::Nullable {
            inner: Box::new(TypeExpr::Named {
                name,
                arguments,
                span,
            }),
            span,
        };
    }
    TypeExpr::Named {
        name,
        arguments,
        span,
    }
}


fn parse_lambda_body(parser: &mut Parser) -> crate::ast::expression::LambdaBody {
    if parser.check(TokenKind::LeftBrace) {
        let block = super::event::parse_block(parser);
        crate::ast::expression::LambdaBody::Block(Box::new(block))
    } else {
        let expression = parse_expression(parser);
        crate::ast::expression::LambdaBody::Expression(Box::new(expression))
    }
}

impl crate::ast::expression::LambdaBody {
    pub fn span(&self) -> platipus_diagnostics::Span {
        match self {
            crate::ast::expression::LambdaBody::Expression(expr) => expr.span(),
            crate::ast::expression::LambdaBody::Block(block) => block.span,
        }
    }
}

pub fn try_parse_assignment(parser: &mut Parser) -> Option<Assignment> {
    let start = parser.peek().span.start;
    let target = parse_postfix(parser);
    if !parser.peek().kind.is_assignment() {
        return None;
    }
    let op = assign_op(parser.advance().kind);
    let value = parse_expression(parser);
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    Some(Assignment {
        target,
        op,
        value,
        span,
    })
}

pub fn assign_op(kind: TokenKind) -> AssignOp {
    match kind {
        TokenKind::PlusAssign => AssignOp::Add,
        TokenKind::MinusAssign => AssignOp::Sub,
        TokenKind::StarAssign => AssignOp::Mul,
        TokenKind::SlashAssign => AssignOp::Div,
        TokenKind::PercentAssign => AssignOp::Rem,
        _ => AssignOp::Assign,
    }
}

pub fn try_parse_block(parser: &mut Parser) -> Option<Block> {
    if !parser.check(TokenKind::LeftBrace) {
        return None;
    }
    Some(super::event::parse_block(parser))
}
