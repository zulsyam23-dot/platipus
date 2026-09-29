use super::Parser;
use crate::ast::{Element, ElementBody, ElementItem, PropertyValue};
use crate::lexer::{Keyword, TokenKind};

use super::app::identifier;

pub fn try_parse_element(parser: &mut Parser) -> Option<Element> {
    if !super::looks_like_element(parser) {
        return None;
    }
    let start = parser.peek().span.start;
    let name = identifier(parser, "an element name")?;
    let mut element = Element::new(name, crate::diagnostics::Span::empty());
    let arguments = parse_arguments(parser);
    let text = if parser.check(TokenKind::StringLiteral) {
        Some(
            super::app::string_literal(parser)
                .map(|value| crate::ast::Expression::StringLiteral(value, parser.previous().span))
                .unwrap_or(crate::ast::Expression::NullLiteral(parser.previous().span)),
        )
    } else if arguments.is_empty() && starts_expression(parser) {
        Some(expression::parse_expression(parser))
    } else {
        None
    };
    let body = if parser.check(TokenKind::LeftBrace) {
        Some(parse_element_body(parser))
    } else {
        None
    };
    let span = crate::diagnostics::Span::new(start, parser.previous().span.end);
    element.arguments = arguments;
    element.text = text;
    element.body = body;
    element.span = span;
    Some(element)
}

pub fn parse_arguments(parser: &mut Parser) -> Vec<PropertyValue> {
    let mut arguments = Vec::new();
    while parser.check_identifier() && parser.peek_next().kind == TokenKind::Colon {
        let start = parser.peek().span.start;
        let name = match identifier(parser, "a property name") {
            Some(name) => name,
            None => break,
        };
        parser.advance();
        let value = expression::parse_expression(parser);
        let span = crate::diagnostics::Span::new(start, parser.previous().span.end);
        arguments.push(PropertyValue::new(name, value, span));
    }
    arguments
}

pub fn parse_element_body(parser: &mut Parser) -> ElementBody {
    let open = parser.advance().span;
    let mut items = Vec::new();
    while !parser.check(TokenKind::RightBrace) && !parser.at_end() {
        if let Some(item) = try_parse_element_item(parser) {
            items.push(item);
            continue;
        }
        let found = parser.peek().describe();
        let span = parser.peek().span;
        parser.push_error(
            "invalid-element-body",
            format!(
                "`{}` is not valid inside an element body, found {found}",
                "element"
            ),
            span,
        );
        parser.synchronize();
    }
    parser.consume(TokenKind::RightBrace, "}").ok();
    let span = crate::diagnostics::Span::new(open.start, parser.previous().span.end);
    ElementBody::new(items, span)
}

pub fn try_parse_element_item(parser: &mut Parser) -> Option<ElementItem> {
    if parser.check_keyword(Keyword::Bind) {
        return Some(ElementItem::Binding(parse_binding(parser)));
    }
    if parser.check_keyword(Keyword::On) {
        parser.advance();
        return Some(ElementItem::Handler(event::parse_handler_rest(parser)));
    }
    if parser.check_keyword(Keyword::Style) {
        if parser.peek_next().kind == TokenKind::Colon {
            let start = parser.peek().span.start;
            parser.advance();
            parser.advance();
            let span = crate::diagnostics::Span::new(start, parser.previous().span.end);
            return identifier(parser, "a style name").map(|name| {
                ElementItem::Property(PropertyValue::new(
                    name,
                    crate::ast::Expression::Identifier(crate::ast::Identifier::new(
                        "style-reference",
                        span,
                    )),
                    span,
                ))
            });
        }
        parser.advance();
        return Some(ElementItem::Style(style::parse_style_block_rest(parser)));
    }
    if parser.check_keyword(Keyword::Responsive) {
        parser.advance();
        return Some(ElementItem::Responsive(style::parse_responsive_rest(
            parser,
        )));
    }
    if parser.check_identifier() && parser.peek_next().kind == TokenKind::Colon {
        let start = parser.peek().span.start;
        let name = identifier(parser, "a property name");
        parser.advance();
        let value = expression::parse_expression(parser);
        let span = crate::diagnostics::Span::new(start, parser.previous().span.end);
        return name.map(|name| ElementItem::Property(PropertyValue::new(name, value, span)));
    }
    if let Some(element) = try_parse_element(parser) {
        return Some(ElementItem::Child(element));
    }
    statement::try_parse_statement(parser).map(ElementItem::Stmt)
}

fn parse_binding(parser: &mut Parser) -> crate::ast::BindingDecl {
    let start = parser.advance().span.start;
    let property = identifier(parser, "a bound property name")
        .unwrap_or_else(|| crate::ast::Identifier::new("", crate::diagnostics::Span::empty()));
    parser.consume(TokenKind::Colon, ":").ok();
    let target = expression::parse_expression(parser);
    let span = crate::diagnostics::Span::new(start, parser.previous().span.end);
    crate::ast::BindingDecl {
        property,
        target,
        span,
    }
}

fn starts_expression(parser: &Parser) -> bool {
    if parser.check(TokenKind::Identifier) {
        return true;
    }
    if parser.check(TokenKind::Keyword) {
        return matches!(parser.peek().lexeme.as_str(), "true" | "false" | "null");
    }
    matches!(
        parser.peek().kind,
        TokenKind::IntLiteral
            | TokenKind::FloatLiteral
            | TokenKind::StringLiteral
            | TokenKind::LeftBracket
            | TokenKind::Bang
            | TokenKind::Minus
    )
}

use super::{event, expression, statement, style};
