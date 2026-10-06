use super::Parser;
use crate::ast::{FunctionDecl, Parameter};
use crate::lexer::{Keyword, TokenKind};

use super::app::identifier;

pub fn parse_function_rest(parser: &mut Parser, is_async: bool) -> FunctionDecl {
    let start = parser.previous().span.end;
    let name = identifier(parser, "a function name")
        .unwrap_or_else(|| crate::ast::Identifier::new("", platipus_diagnostics::Span::empty()));
    let parameters = parse_parameters(parser);
    let return_type = if parser.match_token(TokenKind::Arrow) {
        Some(super::expression::parse_type(parser))
    } else {
        None
    };
    let body = super::event::parse_block(parser);
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    FunctionDecl {
        is_async,
        name,
        parameters,
        return_type,
        body,
        span,
    }
}

pub fn try_parse_function(parser: &mut Parser) -> Option<FunctionDecl> {
    let is_async = parser.match_keyword(Keyword::Async);
    if !parser.check_keyword(Keyword::Fn) {
        return None;
    }
    parser.advance();
    Some(parse_function_rest(parser, is_async))
}

pub fn parse_parameters(parser: &mut Parser) -> Vec<Parameter> {
    let mut parameters = Vec::new();
    if parser.consume(TokenKind::LeftParen, "(").is_err() {
        return parameters;
    }
    while !parser.check(TokenKind::RightParen) && !parser.at_end() {
        let start = parser.peek().span.start;
        let Some(name) = identifier(parser, "a parameter name") else {
            break;
        };
        let type_annotation = if parser.match_token(TokenKind::Colon) {
            Some(super::expression::parse_type(parser))
        } else {
            None
        };
        let default = if parser.match_token(TokenKind::Assign) {
            Some(super::expression::parse_expression(parser))
        } else {
            None
        };
        let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
        parameters.push(Parameter {
            name,
            type_annotation,
            default,
            span,
        });
        if !parser.match_token(TokenKind::Comma) {
            break;
        }
    }
    parser.consume(TokenKind::RightParen, ")").ok();
    parameters
}
