use super::Parser;
use crate::ast::{ApiDecl, ApiMethod, AppDecl, ImportDecl, Program, TestDecl, TestStep};
use crate::lexer::{ContextualKeyword, Keyword, TokenKind};
use super::function;

pub fn parse_program(parser: &mut Parser) -> Program {
    let mut program = Program::new(parser.source_name());
    let start = parser.peek().span.start;
    while !parser.at_end() {
        let before = parser.current();
        parse_top_level(parser, &mut program);
        if parser.current() == before {
            let token = parser.peek().clone();
            parser.push_error(
                "unexpected-top-level",
                format!(
                    "expected a top-level declaration but found {}",
                    token.describe()
                ),
                token.span,
            );
            parser.synchronize();
        }
    }
    program.span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    program
}

fn parse_top_level(parser: &mut Parser, program: &mut Program) {
    if parser.check_keyword(Keyword::App) {
        if let Some(app) = parse_app(parser) {
            if let Some(previous) = program.app.replace(app) {
                parser.push_error_with_help(
                    "duplicate-app",
                    format!("`{}` is already declared", previous.name.as_str()),
                    program
                        .app
                        .as_ref()
                        .map(|_| parser.previous().span)
                        .unwrap_or_default(),
                    "an application file declares exactly one app",
                );
            }
        }
        return;
    }
    if parser.check_keyword(Keyword::Component) {
        program
            .components
            .extend(component::parse_component(parser));
        return;
    }
    if parser.check_keyword(Keyword::Api) {
        program.apis.extend(parse_api(parser));
        return;
    }
    if parser.check_keyword(Keyword::Import) {
        program.imports.extend(parse_import(parser));
        return;
    }
    if parser.check_keyword(Keyword::Fn) || parser.check_keyword(Keyword::Async) {
        if let Some(decl) = function::try_parse_function(parser) {
            program.functions.push(decl);
        }
        return;
    }
    if parser.check_keyword(Keyword::Test) {
        program.tests.extend(parse_test(parser));
        return;
    }
    if parser.check(TokenKind::RustBlock) {
        let token = parser.advance();
        let (attributes, source_start) = match token.literal {
            Some(crate::lexer::Literal::RustBlock(attributes, source_start)) => {
                (attributes, source_start)
            }
            _ => (Vec::new(), token.span.start),
        };
        program.rust_blocks.push(crate::ast::RustBlock::new(
            attributes,
            token.lexeme,
            token.span,
            source_start,
        ));
        return;
    }
    if parser.check_keyword(Keyword::Style) && parser.peek_next().is_identifier() {
        program.styles.extend(style::parse_style_definition(parser));
        return;
    }
    if parser.check_keyword(Keyword::Theme) {
        program.themes.extend(style::parse_theme(parser));
        return;
    }
    parser.advance();
}

pub fn parse_app(parser: &mut Parser) -> Option<AppDecl> {
    if !parser.check_keyword(Keyword::App) {
        return None;
    }
    parser.advance();
    let start = parser.previous().span.start;
    let name = identifier(parser, "an application name")?;
    let mut inputs = component::parse_inputs(parser);
    if parser.consume(TokenKind::LeftBrace, "{").is_err() {
        return None;
    }
    if inputs.is_empty() {
        inputs = component::parse_inputs(parser);
    }
    let body = component::parse_component_body_items(parser);
    if parser.consume(TokenKind::RightBrace, "}").is_err() {
        return None;
    }
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    Some(AppDecl {
        name,
        inputs,
        body,
        span,
    })
}

pub fn parse_import(parser: &mut Parser) -> Vec<ImportDecl> {
    parser.advance();
    let start = parser.previous().span.start;
    let mut names = Vec::new();
    if let Some(name) = identifier(parser, "an imported name") {
        names.push(name);
        while parser.match_token(TokenKind::Comma) {
            match identifier(parser, "an imported name") {
                Some(name) => names.push(name),
                None => break,
            }
        }
    }
    if parser.consume_keyword(Keyword::From).is_err() {
        return Vec::new();
    }
    let Some(path) = string_literal(parser) else {
        return Vec::new();
    };
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    names
        .into_iter()
        .map(|name| ImportDecl {
            name,
            path: path.clone(),
            span,
        })
        .collect()
}

pub fn parse_imports(parser: &mut Parser) -> Vec<ImportDecl> {
    let mut imports = Vec::new();
    while parser.check_keyword(Keyword::Import) {
        let decls = parse_import(parser);
        if decls.is_empty() {
            break;
        }
        imports.extend(decls);
    }
    imports
}

pub fn parse_api(parser: &mut Parser) -> Option<ApiDecl> {
    parser.advance();
    let start = parser.previous().span.start;
    let name = identifier(parser, "an api name")?;
    if parser.consume(TokenKind::LeftBrace, "{").is_err() {
        return None;
    }
    let mut routes = Vec::new();
    while !parser.check(TokenKind::RightBrace) && !parser.at_end() {
        let checkpoint = parser.checkpoint();
        match parse_api_route(parser) {
            Some(route) => routes.push(route),
            None => {
                parser.advance();
                parser.restore(checkpoint);
                parser.synchronize();
                if parser.current() == checkpoint.position() {
                    break;
                }
            }
        }
    }
    if parser.consume(TokenKind::RightBrace, "}").is_err() {
        return None;
    }
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    Some(ApiDecl { name, routes, span })
}

fn parse_api_route(parser: &mut Parser) -> Option<crate::ast::ApiRoute> {
    let start = parser.peek().span.start;
    let method = api_method(parser.peek().lexeme.as_str())?;
    parser.advance();
    let name = identifier(parser, "a route name")?;
    if parser.check_keyword(Keyword::From) || parser.check_keyword(Keyword::In) {
        parser.advance();
    } else if !parser.check(TokenKind::StringLiteral) {
        return None;
    }
    let path = string_literal(parser)?;
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    Some(crate::ast::ApiRoute {
        method,
        name,
        path,
        span,
    })
}

fn api_method(text: &str) -> Option<ApiMethod> {
    match text {
        "get" => Some(ApiMethod::Get),
        "post" => Some(ApiMethod::Post),
        "put" => Some(ApiMethod::Put),
        "delete" => Some(ApiMethod::Delete),
        "patch" => Some(ApiMethod::Patch),
        _ => None,
    }
}

pub fn parse_test(parser: &mut Parser) -> Option<TestDecl> {
    parser.advance();
    let start = parser.previous().span.start;
    let name = identifier(parser, "a test name")?;
    if parser.consume(TokenKind::LeftBrace, "{").is_err() {
        return None;
    }
    let mut steps = Vec::new();
    while !parser.check(TokenKind::RightBrace) && !parser.at_end() {
        if parser.check_contextual(ContextualKeyword::Expect) {
            let span_start = parser.peek().span.start;
            parser.advance();
            let expression = expression::parse_expression(parser);
            steps.push(TestStep::Expect {
                expression,
                span: platipus_diagnostics::Span::new(span_start, parser.previous().span.end),
            });
            continue;
        }
        let span_start = parser.peek().span.start;
        let Some(action) = identifier(parser, "a test action") else {
            break;
        };
        let argument = if parser.check(TokenKind::StringLiteral) {
            string_literal(parser).map(|text| {
                crate::ast::Expression::StringLiteral(text, platipus_diagnostics::Span::empty())
            })
        } else {
            None
        };
        steps.push(TestStep::Action {
            name: action,
            argument,
            span: platipus_diagnostics::Span::new(span_start, parser.previous().span.end),
        });
    }
    if parser.consume(TokenKind::RightBrace, "}").is_err() {
        return None;
    }
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    Some(TestDecl { name, steps, span })
}

pub fn parse_tests(parser: &mut Parser) -> Vec<TestDecl> {
    let mut tests = Vec::new();
    while parser.check_keyword(Keyword::Test) {
        let Some(decl) = parse_test(parser) else {
            break;
        };
        tests.push(decl);
    }
    tests
}

pub(crate) fn identifier(parser: &mut Parser, expected: &str) -> Option<crate::ast::Identifier> {
    if !parser.check_identifier() {
        let found = parser.peek().describe();
        let span = parser.peek().span;
        parser.push_error(
            "expected-identifier",
            format!("expected {expected} but found {found}"),
            span,
        );
        return None;
    }
    let token = parser.advance();
    Some(crate::ast::Identifier::new(token.lexeme, token.span))
}

pub(crate) fn string_literal(parser: &mut Parser) -> Option<String> {
    if !parser.check(TokenKind::StringLiteral) {
        let found = parser.peek().describe();
        let span = parser.peek().span;
        parser.push_error(
            "expected-string",
            format!("expected a string literal but found {found}"),
            span,
        );
        return None;
    }
    let token = parser.advance();
    match token.literal {
        Some(crate::lexer::Literal::Str(text)) => Some(text),
        _ => Some(token.lexeme),
    }
}

use super::{component, expression, style};
