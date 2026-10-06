use super::Parser;
use crate::ast::{ComponentDecl, ComponentItem, InputDecl};
use crate::lexer::{Keyword, TokenKind};

use super::app::identifier;

pub fn parse_component(parser: &mut Parser) -> Option<ComponentDecl> {
    if !parser.check_keyword(Keyword::Component) {
        return None;
    }
    parser.advance();
    let start = parser.previous().span.start;
    let name = identifier(parser, "a component name")?;
    let mut inputs = parse_inputs(parser);
    if parser.consume(TokenKind::LeftBrace, "{").is_err() {
        return None;
    }
    if inputs.is_empty() {
        inputs = parse_inputs(parser);
    }
    let body = parse_component_body_items(parser);
    if parser.consume(TokenKind::RightBrace, "}").is_err() {
        return None;
    }
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    Some(ComponentDecl {
        name,
        inputs,
        body,
        span,
    })
}

pub fn parse_components(parser: &mut Parser) -> Vec<ComponentDecl> {
    let mut components = Vec::new();
    while let Some(component) = parse_component(parser) {
        components.push(component);
    }
    components
}

pub fn parse_inputs(parser: &mut Parser) -> Vec<InputDecl> {
    let mut inputs = Vec::new();
    while parser.check_keyword(Keyword::Input) {
        parser.advance();
        let start = parser.previous().span.start;
        let Some(name) = identifier(parser, "an input name") else {
            break;
        };
        let type_annotation = if parser.match_token(TokenKind::Colon) {
            Some(expression::parse_type(parser))
        } else {
            None
        };
        let default = if parser.match_token(TokenKind::Assign) {
            Some(expression::parse_expression(parser))
        } else {
            None
        };
        let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
        inputs.push(InputDecl {
            name,
            type_annotation,
            default,
            span,
        });
    }
    inputs
}

pub fn parse_component_body_items(parser: &mut Parser) -> Vec<ComponentItem> {
    let mut items = Vec::new();
    while !parser.check(TokenKind::RightBrace) && !parser.at_end() {
        if let Some(item) = state::try_parse_state_item(parser) {
            items.push(item);
            continue;
        }
        if parser.match_keyword(Keyword::Fn) {
            items.push(ComponentItem::Function(function::parse_function_rest(
                parser, false,
            )));
            continue;
        }
        if parser.check_keyword(Keyword::Async) && parser.peek_next().is_keyword(Keyword::Fn) {
            parser.advance();
            parser.advance();
            items.push(ComponentItem::Function(function::parse_function_rest(
                parser, true,
            )));
            continue;
        }
        if parser.match_keyword(Keyword::On) {
            items.push(ComponentItem::Handler(event::parse_handler_rest(parser)));
            continue;
        }
        if parser.check_keyword(Keyword::Style) && parser.peek_next().is_keyword(Keyword::Style) {
            parser.advance();
            parser.advance();
            items.push(ComponentItem::Style(style::parse_style_block_rest(parser)));
            continue;
        }
        if let Some(element) = element::try_parse_element(parser) {
            items.push(ComponentItem::Child(element));
            continue;
        }
        if let Some(statement) = statement::try_parse_statement(parser) {
            items.push(ComponentItem::Stmt(statement));
            continue;
        }
        parser.synchronize();
    }
    items
}

use super::{element, event, expression, function, state, statement, style};
