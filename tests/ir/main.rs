use platipus_compiler::ast::ComponentItem;
use platipus_compiler::ir::{
    ElementItem, ElementKind, IrComponent, IrElement, IrModule, IrStateKind, Lowering,
    StatementKind, lower_program, verify,
};
use platipus_compiler::parser::parse;

fn build(source: &str) -> IrModule {
    let outcome = parse("test.plt", source);
    assert!(
        outcome.errors.is_empty(),
        "fixture must parse cleanly, got {:?}",
        outcome
            .errors
            .iter()
            .map(|error| error.label())
            .collect::<Vec<_>>()
    );
    lower_program(&outcome.program).expect("lowering requires an app")
}

fn component<'a>(module: &'a IrModule, name: &str) -> &'a IrComponent {
    module
        .components
        .iter()
        .find(|component| component.name == name)
        .unwrap_or_else(|| panic!("component `{name}` is missing from the IR module"))
}

fn root(component: &IrComponent) -> &IrElement {
    component
        .body
        .iter()
        .find_map(|item| match item {
            ElementItem::Child(element) => Some(element),
            _ => None,
        })
        .expect("component must lower to a root element")
}

fn child<'a>(element: &'a IrElement, name: &str) -> &'a IrElement {
    element
        .body
        .as_ref()
        .expect("element must have a body")
        .items
        .iter()
        .find_map(|item| match item {
            ElementItem::Child(child) if child.name == name => Some(child),
            _ => None,
        })
        .unwrap_or_else(|| panic!("`{name}` is missing from the lowered body"))
}

const COUNTER: &str = r##"
app CounterApp {
    state count = 0
    derived doubled = count * 2

    fn increment(by: Int) {
        count += by
    }

    Column {
        Text count
        Button "+" {
            disabled: count == 0
            on click {
                increment(1)
            }
        }
    }
}
"##;

#[test]
fn lowering_produces_a_verified_module() {
    let module = build(COUNTER);
    verify(&module).expect("lowered module must verify");
}

#[test]
fn the_app_component_is_lowered_first() {
    let module = build(COUNTER);
    assert_eq!(module.name, "CounterApp");
    assert_eq!(module.components[0].name, "CounterApp");
}

#[test]
fn state_kinds_are_preserved() {
    let module = build(COUNTER);
    let app = component(&module, "CounterApp");
    assert_eq!(app.states.len(), 1);
    assert_eq!(app.states[0].name, "count");
    assert_eq!(app.states[0].kind, IrStateKind::Local);
    assert_eq!(
        app.states[0].initializer.as_ref().map(|e| e.value.as_str()),
        Some("0")
    );
}

#[test]
fn state_scopes_reach_the_ir() {
    let module = build(
        r##"
app Main {
    state local = 1
    shared state sharedValue = 2
    global state globalValue = 3
    persistent state saved = 4
    Column { }
}
"##,
    );
    let app = component(&module, "Main");
    let kinds: Vec<(&str, IrStateKind)> = app
        .states
        .iter()
        .map(|state| (state.name.as_str(), state.kind))
        .collect();
    assert_eq!(
        kinds,
        vec![
            ("local", IrStateKind::Local),
            ("sharedValue", IrStateKind::Shared),
            ("globalValue", IrStateKind::Global),
            ("saved", IrStateKind::Persistent),
        ]
    );
}

#[test]
fn derived_values_record_their_dependencies() {
    let module = build(COUNTER);
    let app = component(&module, "CounterApp");
    assert_eq!(app.derived.len(), 1);
    let doubled = &app.derived[0];
    assert_eq!(doubled.name, "doubled");
    assert_eq!(doubled.value, "count * 2");
    assert_eq!(doubled.dependencies, vec!["count".to_string()]);
}

#[test]
fn functions_keep_their_parameters_and_body() {
    let module = build(COUNTER);
    let app = component(&module, "CounterApp");
    assert_eq!(app.functions.len(), 1);
    let increment = &app.functions[0];
    assert_eq!(increment.name, "increment");
    assert_eq!(increment.parameters.len(), 1);
    assert_eq!(increment.parameters[0].name, "by");
    assert_eq!(increment.parameters[0].type_name.as_deref(), Some("Int"));
    match &increment.body[0].kind {
        StatementKind::Assign {
            target,
            operator,
            value,
        } => {
            assert_eq!(target, "count");
            assert_eq!(operator, "+=");
            assert_eq!(value, "by");
        }
        other => panic!("expected an assignment, got {other:?}"),
    }
}

#[test]
fn primitive_elements_are_typed_as_primitives() {
    let module = build(COUNTER);
    let app = component(&module, "CounterApp");
    let column = root(app);
    assert_eq!(column.name, "Column");
    assert_eq!(column.kind, ElementKind::Primitive);
    assert_eq!(child(column, "Text").kind, ElementKind::Primitive);
    assert_eq!(child(column, "Button").kind, ElementKind::Primitive);
}

#[test]
fn declared_components_are_typed_as_components() {
    let module = build(
        r##"
component UserCard {
    Text "card"
}
app Main {
    Column {
        UserCard name: "world"
    }
}
"##,
    );
    let app = component(&module, "Main");
    let column = root(app);
    let card = child(column, "UserCard");
    assert_eq!(card.kind, ElementKind::Component);
    assert_eq!(
        card.properties
            .iter()
            .map(|property| property.name.as_str())
            .collect::<Vec<_>>(),
        vec!["name"]
    );
}

#[test]
fn unknown_pascal_case_names_are_not_treated_as_components() {
    let module = build(
        r##"
app Main {
    Column { }
}
"##,
    );
    let lowering = Lowering::new(Vec::new());
    assert!(!lowering.is_component("Column"));
    assert!(!lowering.is_component("Unknown"));
    assert!(module.components.len() == 1);
}

#[test]
fn element_arguments_and_properties_collect_dependencies() {
    let module = build(COUNTER);
    let app = component(&module, "CounterApp");
    let button = child(root(app), "Button");
    let disabled = button
        .properties
        .iter()
        .find(|property| property.name == "disabled")
        .expect("disabled property");
    assert_eq!(disabled.value.value, "count == 0");
    assert_eq!(disabled.value.dependencies, vec!["count".to_string()]);
}

#[test]
fn text_content_is_lowered_as_a_reactive_expression() {
    let module = build(COUNTER);
    let app = component(&module, "CounterApp");
    let text = child(root(app), "Text");
    let content = text.text.as_ref().expect("Text lowers its content");
    assert_eq!(content.value, "count");
    assert_eq!(content.dependencies, vec!["count".to_string()]);
}

#[test]
fn event_handlers_keep_their_category() {
    let module = build(COUNTER);
    let app = component(&module, "CounterApp");
    let button = child(root(app), "Button");
    assert_eq!(button.handlers.len(), 1);
    let handler = &button.handlers[0];
    assert_eq!(handler.event, "click");
    assert_eq!(handler.category.as_str(), "pointer");
    assert!(!handler.is_custom);
}

#[test]
fn custom_event_handlers_survive_lowering() {
    let module = build(
        r##"
component UserCard {
    fn pick() {
        emit selected(1)
    }
    Button "pick" {
        on click {
            pick()
        }
    }
}
app Main {
    UserCard {
        on selected {
            choose(1)
        }
    }
    fn choose(value: Int) { }
}
"##,
    );
    let card = component(&module, "UserCard");
    assert_eq!(card.emits, vec!["selected".to_string()]);
    let main = component(&module, "Main");
    let card_element = root(main);
    assert_eq!(card_element.handlers.len(), 1);
    let handler = &card_element.handlers[0];
    assert_eq!(handler.event, "selected");
    assert!(handler.is_custom);
    assert_eq!(handler.category.as_str(), "custom");
}

#[test]
fn lifecycle_handlers_are_kept() {
    let module = build(
        r##"
app Main {
    Column {
        on mount {
            count = 1
        }
    }
    state count = 0
}
"##,
    );
    let app = component(&module, "Main");
    let column = root(app);
    assert_eq!(column.handlers[0].event, "mount");
    assert_eq!(column.handlers[0].category.as_str(), "lifecycle");
}

#[test]
fn control_flow_lowers_into_nested_statements() {
    let module = build(
        r##"
app Main {
    state items = []
    fn run() {
        for item in items {
            if item == 1 {
                continue
            }
            break
        }
    }
    Column { }
}
"##,
    );
    let app = component(&module, "Main");
    let run = &app.functions[0];
    let StatementKind::For {
        binding,
        iterable,
        body,
    } = &run.body[0].kind
    else {
        panic!("expected a loop, got {:?}", run.body[0].kind);
    };
    assert_eq!(binding, "item");
    assert_eq!(iterable, "items");
    let StatementKind::If {
        then_branch,
        else_branch,
        ..
    } = &body[0].kind
    else {
        panic!("expected a conditional, got {:?}", body[0].kind);
    };
    assert!(else_branch.is_none());
    assert!(matches!(then_branch[0].kind, StatementKind::Continue));
    assert!(matches!(body[1].kind, StatementKind::Break));
}

#[test]
fn try_lowers_both_branches() {
    let module = build(
        r##"
app Main {
    fn run() {
        try {
            load()
        } catch error {
            report(error)
        }
    }
    Column { }
}
"##,
    );
    let app = component(&module, "Main");
    let run = &app.functions[0];
    let StatementKind::Try {
        binding: _,
        body,
        handler,
    } = &run.body[0].kind
    else {
        panic!("expected try, got {:?}", run.body[0].kind);
    };
    assert_eq!(body.len(), 1);
    assert_eq!(handler.len(), 1);
}

#[test]
fn emit_lowers_with_its_payload() {
    let module = build(
        r##"
component Card {
    fn pick() {
        emit selected(1)
    }
    Column { }
}
app Main {
    Card {}
}
"##,
    );
    let card = component(&module, "Card");
    let pick = &card.functions[0];
    let StatementKind::Emit { event, payload } = &pick.body[0].kind else {
        panic!("expected emit, got {:?}", pick.body[0].kind);
    };
    assert_eq!(event, "selected");
    assert_eq!(payload.as_deref(), Some("1"));
}

#[test]
fn style_blocks_lower_with_state_groups() {
    let module = build(
        r##"
app Main {
    Column {
        style {
            background: "#000"
            padding: 16
            hover {
                background: "#111"
            }
        }
    }
}
"##,
    );
    let app = component(&module, "Main");
    let column = root(app);
    let style = column
        .body
        .as_ref()
        .unwrap()
        .items
        .iter()
        .find_map(|item| match item {
            ElementItem::Style(block) => Some(block),
            _ => None,
        })
        .expect("style block");
    assert_eq!(style.entries.len(), 2);
    assert_eq!(style.states.len(), 1);
    assert_eq!(style.states[0].0.as_str(), "hover");
    assert_eq!(style.states[0].1.entries[0].value, "#111");
}

#[test]
fn api_routes_are_lowered() {
    let module = build(
        r##"
api Notes {
    get all "/notes"
    post create "/notes"
}
app Main {
    Column { }
}
"##,
    );
    assert_eq!(module.apis.len(), 1);
    let notes = &module.apis[0];
    assert_eq!(notes.routes.len(), 2);
    assert_eq!(notes.routes[0].name, "all");
    assert_eq!(notes.routes[0].path, "/notes");
}

#[test]
fn the_verifier_rejects_derived_assignment() {
    let module = build(
        r##"
app Main {
    state count = 0
    derived doubled = count * 2
    fn bump() {
        doubled = 3
    }
    Column { }
}
"##,
    );
    let errors = verify(&module).expect_err("derived assignment must not verify");
    assert!(
        errors.iter().any(|error| error.code == "not-assignable"),
        "got {errors:?}"
    );
}

#[test]
fn the_verifier_rejects_input_assignment() {
    let module = build(
        r##"
component Greeting {
    input name: String
    fn bump() {
        name = "other"
    }
    Text name
}
app Main {
    Greeting name: "x"
}
"##,
    );
    let errors = verify(&module).expect_err("input assignment must not verify");
    assert!(
        errors.iter().any(|error| error.code == "not-assignable"),
        "got {errors:?}"
    );
}

#[test]
fn the_verifier_does_not_allow_assigning_another_functions_parameter() {
    let module = build(
        r##"
app Main {
    fn first() {
        other = 1
    }
    fn second(other: Int) { }
    Column { }
}
"##,
    );
    let errors = verify(&module).expect_err("a parameter is writable only in its own function");
    assert!(
        errors.iter().any(|error| error.code == "not-assignable"),
        "got {errors:?}"
    );
}

#[test]
fn the_verifier_reports_empty_handlers() {
    let module = build(
        r##"
app Main {
    Column {
        on click { }
    }
}
"##,
    );
    let errors = verify(&module).expect_err("an empty handler must not verify");
    assert!(
        errors.iter().any(|error| error.code == "empty-handler"),
        "got {errors:?}"
    );
}

#[test]
fn the_verifier_reports_a_module_without_an_app() {
    let outcome = parse("test.plt", "component Lonely { Column { } }");
    assert!(outcome.errors.is_empty());
    assert!(lower_program(&outcome.program).is_none());
}

#[test]
fn styles_are_hoisted_to_the_module() {
    let outcome = parse(
        "test.plt",
        r##"
style card {
    color: "#fff"
}
app Main {
    Column { }
}
"##,
    );
    assert!(
        outcome.errors.is_empty(),
        "got {:?}",
        outcome
            .errors
            .iter()
            .map(|error| error.label())
            .collect::<Vec<_>>()
    );
    let module = lower_program(&outcome.program).expect("app lowers");
    assert_eq!(module.styles.len(), 1);
    assert_eq!(module.styles[0].entries[0].name, "color");
}

#[test]
fn app_roots_are_preserved_in_order() {
    let module = build(
        r##"
app Main {
    state ready = true
    Column {
        Text "a"
    }
}
"##,
    );
    let app = component(&module, "Main");
    let statements = app
        .body
        .iter()
        .filter(|item| matches!(item, ElementItem::Statement(_)))
        .count();
    assert_eq!(statements, 0);
    assert!(matches!(app.body[0], ElementItem::Child(_)));
    let _ = ComponentItem::State;
}
