use super::Parser;
use crate::ast::{
    Breakpoint, ResponsiveBlock, ResponsiveEntry, ResponsiveEntryKind, StyleBlock, StyleDefinition,
    StyleEntry, StyleState, StyleStateGroup, ThemeDecl, ThemeToken,
};
use crate::lexer::{ContextualKeyword, Keyword, TokenKind};

use super::app::{identifier, string_literal};

pub fn parse_style_definition(parser: &mut Parser) -> Option<StyleDefinition> {
    if !parser.check_keyword(Keyword::Style) || !parser.peek_next().is_identifier() {
        return None;
    }
    parser.advance();
    let start = parser.previous().span.start;
    let name = identifier(parser, "a style name")?;
    let block = parse_style_block_rest(parser);
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    Some(StyleDefinition { name, block, span })
}

pub fn parse_style_definitions(parser: &mut Parser) -> Vec<StyleDefinition> {
    let mut definitions = Vec::new();
    while let Some(definition) = parse_style_definition(parser) {
        definitions.push(definition);
    }
    definitions
}

pub fn parse_style_block_rest(parser: &mut Parser) -> StyleBlock {
    let open = parser
        .consume(TokenKind::LeftBrace, "{")
        .map(|t| t.span)
        .unwrap_or_default();
    let mut entries = Vec::new();
    let mut groups = Vec::new();
    while !parser.check(TokenKind::RightBrace) && !parser.at_end() {
        if let Some(state) = style_state(parser) {
            let group_start = parser.previous().span.start;
            let block = parse_style_block_rest(parser);
            let span = platipus_diagnostics::Span::new(group_start, parser.previous().span.end);
            groups.push(StyleStateGroup { state, block, span });
            continue;
        }
        if parser.check(TokenKind::StringLiteral) {
            let start = parser.peek().span.start;
            let name = string_literal(parser).unwrap_or_default();
            parser.consume(TokenKind::Colon, ":").ok();
            let value = super::expression::parse_expression(parser);
            let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
            entries.push(StyleEntry::Property {
                name: crate::ast::Identifier::new(name, span),
                value,
                span,
            });
            continue;
        }
        if parser.check_identifier() {
            let start = parser.peek().span.start;
            let Some(name) = identifier(parser, "a style property") else {
                parser.synchronize();
                continue;
            };
            if parser.consume(TokenKind::Colon, ":").is_err() {
                parser.synchronize();
                continue;
            }
            let value = super::expression::parse_expression(parser);
            let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
            entries.push(StyleEntry::Property { name, value, span });
            continue;
        }
        let found = parser.peek().describe();
        let span = parser.peek().span;
        parser.push_error(
            "invalid-style-entry",
            format!("expected a style property but found {found}"),
            span,
        );
        parser.synchronize();
    }
    parser.consume(TokenKind::RightBrace, "}").ok();
    let span = platipus_diagnostics::Span::new(open.start, parser.previous().span.end);
    StyleBlock::new(entries, groups, span)
}

pub fn parse_responsive_rest(parser: &mut Parser) -> ResponsiveBlock {
    let open = parser
        .consume(TokenKind::LeftBrace, "{")
        .map(|t| t.span)
        .unwrap_or_default();
    let mut entries = Vec::new();
    while !parser.check(TokenKind::RightBrace) && !parser.at_end() {
        let start = parser.peek().span.start;
        let Some(breakpoint) = breakpoint(parser) else {
            parser.synchronize();
            continue;
        };
        parser.consume(TokenKind::Colon, ":").ok();
        let kind = if parser.check_identifier() && parser.peek_next().kind == TokenKind::Colon {
            match identifier(parser, "a responsive property") {
                Some(name) => {
                    parser.advance();
                    ResponsiveEntryKind::Property {
                        name,
                        value: super::expression::parse_expression(parser),
                    }
                }
                None => {
                    parser.synchronize();
                    continue;
                }
            }
        } else if parser.check_identifier() {
            match identifier(parser, "a responsive value") {
                Some(name) => ResponsiveEntryKind::Named { name },
                None => {
                    parser.synchronize();
                    continue;
                }
            }
        } else {
            parser.synchronize();
            continue;
        };
        let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
        entries.push(ResponsiveEntry {
            breakpoint,
            kind,
            span,
        });
    }
    parser.consume(TokenKind::RightBrace, "}").ok();
    let span = platipus_diagnostics::Span::new(open.start, parser.previous().span.end);
    ResponsiveBlock { entries, span }
}

pub fn parse_theme(parser: &mut Parser) -> Option<ThemeDecl> {
    if !parser.check_keyword(Keyword::Theme) {
        return None;
    }
    parser.advance();
    let start = parser.previous().span.start;
    let name = identifier(parser, "a theme name")?;
    if parser.consume(TokenKind::LeftBrace, "{").is_err() {
        return None;
    }
    let mut tokens = Vec::new();
    while !parser.check(TokenKind::RightBrace) && !parser.at_end() {
        let token_start = parser.peek().span.start;
        let mut path = Vec::new();
        while let Some(segment) = identifier(parser, "a theme token segment") {
            path.push(segment);
            if !parser.match_token(TokenKind::Dot) {
                break;
            }
        }
        if path.is_empty() {
            break;
        }
        if parser.consume(TokenKind::Colon, ":").is_err() {
            break;
        }
        let value = super::expression::parse_expression(parser);
        let span = platipus_diagnostics::Span::new(token_start, parser.previous().span.end);
        tokens.push(ThemeToken { path, value, span });
    }
    if parser.consume(TokenKind::RightBrace, "}").is_err() {
        return None;
    }
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    Some(ThemeDecl { name, tokens, span })
}

pub fn parse_themes(parser: &mut Parser) -> Vec<ThemeDecl> {
    let mut themes = Vec::new();
    while let Some(theme) = parse_theme(parser) {
        themes.push(theme);
    }
    themes
}

fn style_state(parser: &mut Parser) -> Option<StyleState> {
    if !parser.check_identifier() {
        return None;
    }
    if parser.peek_next().kind != TokenKind::LeftBrace {
        return None;
    }
    let state = match ContextualKeyword::from_ident(&parser.peek().lexeme) {
        Some(ContextualKeyword::Normal) => StyleState::Normal,
        Some(ContextualKeyword::Hover) => StyleState::Hover,
        Some(ContextualKeyword::Pressed) => StyleState::Pressed,
        Some(ContextualKeyword::Focused) => StyleState::Focused,
        Some(ContextualKeyword::Disabled) => StyleState::Disabled,
        Some(ContextualKeyword::Selected) => StyleState::Selected,
        _ => return None,
    };
    parser.advance();
    Some(state)
}

fn breakpoint(parser: &mut Parser) -> Option<Breakpoint> {
    let name = identifier(parser, "a breakpoint name")?;
    let breakpoint = match name.as_str() {
        "mobile" => Breakpoint::Mobile,
        "tablet" => Breakpoint::Tablet,
        "desktop" => Breakpoint::Desktop,
        other => {
            let span = name.span;
            parser.push_error_with_help(
                "unknown-breakpoint",
                format!("`{other}` is not a known breakpoint"),
                span,
                "use `mobile`, `tablet`, or `desktop`",
            );
            return None;
        }
    };
    Some(breakpoint)
}
