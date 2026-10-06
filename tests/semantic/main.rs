use platipus_compiler::diagnostics::DiagnosticBag;
use platipus_compiler::parser::parse;
use platipus_compiler::semantic::SemanticChecker;

fn analyze(source: &str) -> DiagnosticBag {
    let outcome = parse("test.plt", source);
    assert!(
        outcome.errors.is_empty(),
        "fixture must parse cleanly before semantic analysis, got {:?}",
        outcome
            .errors
            .iter()
            .map(|error| error.label())
            .collect::<Vec<_>>()
    );
    SemanticChecker::new().check(&outcome.program)
}

fn codes(bag: &DiagnosticBag) -> Vec<&str> {
    let mut codes: Vec<&str> = bag.errors().map(|error| error.code).collect();
    codes.sort_unstable();
    codes.dedup();
    codes
}

fn assert_clean(source: &str) {
    let bag = analyze(source);
    let messages: Vec<String> = bag
        .errors()
        .map(|error| format!("{}: {}", error.code, error.message))
        .collect();
    assert!(
        !bag.has_errors(),
        "expected no semantic errors, got {:?}",
        messages
    );
}

fn assert_reports(source: &str, expected: &str) {
    let bag = analyze(source);
    let codes = codes(&bag);
    assert!(
        codes.contains(&expected),
        "expected diagnostic `{expected}`, got {codes:?}"
    );
}

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
        }

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
fn accepts_a_well_formed_application() {
    assert_clean(COUNTER);
}

#[test]
fn accepts_known_elements_and_properties() {
    assert_clean(
        r##"
app Main {
    Column {
        Button "Save" {
            disabled: false
        }
        Image src: "logo.png"
    }
}
"##,
    );
}

#[test]
fn accepts_every_known_event() {
    for event in platipus_compiler::ast::known::ALL {
        let source = format!(
            r##"
app Main {{
    fn nothing() {{}}
    Button "x" {{
        on {event} {{
            nothing()
        }}
    }}
}}
"##
        );
        assert_clean(&source);
    }
}

#[test]
fn rejects_unknown_element() {
    assert_reports(
        r##"
app Main {
    Buton "Save" { }
}
"##,
        "unknown-element",
    );
}

#[test]
fn suggests_a_close_element_name() {
    let bag = analyze(
        r##"
app Main {
    Buton "Save" { }
}
"##,
    );
    let error = bag.errors().find(|e| e.code == "unknown-element").unwrap();
    let note = error
        .notes
        .iter()
        .find(|note| note.message.contains("Button"))
        .unwrap_or_else(|| panic!("expected a `Button` suggestion, got {:?}", error.notes));
    assert!(note.message.contains("Button"));
}

#[test]
fn catalog_primitives_are_registered() {
    use platipus_compiler::semantic::ElementRegistry;
    let registry = ElementRegistry::new();
    for name in [
        "Panel",
        "Splitter",
        "Spacer",
        "Viewport",
        "Heading",
        "Date",
        "Time",
        "File",
        "Color",
        "ContextMenu",
        "Command",
        "Tree",
        "DataGrid",
        "Dialog",
        "Popover",
        "Navigation",
        "Sidebar",
        "Toolbar",
        "Editor",
        "CodeEditor",
        "TableRow",
        "Cell",
        "Header",
    ] {
        assert!(registry.contains(name), "missing primitive `{name}`");
    }
}

#[test]
fn catalog_covers_the_documented_builtin_surface() {
    use platipus_compiler::semantic::ElementRegistry;
    use std::collections::BTreeSet;
    let registry = ElementRegistry::new();
    assert_eq!(registry.all().len(), 62);
    let mut properties: BTreeSet<&str> = BTreeSet::new();
    for definition in registry.all() {
        for property in &definition.properties {
            properties.insert(&property.name);
        }
    }
    assert_eq!(properties.len(), 49);
}

#[test]
fn accepts_the_full_primitive_catalog() {
    assert_clean(
        r##"
app Main {
    Container {
        Heading "Title" { level: 2 size: 20 weight: 700 }
        Panel { padding: 8 }
        Splitter {
            Spacer {}
            Spacer {}
        }
        Viewport {
            Date value: "2026-01-01"
            Time value: "09:30"
            File accept: ".pdf" multiple: true
            Color value: "#ff0000"
        }
        Field {
            Checkbox { checked: true }
            Radio { name: "g" value: "a" }
            Switch checked: false
            Slider min: 0 max: 100 step: 5 value: 40
        }
        Navigation {
            Menu {
                MenuItem "Home"
            }
            Sidebar open: true {
                Toolbar {
                    Command "New" { shortcut: "Ctrl+N" }
                }
            }
        }
        ContextMenu open: false
        Tabs {
            Tab "One"
            Tab "Two"
        }
        Dialog open: false
        Popover open: false
        Modal open: false
        Sheet { Text "sheet" }
        Toast { Text "saved" }
        Tooltip "help"
        Video src: "clip.mp4" controls: true
        Audio src: "song.mp3" controls: true
        Canvas width: 400 height: 240
        Editor "body"
        CodeEditor language: "js"
        List { Table { } }
        Tree { }
        DataGrid {
            TableRow {
                Header "Name"
                Header "Age"
            }
            TableRow {
                Cell "Ann"
                Cell "32"
            }
        }
        Progress value: 0.5
        Loader {}
        Spinner {}
        Form action: "/save" method: "post" novalidate: true {
            Input placeholder: "type"
        }
        Grid columns: 3 {
            Text "cell"
        }
        Image src: "pic.png"
        Icon name: "star"
    }
}
"##,
    );
}

#[test]
fn accepts_the_container_primitive() {
    assert_clean(
        r##"
app Main {
    Container {
        Text "Hello"
        Button "Save" { }
    }
}
"##,
    );
}

#[test]
fn container_rejects_undeclared_properties() {
    assert_reports(
        r##"
app Main {
    Container {
        bogus: 1
    }
}
"##,
        "unknown-property",
    );
}

#[test]
fn rejects_undefined_identifier() {
    assert_reports(
        r##"
app Main {
    Column {
        Text missing
    }
}
"##,
        "undefined-identifier",
    );
}

#[test]
fn rejects_assigning_to_derived() {
    assert_reports(
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
        "assign-to-derived",
    );
}

#[test]
fn rejects_assigning_to_input() {
    assert_reports(
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
        "assign-to-input",
    );
}

#[test]
fn rejects_duplicate_declaration() {
    assert_reports(
        r##"
app Main {
    state count = 0
    state count = 1
    Column { }
}
"##,
        "duplicate-declaration",
    );
}

#[test]
fn rejects_missing_state_initializer() {
    assert_reports(
        r##"
app Main {
    state count
    Column { }
}
"##,
        "missing-initializer",
    );
}

#[test]
fn allows_declaration_only_for_shared_state() {
    assert_clean(
        r##"
app Main {
    shared state count: Int
    Column { }
}
"##,
    );
}

#[test]
fn accepts_input_without_a_type() {
    assert_clean(
        r##"
component UserCard {
    input user
    Text "hello"
}
app Main {
    UserCard user: "x"
}
"##,
    );
}

#[test]
fn rejects_unknown_type() {
    assert_reports(
        r##"
app Main {
    state count: Intt = 0
    Column { }
}
"##,
        "unknown-type",
    );
}

#[test]
fn accepts_builtin_types() {
    assert_clean(
        r##"
app Main {
    state a: Int = 0
    state b: Float = 0.0
    state c: String = ""
    state d: Bool = false
    state e: Any = null
    Column { }
}
"##,
    );
}

#[test]
fn rejects_unknown_event() {
    assert_reports(
        r##"
app Main {
    fn nothing() {}
    Button "x" {
        on klak {
            nothing()
        }
    }
}
"##,
        "unknown-event",
    );
}

#[test]
fn accepts_a_custom_event_emitted_by_the_component() {
    assert_clean(
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
    UserCard {}
}
"##,
    );
}

#[test]
fn rejects_reserved_event_name_on_emit() {
    assert_reports(
        r##"
component Bad {
    fn pick() {
        emit click(1)
    }
    Button "pick" { }
}
app Main {
    Bad {}
}
"##,
        "reserved-event-name",
    );
}

#[test]
fn rejects_unknown_property() {
    assert_reports(
        r##"
app Main {
    Button "x" {
        onnClick: true
    }
}
"##,
        "unknown-property",
    );
}

#[test]
fn rejects_missing_required_property() {
    assert_reports(
        r##"
app Main {
    Image { }
}
"##,
        "missing-property",
    );
}

#[test]
fn rejects_children_in_leaf_elements() {
    assert_reports(
        r##"
app Main {
    Column {
        Icon name: "star" {
            Text "x"
        }
    }
}
"##,
        "children-not-allowed",
    );
}

#[test]
fn rejects_text_in_elements_that_do_not_accept_it() {
    assert_reports(
        r##"
app Main {
    Column {
        Icon "star"
    }
}
"##,
        "text-not-allowed",
    );
}

#[test]
fn rejects_break_outside_a_loop() {
    assert_reports(
        r##"
app Main {
    fn stop() {
        break
    }
    Column { }
}
"##,
        "break-outside-loop",
    );
}

#[test]
fn accepts_break_inside_a_loop() {
    assert_clean(
        r##"
app Main {
    state items = []
    fn stop() {
        for item in items {
            break
        }
    }
    Column { }
}
"##,
    );
}

#[test]
fn rejects_app_without_a_root_element() {
    assert_reports(
        r##"
app Main {
    state count = 0
}
"##,
        "empty-app",
    );
}

#[test]
fn rejects_app_with_multiple_roots() {
    assert_reports(
        r##"
app Main {
    Column { }
    Row { }
}
"##,
        "multiple-roots",
    );
}

#[test]
fn loop_binding_is_visible_inside_the_loop_only() {
    assert_reports(
        r##"
app Main {
    state items = []
    Column {
        Text item
        for item in items {
            Text item
        }
    }
}
"##,
        "undefined-identifier",
    );
}

#[test]
fn loop_binding_is_visible_inside_the_loop() {
    assert_clean(
        r##"
app Main {
    state items = []
    Column {
        for item in items {
            Text item
        }
    }
}
"##,
    );
}

#[test]
fn function_parameters_are_visible_inside_the_function_only() {
    assert_reports(
        r##"
app Main {
    fn use(by: Int) {
        consume(by)
    }
    fn consume(value: Int) { }
    Column { }
}
"##,
        "undefined-identifier",
    );
}

#[test]
fn rejects_route_path_without_leading_slash() {
    assert_reports(
        r##"
api Notes {
    get all "notes"
}
app Main {
    Column { }
}
"##,
        "invalid-route-path",
    );
}

#[test]
fn accepts_well_formed_api_routes() {
    assert_clean(
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
}

#[test]
fn warns_about_non_plt_import() {
    let bag = analyze(
        r##"
import helper from "helper.txt"
app Main {
    Column { }
}
"##,
    );
    assert!(
        bag.warnings()
            .any(|warning| warning.label().contains("convention")),
        "expected a convention warning, got {:?}",
        bag.warnings()
            .map(|warning| warning.label())
            .collect::<Vec<_>>()
    );
}

#[test]
fn component_inputs_are_visible_in_the_component_body() {
    assert_clean(
        r##"
component Greeting {
    input name: String
    Text "Hello " name
}
app Main {
    Greeting name: "world"
}
"##,
    );
}

#[test]
fn diagnostics_carry_a_usable_span() {
    let source = r##"
app Main {
    Column {
        Text missing
    }
}
"##;
    let bag = analyze(source);
    let error = bag
        .errors()
        .find(|error| error.code == "undefined-identifier")
        .expect("undefined-identifier diagnostic");
    let span = error.span.expect("diagnostic should carry a span");
    assert_eq!(&source[span.start as usize..span.end as usize], "missing");
}
