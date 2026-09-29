use super::Parser;
use crate::ast::{
    Block, ElseBranch, ForStatement, IfStatement, ReturnStatement, Statement, TryStatement,
};
use crate::lexer::{Keyword, TokenKind};

use super::app::identifier;

pub fn parse_block(parser: &mut Parser) -> Block {
    super::event::parse_block(parser)
}

pub fn try_parse_statement(parser: &mut Parser) -> Option<Statement> {
    if parser.check(TokenKind::Keyword) {
        let keyword = parser.peek().lexeme.as_str();
        match keyword {
            "state" | "shared" | "global" | "persistent" | "derived" => {
                return super::state::try_parse_state_item(parser).map(|item| match item {
                    crate::ast::ComponentItem::State(decl) => Statement::State(decl),
                    crate::ast::ComponentItem::Derived(decl) => Statement::Derived(decl),
                    _ => unreachable!("state parser only produces state or derived"),
                });
            }
            "fn" | "async" => {
                return super::function::try_parse_function(parser).map(Statement::Function);
            }
            "on" => {
                parser.advance();
                return Some(Statement::Handler(super::event::parse_handler_rest(parser)));
            }
            "emit" => return super::event::try_parse_emit(parser).map(Statement::Emit),
            "import" => {
                let mut imports = super::app::parse_imports(parser);
                return imports.pop().map(Statement::Import);
            }
            "test" => {
                let mut tests = super::app::parse_tests(parser);
                return tests.pop().map(Statement::Test);
            }
            "if" => return Some(Statement::If(parse_if(parser))),
            "for" => return Some(Statement::For(parse_for(parser))),
            "return" => return Some(Statement::Return(parse_return(parser))),
            "try" => return Some(Statement::Try(parse_try(parser))),
            "break" => {
                let span = parser.advance().span;
                return Some(Statement::Break(span));
            }
            "continue" => {
                let span = parser.advance().span;
                return Some(Statement::Continue(span));
            }
            "let" => {
                let span = parser.peek().span;
                parser.push_error_with_help(
                    "forbidden-declaration",
                    "`let` is not part of the language",
                    span,
                    "declare reactive state with `state name = value`",
                );
                parser.advance();
                return Some(Statement::Empty(span));
            }
            "const" | "var" | "signal" | "reactive" => {
                let span = parser.peek().span;
                let keyword = parser.advance().lexeme;
                parser.push_error_with_help(
                    "forbidden-declaration",
                    format!("`{keyword}` is not part of the language"),
                    span,
                    "declare reactive state with `state name = value`",
                );
                return Some(Statement::Empty(span));
            }
            _ => {}
        }
    }
    if parser.check(TokenKind::LeftBrace) {
        return Some(Statement::Block(parse_block(parser)));
    }
    if let Some(element) = super::element::try_parse_element(parser) {
        return Some(Statement::Element(element));
    }
    let checkpoint = parser.checkpoint();
    if let Some(assignment) = super::expression::try_parse_assignment(parser) {
        return Some(Statement::Assignment(assignment));
    }
    parser.restore(checkpoint);
    let expression = super::expression::parse_expression(parser);
    Some(Statement::Expression(expression))
}

fn parse_if(parser: &mut Parser) -> IfStatement {
    let start = parser.advance().span.start;
    let condition = super::expression::parse_expression(parser);
    let then_branch = parse_block(parser);
    let mut else_branch = None;
    if parser.check_keyword(Keyword::Else) {
        parser.advance();
        let inner = if parser.check_keyword(Keyword::If) {
            Box::new(ElseBranch::If(parse_if(parser)))
        } else {
            Box::new(ElseBranch::Block(parse_block(parser)))
        };
        else_branch = Some(inner);
    }
    let span = crate::diagnostics::Span::new(start, parser.previous().span.end);
    IfStatement {
        condition,
        then_branch,
        else_branch,
        span,
    }
}

fn parse_for(parser: &mut Parser) -> ForStatement {
    let start = parser.advance().span.start;
    let binding = identifier(parser, "a loop variable")
        .unwrap_or_else(|| crate::ast::Identifier::new("", crate::diagnostics::Span::empty()));
    if parser.consume_keyword(Keyword::In).is_err() {
        parser.push_error_with_help(
            "malformed-loop",
            format!("`for {binding} in ...` requires the `in` keyword"),
            parser.peek().span,
            format!("write `for {binding} in items {{ ... }}`"),
        );
    }
    let iterable = super::expression::parse_expression(parser);
    let body = parse_block(parser);
    let span = crate::diagnostics::Span::new(start, parser.previous().span.end);
    ForStatement {
        binding,
        iterable,
        body,
        span,
    }
}

fn parse_return(parser: &mut Parser) -> ReturnStatement {
    let start = parser.advance().span.start;
    let value = if parser.check(TokenKind::RightBrace) {
        None
    } else {
        Some(super::expression::parse_expression(parser))
    };
    let span = crate::diagnostics::Span::new(start, parser.previous().span.end);
    ReturnStatement { value, span }
}

fn parse_try(parser: &mut Parser) -> TryStatement {
    let start = parser.advance().span.start;
    let body = parse_block(parser);
    let mut binding = None;
    let handler = if parser.check_keyword(Keyword::Catch) {
        parser.advance();
        if parser.check_identifier() {
            binding = identifier(parser, "an error binding");
        }
        parse_block(parser)
    } else {
        parser.push_error_with_help(
            "missing-catch",
            "`try` must be followed by `catch`",
            parser.peek().span,
            "write `try { ... } catch error { ... }`",
        );
        Block::new(Vec::new(), crate::diagnostics::Span::empty())
    };
    let span = crate::diagnostics::Span::new(start, parser.previous().span.end);
    TryStatement {
        body,
        binding,
        handler,
        span,
    }
}
