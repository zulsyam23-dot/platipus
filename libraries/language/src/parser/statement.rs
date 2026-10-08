use super::Parser;
use crate::ast::{
    Block, ElseBranch, ForIterable, ForStatement, IfStatement, LetStatement, ReturnStatement,
    Statement, TryStatement, WhileStatement,
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
            "while" => return Some(Statement::While(parse_while(parser))),
            "let" => return Some(Statement::Let(parse_let(parser))),
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
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
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
        .unwrap_or_else(|| crate::ast::Identifier::new("", platipus_diagnostics::Span::empty()));
    if parser.consume_keyword(Keyword::In).is_err() {
        parser.push_error_with_help(
            "malformed-loop",
            format!("`for {binding} in ...` requires the `in` keyword"),
            parser.peek().span,
            format!("write `for {binding} in items {{ ... }}`"),
        );
    }
    let start_expression = super::expression::parse_expression_raw(parser);
    let start_span = start_expression.span();
    let mut end = None;
    let mut inclusive = false;
    // `..` belongs to the `for` header only; every other place reports it as
    // `range-outside-for` from `parse_expression`.
    while matches!(
        parser.peek().kind,
        TokenKind::DotDot | TokenKind::DotDotEqual
    ) {
        let operator = parser.advance();
        if end.is_some() {
            parser.push_error_with_help(
                "malformed-loop",
                "`for` ranges accept a single `..`",
                operator.span,
                format!("write `for {binding} in start..end {{ ... }}`"),
            );
        }
        inclusive = operator.kind == TokenKind::DotDotEqual;
        end = Some(super::expression::parse_expression_raw(parser));
    }
    let iterable = match end {
        Some(end) => ForIterable::Range {
            span: platipus_diagnostics::Span::new(start_span.start, end.span().end),
            start: start_expression,
            end,
            inclusive,
        },
        None => ForIterable::Value(start_expression),
    };
    let body = parse_block(parser);
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    ForStatement {
        binding,
        iterable,
        body,
        span,
    }
}

fn parse_let(parser: &mut Parser) -> LetStatement {
    let start = parser.advance().span.start;
    let name = identifier(parser, "a variable name")
        .unwrap_or_else(|| crate::ast::Identifier::new("", platipus_diagnostics::Span::empty()));
    let annotation = if parser.check(TokenKind::Colon) {
        parser.advance();
        Some(super::expression::parse_type(parser))
    } else {
        None
    };
    if !parser.check(TokenKind::Assign) {
        let span = parser.peek().span;
        parser.push_error_with_help(
            "missing-initializer",
            "`let` requires an initializer",
            span,
            format!("write `let {} = value`", name.name),
        );
        let span = platipus_diagnostics::Span::new(start, span.end);
        return LetStatement {
            name,
            annotation,
            initializer: crate::ast::Expression::NullLiteral(span),
            span,
        };
    }
    parser.advance();
    let initializer = super::expression::parse_expression(parser);
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    LetStatement {
        name,
        annotation,
        initializer,
        span,
    }
}

fn parse_while(parser: &mut Parser) -> WhileStatement {
    let start = parser.advance().span.start;
    let condition = super::expression::parse_expression(parser);
    let body = parse_block(parser);
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    WhileStatement {
        condition,
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
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
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
        Block::new(Vec::new(), platipus_diagnostics::Span::empty())
    };
    let span = platipus_diagnostics::Span::new(start, parser.previous().span.end);
    TryStatement {
        body,
        binding,
        handler,
        span,
    }
}
