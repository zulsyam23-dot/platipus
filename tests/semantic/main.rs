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

#[test]
fn state_annotation_mismatch_is_an_error() {
    assert_reports(
        r##"
app Main {
    state count: Int = "hello"
    Column { }
}
"##,
        "type-mismatch",
    );
}

#[test]
fn state_annotation_matching_is_clean() {
    assert_clean(
        r##"
app Main {
    state count: Int = 0
    state name: String = "x"
    state rate: Float = 1.5
    state ok: Bool = true
    Column { }
}
"##,
    );
}

#[test]
fn derived_annotation_mismatch_is_an_error() {
    assert_reports(
        r##"
app Main {
    derived label: Int = "ready"
    Column { }
}
"##,
        "type-mismatch",
    );
}

#[test]
fn parameter_default_mismatch_is_an_error() {
    assert_reports(
        r##"
app Main {
    fn add(by: Int = "ten") {
        count = count + by
    }
    Column { }
}
"##,
        "type-mismatch",
    );
}

#[test]
fn string_annotation_accepts_strings() {
    assert_clean(
        r##"
app Main {
    state title: String = "Hi"
    Column { }
}
"##,
    );
}

#[test]
fn arithmetic_annotations_are_accepted() {
    assert_clean(
        r##"
app Main {
    state n: Int = 2 + 3
    state r: Float = 1.5 * 2
    state s: String = "a" + "b"
    Column { }
}
"##,
    );
}

#[test]
fn float_plus_string_is_not_float() {
    assert_reports(
        r##"
app Main {
    state x: Float = 1.5 + "a"
    Column { }
}
"##,
        "type-mismatch",
    );
}

const LOOPING: &str = r##"
app Looping {
    state count = 0

    fn run(items: Any) {
        let total = 0
        for item in items {
            total += 1
        }
        for i in 0..3 {
            count += 1
        }
        while total > 0 {
            total -= 1
        }
    }

    Column {
        Text "x"
    }
}
"##;

#[test]
fn accepts_let_while_and_counted_ranges_inside_a_function() {
    assert_clean(LOOPING);
}

#[test]
fn a_let_may_not_shadow_a_declaration() {
    assert_reports(
        r##"
app Shadow {
    state total = 0
    fn f() {
        let total = 1
        total = 2
    }
    Column { Text "x" }
}
"##,
        "shadow-local",
    );
}

#[test]
fn a_parameter_cannot_be_assigned_to() {
    assert_reports(
        r##"
app Parameter {
    fn f(step: Int) {
        step = 1
    }
    Column { Text "x" }
}
"##,
        "assign-to-parameter",
    );
}

#[test]
fn a_for_loop_cannot_walk_a_single_number() {
    assert_reports(
        r##"
app Counting {
    fn f() {
        for i in 10 { }
    }
    Column { Text "x" }
}
"##,
        "for-over-number",
    );
}

#[test]
fn let_and_while_stay_out_of_template_positions() {
    assert_reports(
        "app A { let x = 1 Column { } }",
        "statement-outside-function",
    );
    assert_reports(
        "app A { while x < 1 { } Column { } }",
        "statement-outside-function",
    );
    assert_reports(
        "app A { Column { let x = 1 } }",
        "statement-outside-function",
    );
}

#[test]
fn a_lambda_parameter_may_not_shadow_an_outer_binding() {
    assert_reports(
        r##"
app Shadow {
    fn f() {
        let total = 1
        let g = total => total
    }
    Column { Text "x" }
}
"##,
        "shadow-local",
    );
}

#[test]
fn a_lambda_body_is_checked_for_undefined_identifiers() {
    assert_reports(
        r##"
app LambdaScope {
    fn f() {
        let g = x => x + missing
    }
    Column { Text "x" }
}
"##,
        "undefined-identifier",
    );
}

#[test]
fn a_lambda_body_may_use_let_and_while() {
    assert_clean(
        r##"
app LambdaBody {
    fn f() {
        let g = x => {
            let y = 0
            while y < x { y = y + 1 }
            return y
        }
    }
    Column { Text "x" }
}
"##,
    );
}

// -- arity, purity, and bitwise operands -------------------------------------
//
// The three rules below were all enforced by the checker but none of them had a
// test here, so a change that stopped enforcing one would have been invisible.
// Each test states the diagnostic the rule produces, because a diagnostic that
// stops being reported is how the rule quietly stops existing.

#[test]
fn a_user_function_rejects_too_many_arguments() {
    assert_reports(
        r##"
fn add(a: Int, b: Int) -> Int {
    return a + b
}

app Arity {
    state total = add(1, 2, 3)
    Column { Text total }
}
"##,
        "wrong-argument-count",
    );
}

#[test]
fn a_user_function_rejects_too_few_arguments() {
    assert_reports(
        r##"
fn add(a: Int, b: Int) -> Int {
    return a + b
}

app ArityFew {
    state total = add(1)
    Column { Text total }
}
"##,
        "wrong-argument-count",
    );
}

#[test]
fn a_parameter_with_a_default_makes_the_arity_a_range() {
    assert_clean(
        r##"
fn greet(name: String, punctuation: String = "!") -> String {
    return name + punctuation
}

app ArityDefault {
    state a = greet("hi")
    state b = greet("hi", "?")
    Column { Text a Text b }
}
"##,
    );
}

#[test]
fn a_builtin_rejects_the_wrong_number_of_arguments() {
    assert_reports(
        r##"
app BuiltinArity {
    state count = len()
    Column { Text count }
}
"##,
        "wrong-argument-count",
    );
}

#[test]
fn a_three_argument_builtin_rejects_two() {
    assert_reports(
        r##"
app BuiltinArityThree {
    state text = replace("a", "b")
    Column { Text text }
}
"##,
        "wrong-argument-count",
    );
}

#[test]
fn a_variadic_builtin_accepts_a_range() {
    assert_clean(
        r##"
app Variadic {
    state least = min(3, 1, 2)
    state most = max(1, 2, 3, 4, 5, 6, 7)
    Column { Text least Text most }
}
"##,
    );
}

#[test]
fn a_variadic_builtin_still_rejects_one_argument() {
    assert_reports(
        r##"
app VariadicFew {
    state least = min(3)
    Column { Text least }
}
"##,
        "wrong-argument-count",
    );
}

#[test]
fn an_impure_builtin_is_rejected_inside_a_top_level_function() {
    assert_reports(
        r##"
fn readIt(key: String) -> String {
    return load("local", key)
}

app Impure {
    state value = readIt("k")
    Column { Text value }
}
"##,
        "impure-call-in-pure-fn",
    );
}

#[test]
fn a_top_level_function_may_not_declare_state() {
    assert_reports(
        r##"
fn bad() {
    state counter = 0
}

app PureFnState {
    Column { Text "x" }
}
"##,
        "pure-fn-uses-state",
    );
}

#[test]
fn a_top_level_function_may_not_declare_a_derived() {
    assert_reports(
        r##"
fn bad() {
    derived twice = 1 + 1
}

app PureFnDerived {
    Column { Text "x" }
}
"##,
        "pure-fn-uses-state",
    );
}

#[test]
fn a_top_level_function_may_not_emit() {
    assert_reports(
        r##"
fn bad() {
    emit done("done")
}

app PureFnEmit {
    Column { Text "x" }
}
"##,
        "impure-call-in-pure-fn",
    );
}

#[test]
fn a_top_level_function_may_use_pure_builtins() {
    assert_clean(
        r##"
fn total(values: Any) -> Int {
    return len(values) + abs(-1)
}

app PureFnOk {
    state n = total([1, 2, 3])
    Column { Text n }
}
"##,
    );
}

#[test]
fn bitwise_operators_require_integer_operands() {
    assert_reports(
        r##"
app BitwiseFloat {
    state masked = 5 & 3.5
    Column { Text masked }
}
"##,
        "type-mismatch",
    );
}

#[test]
fn bitwise_operators_accept_integer_operands() {
    assert_clean(
        r##"
app BitwiseInt {
    state masked = 5 & 3
    state shifted = 1 << 4
    state ored = 0xF0 | 0x0F
    state xored = 5 ^ 3
    state inverted = ~0
    Column { Text masked Text shifted Text ored Text xored Text inverted }
}
"##,
    );
}

// -- regressions found while building a real package --------------------------
//
// Each of these was a bug that only a program using the feature could reach, so
// each is now stated here rather than left to the next person to rediscover.

#[test]
fn a_style_value_that_is_not_a_literal_is_rejected() {
    // A stylesheet cannot read state, so an expression written into one would be
    // copied out as its own source text and dropped by the browser with nothing
    // to say so. `computasi`'s widgets hit this while trying to colour a tile
    // from an input.
    assert_reports(
        r##"
app StaticStylesheet {
    state rule = "var(--plt-accent)"
    Card {
        style {
            "border-left": "3px solid " + rule
        }
    }
}
"##,
        "style-value-not-literal",
    );
}

#[test]
fn a_literal_style_value_is_accepted() {
    assert_clean(
        r##"
app StaticStylesheet {
    Card {
        style {
            "border-left": "3px solid var(--plt-accent)"
            gap: 6
            fontSize: "12px"
        }
    }
}
"##,
    );
}

#[test]
fn a_layout_property_written_in_camel_case_resolves() {
    // The registry spells layout properties the way CSS does, but a property
    // name is an identifier and cannot carry a hyphen, so `maxWidth` is the only
    // way to write `max-width`.
    assert_clean(
        r##"
app CamelCaseLayout {
    Card {
        maxWidth: "960px"
        minHeight: "100vh"
        borderRadius: "12px"
    }
}
"##,
    );
}

#[test]
fn a_layout_property_that_does_not_exist_is_still_rejected() {
    assert_reports(
        r##"
app CamelCaseLayout {
    Card {
        maxWidht: "960px"
    }
}
"##,
        "unknown-property",
    );
}
