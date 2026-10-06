use super::Parser;
use crate::ast::{ComponentItem, DerivedDecl, StateDecl, StateKind};
use crate::lexer::{Keyword, TokenKind};

use super::app::identifier;

pub fn try_parse_state_item(parser: &mut Parser) -> Option<ComponentItem> {
    if parser.check_keyword(Keyword::Derived) {
        return Some(ComponentItem::Derived(parse_derived(parser)));
    }
    let kind = if parser.check_keyword(Keyword::State) {
        parser.advance();
        StateKind::Local
    } else {
        let modifier = parser.peek().lexeme.clone();
        let kind = match modifier.as_str() {
            "shared" => StateKind::Shared,
            "global" => StateKind::Global,
            "persistent" => StateKind::Persistent,
            _ => return None,
        };
        if !parser.peek_next().is_keyword(Keyword::State) {
            return None;
        }
        parser.advance();
        parser.advance();
        kind
    };
    let start = parser.previous().span.start;
    let name = identifier(parser, "a state name")?;
    let type_annotation = if parser.match_token(TokenKind::Colon) {
        Some(super::expression::parse_type(parser))
    } else {
        None
    };
    let initializer = if parser.match_token(TokenKind::Assign) {
        Some(super::expression::parse_expression(parser))
    } else {
        None
    };
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    Some(ComponentItem::State(StateDecl {
        kind,
        name,
        type_annotation,
        initializer,
        span,
    }))
}

pub fn parse_derived(parser: &mut Parser) -> DerivedDecl {
    let start = parser.advance().span.start;
    let name = identifier(parser, "a derived state name")
        .unwrap_or_else(|| crate::ast::Identifier::new("", platipus_diagnostics::Span::empty()));
    let type_annotation = if parser.match_token(TokenKind::Colon) {
        Some(super::expression::parse_type(parser))
    } else {
        None
    };
    if parser.consume(TokenKind::Assign, "=").is_err() {
        let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
        parser.push_error_with_help(
            "missing-derived-value",
            format!("`derived {name}` requires a value expression"),
            span,
            format!("write `derived {name} = expression`"),
        );
    }
    let value = super::expression::parse_expression(parser);
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    DerivedDecl {
        name,
        type_annotation,
        value,
        span,
    }
}
