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
