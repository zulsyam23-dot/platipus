use super::Parser;
use crate::ast::{Block, EmitStatement, EventHandlerDecl};
use crate::lexer::{Keyword, TokenKind};

use super::app::identifier;

pub fn parse_handler_rest(parser: &mut Parser) -> EventHandlerDecl {
    let start = parser.previous().span.end;
    let name = event_name(parser)
        .unwrap_or_else(|| crate::ast::Identifier::new("", platipus_diagnostics::Span::empty()));
    let body = super::statement::parse_block(parser);
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    EventHandlerDecl::new(name, body, span)
}

pub fn event_name(parser: &mut Parser) -> Option<crate::ast::Identifier> {
    if parser.check(TokenKind::Keyword)
        && crate::ast::known::is_known(parser.peek().lexeme.as_str())
    {
        let token = parser.advance();
        return Some(crate::ast::Identifier::new(token.lexeme, token.span));
    }
    identifier(parser, "an event name")
}

pub fn try_parse_emit(parser: &mut Parser) -> Option<EmitStatement> {
    if !parser.check_keyword(Keyword::Emit) {
        return None;
    }
    let start = parser.advance().span.start;
    let name = identifier(parser, "a custom event name")?;
    let mut arguments = Vec::new();
    if parser.check(TokenKind::LeftParen) {
        parser.advance();
        arguments = super::expression::parse_arguments(parser);
    } else {
        parser.push_error_with_help(
            "malformed-emit",
            format!("`emit {name}` must send a payload with parentheses"),
            parser.peek().span,
            format!("write `emit {name}(value)`"),
        );
    }
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    Some(EmitStatement {
        name,
        arguments,
        span,
    })
}

pub fn parse_block(parser: &mut Parser) -> Block {
    let open = parser.peek().span;
    if parser.consume(TokenKind::LeftBrace, "{").is_err() {
        return Block::new(Vec::new(), open);
    }
    let mut statements = Vec::new();
    while !parser.check(TokenKind::RightBrace) && !parser.at_end() {
        if let Some(statement) = super::statement::try_parse_statement(parser) {
            statements.push(statement);
            continue;
        }
        let found = parser.peek().describe();
        let span = parser.peek().span;
        parser.push_error(
            "invalid-statement",
            format!("expected a statement but found {found}"),
            span,
        );
        parser.synchronize();
    }
    parser.consume(TokenKind::RightBrace, "}").ok();
    let span = platipus_diagnostics::Span::new(open.start, parser.previous().span.end);
    Block::new(statements, span)
}
