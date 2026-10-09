const COUNTER: &str = r##"
app CounterApp {

    state count = 0
    derived doubled = count * 2

    fn increment(by: Int) {
        count += by
    }

    Column {

        style {
            background: "#0f172a"
            padding: 16

            hover {
                background: "#1e293b"
            }
        }

        Text count
        Text doubled

        Button "+" {
            disabled: count == 0
            on click {
                increment(1)
            }
        }

        for user in users {
            UserCard user: user {
                on selected {
                    current = event.value
                }
            }
        }

        if count > 10 {
            Text "many"
        } else {
            Text "few"
        }
    }
}
"##;

#[test]
fn parses_counter_application() {
    let outcome = platipus_compiler::parser::parse("counter.plt", COUNTER);
    assert!(
        outcome.errors.is_empty(),
        "unexpected parse errors: {:?}",
        outcome.errors.iter().map(|e| e.label()).collect::<Vec<_>>()
    );
    let app = outcome.program.app.expect("app declaration");
    assert_eq!(app.name.as_str(), "CounterApp");
    assert!(
        app.body
            .iter()
            .any(|item| matches!(item, platipus_compiler::ast::ComponentItem::State(_)))
    );
    assert!(
        app.body
            .iter()
            .any(|item| matches!(item, platipus_compiler::ast::ComponentItem::Derived(_)))
    );
    assert!(
        app.body
            .iter()
            .any(|item| matches!(item, platipus_compiler::ast::ComponentItem::Function(_)))
    );
    assert!(
        app.body
            .iter()
            .any(|item| matches!(item, platipus_compiler::ast::ComponentItem::Child(_)))
    );
}

fn parse_errors(source: &str) -> Vec<String> {
    let outcome = platipus_compiler::parser::parse("test.plt", source);
    outcome
        .errors
        .iter()
        .map(|error| error.code.to_string())
        .collect()
}

#[test]
fn the_three_canonical_breakpoints_are_accepted() {
    for name in ["mobile", "tablet", "desktop"] {
        let errors = parse_errors(&format!(
            "app Main {{ Column {{ responsive {{ {name}: padding: 8 }} }} }}"
        ));
        assert!(
            errors.is_empty(),
            "`{name}` should be a breakpoint: {errors:?}"
        );
    }
}

#[test]
fn any_other_breakpoint_name_is_rejected() {
    for name in ["md", "sm", "lg", "xl", "Small", "Medium"] {
        let errors = parse_errors(&format!(
            "app Main {{ Column {{ responsive {{ {name}: padding: 8 }} }} }}"
        ));
        assert!(
            errors.iter().any(|code| code == "unknown-breakpoint"),
            "`{name}` is not a breakpoint: {errors:?}"
        );
    }
}

#[test]
fn a_responsive_element_swap_does_not_parse() {
    // Elements no longer change with the viewport, so the old spelling has to
    // fail loudly instead of quietly becoming a named style.
    for name in ["mobile", "tablet", "desktop"] {
        let errors = parse_errors(&format!("app Main {{ {name}: Column {{ }} }}"));
        assert!(
            !errors.is_empty(),
            "`{name}: Column` should not parse: {errors:?}"
        );
    }
}

#[test]
fn parses_a_rust_block_into_its_own_ast_node() {
    let source = "#[rust]\nfn foo() {}\n\napp Main { Column { Text \"hi\" } }\n";
    let outcome = platipus_compiler::parser::parse("test.plt", source);
    assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
    assert_eq!(outcome.program.rust_blocks.len(), 1);
    assert_eq!(outcome.program.rust_blocks[0].attributes, vec!["rust".to_string()]);
    assert!(outcome.program.rust_blocks[0].source.contains("fn foo()"));
}

#[test]
fn a_rust_block_can_appear_before_the_app() {
    let source = "#[rust]\n#[export]\nfn add(a: i64, b: i64) -> i64 {\n    a + b\n}\n\napp Main { Column { Text add(1, 2) } }\n";
    let outcome = platipus_compiler::parser::parse("test.plt", source);
    assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
    assert_eq!(outcome.program.rust_blocks.len(), 1);
    assert!(
        outcome.program.rust_blocks[0]
            .attributes
            .iter()
            .any(|a| a == "export")
    );
}

#[test]
fn rust_blocks_are_opaque_to_the_parser() {
    // Braces and strings inside Rust never confuse the Platipus parser.
    let source = "#[rust]\nfn f() {\n    let s = \"}\"; // comment\n}\n\napp Main { Column { } }\n";
    let outcome = platipus_compiler::parser::parse("test.plt", source);
    assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
    assert_eq!(outcome.program.rust_blocks.len(), 1);
}

#[test]
fn a_bare_component_is_not_greedy_with_the_next_sibling() {
    let source = "component C {\n    Card { padding: 4 }\n}\n\napp M {\n    Row {\n        gap: 8\n        C\n        Card { padding: 4 }\n        Text \"hi\"\n    }\n}\n";
    let outcome = platipus_compiler::parser::parse("main.plt", source);
    assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
}

/// Renders a fully-parenthesised view of an expression so precedence bugs are
/// visible as text instead of nested `Debug` dumps.
fn render(expression: &platipus_compiler::ast::Expression) -> String {
    use platipus_compiler::ast::{Expression, LogicalOp};
    match expression {
        Expression::IntLiteral(value, _) => value.to_string(),
        Expression::FloatLiteral(value, _) => value.to_string(),
        Expression::BoolLiteral(value, _) => value.to_string(),
        Expression::Identifier(name) => name.name.clone(),
        Expression::Unary { op, operand, .. } => format!("({}{})", op.symbol(), render(operand)),
        Expression::Binary {
            op, left, right, ..
        } => format!("({} {} {})", render(left), op.symbol(), render(right)),
        Expression::Logical {
            op, left, right, ..
        } => {
            let symbol = match op {
                LogicalOp::And => "&&",
                LogicalOp::Or => "||",
            };
            format!("({} {} {})", render(left), symbol, render(right))
        }
        other => format!("<{other:?}>"),
    }
}

fn render_state(source: &str) -> String {
    let outcome = platipus_compiler::parser::parse("test.plt", source);
    assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
    let app = outcome.program.app.expect("app");
    match &app.body[0] {
        platipus_compiler::ast::ComponentItem::State(decl) => {
            render(decl.initializer.as_ref().expect("state initializer"))
        }
        other => panic!("expected a state declaration, got {other:?}"),
    }
}

#[test]
fn multiplicative_binds_tighter_than_additive() {
    assert_eq!(render_state("app A { state x = 1 + 2 * 3 }"), "(1 + (2 * 3))");
    assert_eq!(render_state("app A { state x = 1 * 2 + 3 }"), "((1 * 2) + 3)");
    assert_eq!(render_state("app A { state x = 1 - 2 - 3 }"), "((1 - 2) - 3)");
    assert_eq!(render_state("app A { state x = 12 / 3 % 4 }"), "((12 / 3) % 4)");
}

#[test]
fn comparison_binds_tighter_than_equality() {
    assert_eq!(
        render_state("app A { state x = 1 + 2 == 3 }"),
        "((1 + 2) == 3)"
    );
    assert_eq!(
        render_state("app A { state x = 1 == 2 < 3 }"),
        "(1 == (2 < 3))"
    );
}

#[test]
fn logical_operators_keep_boolean_precedence() {
    assert_eq!(
        render_state("app A { state x = true || false && false }"),
        "(true || (false && false))"
    );
    assert_eq!(
        render_state("app A { state x = a && b || c }"),
        "((a && b) || c)"
    );
}

#[test]
fn unary_minus_stays_glued_to_its_operand() {
    assert_eq!(render_state("app A { state x = -2 + 3 }"), "((-2) + 3)");
    assert_eq!(render_state("app A { state x = -(2 + 3) }"), "(-(2 + 3))");
}
fn function_statements(source: &str) -> Vec<platipus_compiler::ast::Statement> {
    let outcome = platipus_compiler::parser::parse("test.plt", source);
    assert!(
        outcome.errors.is_empty(),
        "{:?}",
        outcome.errors.iter().map(|error| error.label()).collect::<Vec<_>>()
    );
    let app = outcome.program.app.expect("app declaration");
    for item in &app.body {
        if let platipus_compiler::ast::ComponentItem::Function(decl) = item {
            return decl.body.statements.clone();
        }
    }
    panic!("expected a function in the app body");
}

#[test]
fn parses_let_with_and_without_an_annotation() {
    let statements =
        function_statements("app A { fn f() { let x = 1 let total: Int = 0 } Column { } }");
    assert_eq!(statements.len(), 2);
    match &statements[0] {
        platipus_compiler::ast::Statement::Let(decl) => {
            assert_eq!(decl.name.as_str(), "x");
            assert!(decl.annotation.is_none());
            assert!(
                matches!(
                    decl.initializer,
                    platipus_compiler::ast::Expression::IntLiteral(1, _)
                ),
                "got {:?}",
                decl.initializer
            );
        }
        other => panic!("expected a `let`, got {other:?}"),
    }
    match &statements[1] {
        platipus_compiler::ast::Statement::Let(decl) => {
            let annotation = decl.annotation.as_ref().expect("annotation");
            assert_eq!(annotation.base_name(), "Int");
        }
        other => panic!("expected an annotated `let`, got {other:?}"),
    }
}

#[test]
fn let_requires_an_initializer() {
    let errors = parse_errors("app A { fn f() { let x } Column { } }");
    assert!(
        errors.iter().any(|code| code == "missing-initializer"),
        "{errors:?}"
    );
}

#[test]
fn parses_while_as_a_statement_with_a_block() {
    let statements = function_statements("app A { fn f() { while x < 3 { x += 1 } } Column { } }");
    match &statements[0] {
        platipus_compiler::ast::Statement::While(decl) => {
            assert!(
                matches!(
                    decl.condition,
                    platipus_compiler::ast::Expression::Binary { .. }
                ),
                "got {:?}",
                decl.condition
            );
            assert_eq!(decl.body.statements.len(), 1);
        }
        other => panic!("expected a `while`, got {other:?}"),
    }
}

#[test]
fn parses_a_counted_range_in_a_for_header() {
    let statements = function_statements(
        "app A { fn f() { for i in 0..10 { } for j in 0..=3 { } for item in items { } } Column { } }",
    );
    assert_eq!(statements.len(), 3);
    let ranges: Vec<platipus_compiler::ast::ForIterable> = statements
        .iter()
        .map(|statement| match statement {
            platipus_compiler::ast::Statement::For(for_statement) => {
                for_statement.iterable.clone()
            }
            other => panic!("expected a `for`, got {other:?}"),
        })
        .collect();
    match &ranges[0] {
        platipus_compiler::ast::ForIterable::Range {
            start,
            end,
            inclusive,
            ..
        } => {
            assert!(matches!(start, platipus_compiler::ast::Expression::IntLiteral(0, _)));
            assert!(matches!(end, platipus_compiler::ast::Expression::IntLiteral(10, _)));
            assert!(!inclusive, "`0..10` is exclusive");
        }
        other => panic!("expected a range, got {other:?}"),
    }
    match &ranges[1] {
        platipus_compiler::ast::ForIterable::Range { inclusive, .. } => {
            assert!(inclusive, "`0..=3` is inclusive");
        }
        other => panic!("expected a range, got {other:?}"),
    }
    match &ranges[2] {
        platipus_compiler::ast::ForIterable::Value(expression) => {
            assert!(
                matches!(expression, platipus_compiler::ast::Expression::Identifier(_)),
                "got {expression:?}"
            );
        }
        other => panic!("expected a plain iterable, got {other:?}"),
    }
}

#[test]
fn a_range_outside_a_for_header_is_rejected() {
    for source in [
        "app A { state x = 0..1 Column { } }",
        "app A { fn f() { let y = 0..1 } Column { } }",
        "app A { fn f() { count(0..1) } Column { } }",
    ] {
        let errors = parse_errors(source);
        assert!(
            errors.iter().any(|code| code == "range-outside-for"),
            "{source}: {errors:?}"
        );
    }
}

fn lambda_expression(source: &str) -> platipus_compiler::ast::Expression {
    let statements = function_statements(source);
    match &statements[0] {
        platipus_compiler::ast::Statement::Let(decl) => decl.initializer.clone(),
        other => panic!("expected a `let`, got {other:?}"),
    }
}

#[test]
fn parses_a_lambda_with_one_parameter() {
    match lambda_expression("app A { fn f() { let g = x => x * 2 } Column { } }") {
        platipus_compiler::ast::Expression::Lambda { parameters, .. } => {
            assert_eq!(parameters.len(), 1);
            assert_eq!(parameters[0].name, "x");
        }
        other => panic!("expected a lambda, got {other:?}"),
    }
}

#[test]
fn parses_a_lambda_with_multiple_parameters() {
    match lambda_expression("app A { fn f() { let g = (a, b) => a + b } Column { } }") {
        platipus_compiler::ast::Expression::Lambda { parameters, .. } => {
            assert_eq!(parameters.len(), 2);
            assert_eq!(parameters[1].name, "b");
        }
        other => panic!("expected a lambda, got {other:?}"),
    }
}

#[test]
fn parses_a_lambda_with_no_parameters() {
    match lambda_expression("app A { fn f() { let g = () => 42 } Column { } }") {
        platipus_compiler::ast::Expression::Lambda { parameters, .. } => {
            assert!(parameters.is_empty());
        }
        other => panic!("expected a lambda, got {other:?}"),
    }
}

#[test]
fn parses_a_lambda_with_a_block_body() {
    match lambda_expression(
        "app A { fn f() { let g = x => { return x } } Column { } }",
    ) {
        platipus_compiler::ast::Expression::Lambda { body, .. } => {
            assert!(matches!(
                body,
                platipus_compiler::ast::LambdaBody::Block(_)
            ));
        }
        other => panic!("expected a lambda, got {other:?}"),
    }
}
