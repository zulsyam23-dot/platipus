use platipus_compiler::codegen::{Target, web};
use platipus_compiler::ir::{IrModule, lower_program, verify};

fn lower(source: &str) -> IrModule {
    let outcome = platipus_compiler::parser::parse("codegen.plt", source);
    assert!(
        outcome.errors.is_empty(),
        "parse errors: {:?}",
        outcome.errors
    );
    let bag = platipus_compiler::semantic::SemanticChecker::new().check(&outcome.program);
    assert!(
        !bag.has_errors(),
        "semantic errors: {:?}",
        bag.errors().collect::<Vec<_>>()
    );
    let module = lower_program(&outcome.program).expect("lowering failed");
    match verify(&module) {
        Ok(()) => module,
        Err(errors) => panic!("ir errors: {errors:?}"),
    }
}

fn javascript(source: &str) -> String {
    let module = lower(source);
    platipus_compiler::codegen::Web
        .generate(&module, None)
        .expect("codegen failed")
        .into_iter()
        .find(|artifact| artifact.name == "app.js")
        .map(|artifact| artifact.contents)
        .expect("app.js artifact")
}

fn stylesheet(source: &str) -> String {
    let module = lower(source);
    platipus_compiler::codegen::Web
        .generate(&module, None)
        .expect("codegen failed")
        .into_iter()
        .find(|artifact| artifact.name == "app.css")
        .map(|artifact| artifact.contents)
        .expect("app.css artifact")
}

fn html(source: &str) -> String {
    let module = lower(source);
    platipus_compiler::codegen::Web
        .generate(&module, None)
        .expect("codegen failed")
        .into_iter()
        .find(|artifact| artifact.name == "index.html")
        .map(|artifact| artifact.contents)
        .expect("index.html artifact")
}

fn artifacts(source: &str) -> Vec<&'static str> {
    let module = lower(source);
    platipus_compiler::codegen::Web
        .generate(&module, None)
        .expect("codegen failed")
        .into_iter()
        .map(|artifact| artifact.name)
        .collect()
}

#[test]
fn the_web_target_emits_the_expected_artifacts() {
    let names = artifacts("app Main { Text \"hi\" }");
    assert_eq!(
        names,
        vec![
            "index.html",
            "app.css",
            "app.js",
            "dom.mjs",
            "tests.mjs",
            "program.json"
        ]
    );
}

#[test]
fn html_references_the_generated_assets() {
    let page = html("app Main { Text \"hi\" }");
    assert!(page.starts_with("<!doctype html>"));
    assert!(page.contains("<title>Main</title>"));
    assert!(page.contains("href=\"app.css\""));
    assert!(page.contains("src=\"app.js\""));
    assert!(page.contains("id=\"root\""));
}

#[test]
fn elements_map_to_dom_tags() {
    let script = javascript("app Main { Column { Text \"hi\" } }");
    assert!(
        script.contains("plt.el(\"div\", { attrs: { \"class\": \"plt-column\" } }"),
        "{script}"
    );
    // The class comes with the element, so the call carries both the tag's own
    // props and the class the base stylesheet selects it by.
    assert!(
        script.contains(r#"plt.el("span", { attrs: { "class": "plt-text" }, text: "hi" })"#),
        "{script}"
    );
}

#[test]
fn layout_primitives_receive_their_class() {
    let script = javascript("app Main { Row { Column { } } }");
    assert!(script.contains("\"class\": \"plt-row\""), "{script}");
    assert!(script.contains("\"class\": \"plt-column\""), "{script}");
}

#[test]
fn every_builtin_element_carries_a_class_for_the_base_stylesheet() {
    // The base stylesheet targets these, so an element without one is styled by
    // nothing but the browser's own defaults for its tag.
    let script =
        javascript(r#"app Main { Column { Button "go" Text "hi" Card { Text "c" } Input { } } }"#);
    for class in [
        "plt-button",
        "plt-text",
        "plt-card",
        "plt-input",
        "plt-column",
    ] {
        assert!(script.contains(class), "{class} missing from {script}");
    }
}

#[test]
fn the_base_stylesheet_covers_the_controls_a_browser_would_otherwise_style() {
    let css = stylesheet("app Main { Column { } }");
    // A control that does not inherit the page font is the single most common
    // reason a program looks like it was assembled from unrelated parts.
    assert!(css.contains("font: inherit;"), "{css}");
    // A rule may group its selectors and may qualify them with a tag, so
    // membership is read off every selector's class name rather than matched
    // against one exact rule opening.
    let selectors = selected_classes(&css);
    for class in [
        "plt-button",
        "plt-input",
        "plt-select",
        "plt-textarea",
        "plt-checkbox",
        "plt-radio",
        "plt-switch",
        "plt-slider",
        "plt-table",
        "plt-list",
    ] {
        assert!(
            selectors.contains(&class.to_string()),
            ".{class} is never selected by the base stylesheet"
        );
    }
}

/// The class names every rule in a stylesheet selects on. Only a line that opens
/// a rule is read, and each of its comma-separated parts contributes the class it
/// names whether or not a tag comes first: `input.plt-input` and `.plt-input`
/// select the same class.
fn selected_classes(css: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in css.lines() {
        if !line.contains('{') {
            continue;
        }
        let head = line.split('{').next().unwrap_or_default();
        for part in head.split(',') {
            for segment in part.split('.') {
                let class = segment
                    .split([':', '[', ' ', '>'])
                    .next()
                    .unwrap_or(segment);
                if class.starts_with("plt-") {
                    out.push(class.to_string());
                }
            }
        }
    }
    out
}

#[test]
fn every_builtin_element_is_styled_by_the_base_stylesheet() {
    // The complaint this guards is "most of the elements do not render properly",
    // and its cause is always one of these two: an element that compiles without
    // the class its rule selects on, or a class no rule in the sheet names. Both
    // leave the element on the browser's own defaults, which is what makes a
    // program look assembled rather than designed.
    let css = stylesheet("app Main { Column { } }");
    let styled = selected_classes(&css);
    let mut missing_rule = Vec::new();
    let mut missing_class = Vec::new();
    for (name, _) in platipus_compiler::semantic::element::builtins::ALL {
        let class = platipus_compiler::codegen::elements::class_of(name);
        if !styled.contains(&class) {
            missing_rule.push(name.to_string());
            continue;
        }
        // The class has to reach the node. Elements that take children cannot be
        // checked by compiling them all into one program, so the mapping itself
        // is what is asserted here.
        if class.is_empty() {
            missing_class.push(name.to_string());
        }
    }
    assert!(
        missing_rule.is_empty(),
        "the base stylesheet names no rule for: {missing_rule:?}"
    );
    assert!(missing_class.is_empty(), "{missing_class:?}");
    // A class is shared by every instance of its element, so the sheet may
    // select a class more than once through `:hover` and friends. What matters
    // is that each builtin is reached, which the loop above already established
    // for all of them.
    assert_eq!(
        platipus_compiler::semantic::element::builtins::ALL.len(),
        62,
        "a builtin was added without being listed here"
    );
}

#[test]
fn a_heading_level_reaches_the_tag_it_is_styled_by() {
    // The level is dropped from the properties and compiled into the tag instead,
    // so the stylesheet keys off the tag. A level left as an attribute would both
    // show up in the markup and leave the heading the size of an `h1`.
    for (level, tag) in [(1, "h1"), (2, "h2"), (3, "h3"), (6, "h6")] {
        let script = javascript(&format!("app Main {{ Heading {{ level: {level} }} }}"));
        assert!(
            script.contains(&format!("plt.el(\"{tag}\"")),
            "level {level} should compile to a {tag}: {script}"
        );
        assert!(
            !script.contains("level="),
            "the level should not stay an attribute: {script}"
        );
    }
    // A level outside the range clamps to the ends rather than emitting a tag the
    // HTML parser would rewrite under the program. `level` is typed `Int`, so
    // there is no string form to defend against here.
    for (level, tag) in [(0, "h1"), (9, "h6")] {
        let script = javascript(&format!("app Main {{ Heading {{ level: {level} }} }}"));
        assert!(
            script.contains(&format!("plt.el(\"{tag}\"")),
            "level {level} should clamp to a {tag}: {script}"
        );
    }
}

#[test]
fn a_component_with_several_roots_gets_a_rule_for_every_one_of_them() {
    // A component body may hold several sibling roots. The markup carries a
    // generated class on each of them, so a stylesheet that only walks the first
    // leaves the rest with a class no rule names, and an element with no rule is
    // styled by the browser's defaults alone.
    let source = r#"
component Pair {
    Row {
        gap: 4
    }
    Row {
        gap: 12
    }
}
app Main { Pair { } }
"#;
    let css = stylesheet(source);
    // A generated class is the element's own source position, which is digits
    // after the prefix. Matching the prefix alone also picks up `.plt-editor`.
    let classes: Vec<String> = css
        .lines()
        .filter_map(|line| {
            let name = line.trim().strip_prefix(".plt-e")?;
            let digits: String = name.chars().take_while(char::is_ascii_digit).collect();
            (!digits.is_empty()).then_some(digits)
        })
        .collect();
    assert_eq!(
        classes.len(),
        2,
        "each root needs a class of its own, got {classes:?} in\n{css}"
    );
    assert!(css.contains("gap: 4px;"), "{css}");
    assert!(css.contains("gap: 12px;"), "{css}");
    // The class that reaches the markup has to be one the sheet answers to.
    let script = javascript(source);
    for class in &classes {
        assert!(
            script.contains(&format!("plt-e{class}")),
            "plt-e{class} is in the stylesheet but not the markup"
        );
    }
}

#[test]
fn every_class_the_runtime_adds_is_styled_by_the_base_stylesheet() {
    // A few nodes are built by the runtime rather than by an element in the
    // program. They still land in the document, so they are still on screen, and
    // a class with no rule behind it is one of the ways a program looks like it
    // was assembled from parts that came from different rooms.
    let script = javascript("app Main { Column { Text \"hi\" } }");
    for class in ["plt-root", "plt-code-shell", "plt-code"] {
        assert!(
            script.contains(class),
            "{class} is added by the runtime but never reached this test's program: {script}"
        );
    }
    let css = stylesheet("app Main { Column { Text \"hi\" } }");
    let styled = selected_classes(&css);
    for class in ["plt-root", "plt-code-shell", "plt-code"] {
        assert!(
            styled.contains(&class.to_string()),
            ".{class} is added by the runtime but the base stylesheet names no rule for it"
        );
    }
    // The token spans the highlighter emits are what make the overlay worth
    // drawing, so they are styled too rather than inheriting the plain text.
    assert!(css.contains(".plt-code .plt-tok {"), "{css}");
}

#[test]
fn the_base_stylesheet_defines_the_tokens_its_own_rules_read() {
    let css = stylesheet("app Main { Column { } }");
    // A rule written in terms of a token the sheet never declares resolves to
    // nothing, which is how an element ends up with no background at all.
    let declared: Vec<String> = css
        .lines()
        .filter_map(|line| line.trim().strip_prefix("--plt-"))
        .filter_map(|line| line.split(':').next())
        .map(|name| format!("--plt-{name}"))
        .collect();
    let read: Vec<String> = css
        .lines()
        .flat_map(|line| line.split("var(").skip(1))
        .filter_map(|rest| rest.split(')').next())
        .map(|name| name.trim().to_string())
        .collect();
    let count = read.len();
    for token in read {
        assert!(
            declared.contains(&token),
            "{token} is read but never declared"
        );
    }
    assert!(
        count > 20,
        "the base rules should be written in tokens: {css}"
    );
}

#[test]
fn a_grouped_selector_keeps_its_dot_on_every_part() {
    // Prefixing the whole string produces `plt-button, .plt-command:hover`, where
    // the first part matches nothing and the second silently does all the work.
    let css = stylesheet("app Main { Column { } }");
    // A bare class is the tell: it cannot appear at the start of a rule.
    for line in css.lines() {
        let head = line.split('{').next().unwrap_or_default().trim();
        for part in head.split(',') {
            assert!(
                !part.trim_start().starts_with("plt-"),
                "a rule starts on a bare class, which matches nothing: {part} in {css}"
            );
        }
    }
}

#[test]
fn a_theme_token_becomes_a_custom_property_rather_than_a_css_property() {
    // Written as a plain property the browser drops the declaration, so a theme
    // parses cleanly and then does nothing at all.
    let css = stylesheet(
        r##"
theme dark {
    background: "#0d0d11"
    foreground: "#e8e8ef"
}
app Main { Column { } }
"##,
    );
    let block = theme_block(&css, "dark");
    assert!(block.contains("--plt-bg: #0d0d11;"), "{block}");
    assert!(block.contains("--plt-text: #e8e8ef;"), "{block}");
    // The base sheet does use `background:`, so the check is against the theme's
    // own block rather than against the whole file.
    assert!(!block.contains("background:"), "{block}");
    assert!(!block.contains("foreground:"), "{block}");
}

/// The declarations a theme emits, taken from the stylesheet rather than from a
/// hand-written copy of them.
fn theme_block<'a>(css: &'a str, name: &str) -> &'a str {
    let marker = format!("[data-plt-theme=\"{name}\"]");
    let start = css
        .find(&marker)
        .unwrap_or_else(|| panic!("no theme block for {name} in {css}"));
    let body = &css[start..];
    let end = body.find("\n}\n").expect("an unterminated theme block") + 3;
    &body[..end]
}
#[test]
fn a_theme_token_lands_on_the_token_the_base_rules_actually_read() {
    // The vocabulary is the load-bearing part. A theme that sets `--plt-background`
    // while every rule reads `var(--plt-bg)` compiles to a sheet that looks
    // themed in the source and is not themed on screen.
    let css = stylesheet(
        r##"
theme dark {
    background: "#0d0d11"
    accent: "#3399ff"
    brand.logo: "x"
}
app Main { Column { } }
"##,
    );
    let block = theme_block(&css, "dark");
    assert!(block.contains("--plt-bg: #0d0d11;"), "{block}");
    assert!(block.contains("--plt-accent: #3399ff;"), "{block}");
    // A token the base sheet has never heard of still passes through, so a
    // program can add one for its own rules to read.
    assert!(block.contains("--plt-brand-logo: x;"), "{block}");
}

#[test]
fn a_theme_reaches_the_document_and_not_only_a_selector_nothing_carries() {
    // `[data-plt-theme]` is emitted for a host page to switch with, but nothing
    // in a program sets the attribute, so a theme emitted only there is dead CSS.
    let css = stylesheet(
        r##"
theme dark {
    background: "#0d0d11"
}
app Main { Column { } }
"##,
    );
    assert!(css.contains(r#"[data-plt-theme="dark"],"#), "{css}");
    let block = theme_block(&css, "dark");
    assert!(
        block.contains(":root {\n  --plt-bg: #0d0d11;"),
        "the declared theme should reach the root it is built with: {block}"
    );
}

#[test]
fn container_is_a_div_with_its_layout_class() {
    let script = javascript("app Main { Container { Text \"hi\" } }");
    assert!(
        script.contains("plt.el(\"div\", { attrs: { \"class\": \"plt-container\" } }"),
        "{script}"
    );
    let css = stylesheet("app Main { Container { Text \"hi\" } }");
    assert!(css.contains(".plt-container {"), "{css}");
}

#[test]
fn a_layout_property_becomes_a_declaration_rather_than_an_attribute() {
    let source = "app Main { Stack { gap: 8 } }";
    let script = javascript(source);
    assert!(!script.contains("gap="), "{script}");
    let css = stylesheet(source);
    assert!(css.contains("gap: 8px;"), "{css}");
    // The base rule for the primitive stays untouched, and the override lands on
    // a class generated from this element alone.
    let base = css
        .lines()
        .find(|line| line.starts_with(".plt-stack {"))
        .expect("the stack's base rule");
    assert!(
        !base.contains("gap: 8px"),
        "{base} should not carry this element's gap"
    );
    // The layout class is a class selector, so the rule reads with its dot.
    assert!(base.contains("display: flex;"), "{base}");
}

#[test]
fn a_property_override_is_scoped_to_its_own_element() {
    let css = stylesheet("app Main { Column { Stack { padding: 4 } Stack { padding: 40 } } }");
    // A generated class is the element's own source position, which is digits
    // after the prefix. Matching on the prefix alone also picks up `.plt-editor`.
    let mut scoped: Vec<String> = css
        .lines()
        .filter_map(|line| {
            let name = line.trim().strip_prefix(".plt-e")?;
            let digits: String = name.chars().take_while(char::is_ascii_digit).collect();
            (!digits.is_empty()).then_some(digits)
        })
        .collect();
    scoped.sort_unstable();
    scoped.dedup();
    assert_eq!(
        scoped.len(),
        2,
        "each override needs a class of its own, got {scoped:?} in\n{css}"
    );
    // Neither value may land on the rule the two stacks share.
    assert!(!css.contains(".plt-stack { padding"), "{css}");
    assert!(css.contains("padding: 4px;"), "{css}");
    assert!(css.contains("padding: 40px;"), "{css}");
}

#[test]
fn a_column_count_becomes_a_track_list() {
    let css = stylesheet("app Main { Grid { columns: 3 } }");
    assert!(
        css.contains("grid-template-columns: repeat(3, minmax(0, 1fr));"),
        "{css}"
    );
}

#[test]
fn a_bare_number_on_a_length_gains_a_unit() {
    let css = stylesheet("app Main { Column { padding: 16 } }");
    assert!(css.contains("padding: 16px;"), "{css}");
    assert!(!css.contains("padding: 16;"), "{css}");
}

#[test]
fn a_unitless_property_keeps_its_bare_number() {
    let css = stylesheet("app Main { Card { style { opacity: 0.5 } } }");
    assert!(css.contains("opacity: 0.5;"), "{css}");
    let quoted = stylesheet("app Main { Card { style { \"z-index\": 3 } } }");
    assert!(quoted.contains("z-index: 3;"), "{quoted}");
}

#[test]
fn a_responsive_property_override_reaches_the_stylesheet() {
    let css = stylesheet("app Main { Column { responsive { tablet: padding: 24 } } }");
    assert!(
        css.contains("@media (min-width: 48rem) {\n  .plt-e"),
        "{css}"
    );
    assert!(css.contains("padding: 24px;"), "{css}");
}

#[test]
fn a_responsive_named_style_reaches_the_stylesheet() {
    let css = stylesheet(
        r##"
style wide {
    padding: 32
}

app Main {
    Column {
        responsive {
            desktop: wide
        }
    }
}
"##,
    );
    assert!(css.contains("@media (min-width: 64rem) {"), "{css}");
    assert!(css.contains("padding: 32px;"), "{css}");
}

#[test]
fn a_responsive_entry_on_the_base_breakpoint_is_a_plain_rule() {
    let css = stylesheet("app Main { Column { responsive { mobile: padding: 8 } } }");
    // The base sheet does use `@media`, for the system dark scheme and for reduced
    // motion. What must not appear is a width query, because the base breakpoint
    // is the layout itself rather than a size it switches at.
    assert!(!css.contains("@media (min-width"), "{css}");
    assert!(css.contains("padding: 8px;"), "{css}");
}

#[test]
fn rules_reach_an_element_inside_a_template_position() {
    let css = stylesheet(
        r##"
app Main {
    state show = true
    Column {
        if show {
            Stack {
                padding: 12
            }
        }
    }
}
"##,
    );
    assert!(css.contains("padding: 12px;"), "{css}");
}

#[test]
fn local_state_becomes_a_signal() {
    let script = javascript("app Main { state count = 0 Text \"hi\" }");
    assert!(script.contains("count: plt.signal(0)"), "{script}");
}

#[test]
fn global_state_is_shared_across_instances() {
    let script = javascript("app Main { global state total = 0 Text \"hi\" }");
    assert!(
        script.contains("total: plt.global(\"total\", 0)"),
        "{script}"
    );
}

#[test]
fn persistent_state_round_trips_through_local_storage() {
    let script = javascript("app Main { persistent state token = \"\" Text \"hi\" }");
    assert!(
        script.contains("token: plt.persistent(\"token\", \"\")"),
        "{script}"
    );
}

#[test]
fn reactive_dependencies_are_rewritten() {
    let script = javascript("app Main { state count = 0 Text count }");
    assert!(script.contains(r#"text: s.count.value"#), "{script}");
}

#[test]
fn handlers_become_dom_listeners() {
    let script = javascript(
        r#"
app Main {
    state count = 0
    Button "inc" {
        on click {
            count += 1
        }
    }
}
"#,
    );
    assert!(
        script.contains("on: { \"click\": async () => {"),
        "{script}"
    );
    assert!(script.contains("s.count.value += 1"), "{script}");
}

#[test]
fn custom_events_are_routed_through_channels() {
    let script = javascript(
        r#"
component Child {
    fn pick() {
        emit picked(1)
    }
    Button "pick" { }
}
app Main {
    state note = 0
    Child {
        on picked {
            note = 1
        }
    }
}
"#,
    );
    assert!(script.contains("plt.emitter(\"picked\")"), "{script}");
    assert!(
        script.contains("plt.child(\"Child\", {}, [], { \"plt:picked\": async () => {"),
        "{script}"
    );
}

#[test]
fn a_handler_reading_event_receives_the_normalized_object() {
    let script = javascript(
        r#"
app Main {
    state note = ""
    Input {
        placeholder: "type"
        on input {
            note = event.value
        }
    }
}
"#,
    );
    assert!(
        script.contains("\"input\": async (raw) => { const event = plt.event(raw);"),
        "{script}"
    );
    assert!(script.contains("s.note.value = event.value"), "{script}");
}

#[test]
fn a_non_bubbling_event_is_bound_on_the_capture_phase() {
    let script = javascript(
        r#"
app Main {
    state y = 0
    Scroll {
        on scroll {
            y = event.position.y
        }
        Text "content"
    }
}
"#,
    );
    assert!(script.contains("\"scroll\": ["), "{script}");
    assert!(script.contains(", true]"), "{script}");
    assert!(
        script.contains("const event = plt.event(raw); s.y.value = event.position.y"),
        "{script}"
    );
}

#[test]
fn a_scroll_top_binding_keeps_state_in_step_with_the_scroll() {
    let script = javascript(
        r#"
app Main {
    state depth = 0
    Scroll {
        bind scrollTop: depth
    }
}
"#,
    );
    assert!(
        script.contains("bind: { \"scrollTop\": plt.twoWay(() => s.depth,"),
        "{script}"
    );
}

#[test]
fn a_custom_event_delivers_its_payload_untouched() {
    let script = javascript(
        r#"
component Pump {
    fn go() {
        emit filled("water")
    }
    Button "go" { }
}
app Main {
    state container = ""
    Pump {
        on filled {
            container = event
        }
    }
}
"#,
    );
    assert!(
        script.contains("plt.child(\"Pump\", {}, [], { \"plt:filled\": async (event) => {"),
        "{script}"
    );
    assert!(!script.contains("plt.event("), "{script}");
}

#[test]
fn components_are_invoked_with_inputs_and_children() {
    let script = javascript(
        r#"
component Row2 {
    Column { }
}
app Main {
    Row2 {
        title: "hello"
        Text "child"
    }
}
"#,
    );
    assert!(
        script.contains(
            r#"plt.child("Row2", { title: "hello" }, [plt.el("span", { attrs: { "class": "plt-text" }, text: "child" })])"#
        ),
        "{script}"
    );
}

#[test]
fn functions_are_emitted_with_parameter_defaults() {
    let script = javascript(
        r#"
app Main {
    fn greet(name: String = "world") {
        Text name
    }
    Text "x"
}
"#,
    );
    assert!(
        script.contains("function f_greet(p_name = \"world\")"),
        "{script}"
    );
}

#[test]
fn derived_values_track_their_dependencies() {
    let script = javascript(
        r#"
app Main {
    state count = 0
    derived doubled = count * 2
    Text doubled
}
"#,
    );
    assert!(
        script.contains("d.doubled = plt.computed(() => (s.count.value * 2), [\"count\"], s, d);"),
        "{script}"
    );
}

#[test]
fn declared_inputs_become_a_valid_object_literal() {
    let script = javascript(
        r#"
app Main {
    input label: String = "hi"
    Text label
}
"#,
    );
    assert!(script.contains("inputs = inputs ?? {};"), "{script}");
    assert!(
        script.contains("inputs.label = inputs.label ?? inputDefaults.label;"),
        "{script}"
    );
    assert!(!script.contains("inputs.label:"), "{script}");
    // The instance hands its props object and its defaults back to the runtime,
    // which is what lets a reused component see an input the parent changed.
    assert!(
        script.contains("render, vnode, life, inputs, inputDefaults"),
        "{script}"
    );
    assert!(
        script.contains("function adoptProps(instance, props)"),
        "{script}"
    );
}

#[test]
fn a_component_with_several_roots_renders_a_fragment() {
    let script = javascript(
        r#"
component Pair {
    Column {
        Text "one"
    }
    Column {
        Text "two"
    }
}
app Main {
    Pair { }
}
"#,
    );
    // A component body may hold more than one sibling root. Emitting only the
    // first would silently drop the rest of the component, so the roots go into
    // a tagless fragment that the runtime flattens into the parent.
    assert!(
        script.contains("const render = () => plt.el(null, {}, ["),
        "{script}"
    );
    assert!(script.contains(r#"text: "one""#), "{script}");
    assert!(script.contains(r#"text: "two""#), "{script}");
}

#[test]
fn a_component_whose_body_is_only_a_branch_still_renders_it() {
    let script = javascript(
        r#"
component Flag {
    state lit = true
    if lit {
        Text "shown"
    }
}
app Main {
    Flag { }
}
"#,
    );
    // A body with no element root used to lower to `null` and render nothing.
    assert!(script.contains(r#"text: "shown""#), "{script}");
    assert!(script.contains(r#""class": "plt-text""#), "{script}");
}

#[test]
fn a_single_root_component_is_not_wrapped_in_a_fragment() {
    let script = javascript(
        r#"
component One {
    Column {
        Text "only"
    }
}
app Main {
    One { }
}
"#,
    );
    // Wrapping a lone root would insert a node the program never asked for.
    assert!(
        script.contains(r#"const render = () => plt.el("div", { attrs:"#),
        "{script}"
    );
}

#[test]
fn several_attributes_share_one_attrs_object() {
    let script = javascript(
        r#"
app Main {
    Button {
        disabled: true
        loading: false
    }
}
"#,
    );
    assert!(
        script.contains(
            r#"attrs: { "class": "plt-button", "disabled": true ? "" : null, "loading": false ? "" : null }"#
        ),
        "{script}"
    );
    assert_eq!(script.matches("attrs: {").count(), 1, "{script}");
}

#[test]
fn custom_emits_dispatch_through_the_component_channel() {
    let script = javascript(
        r#"
component Child {
    fn pick() {
        emit picked(1)
    }
    Button "pick" { }
}
app Main {
    state note = 0
    fn pickIt(value: Any) {
        note = value
    }
    fn trigger() {
        emit picked(2)
    }
    Child {
        on picked {
            pickIt(1)
        }
    }
}
"#,
    );
    assert!(script.contains("plt.emit(events.picked, 2)"), "{script}");
}

#[test]
fn control_flow_survives_lowering() {
    let script = javascript(
        r#"
app Main {
    state count = 0
    fn run(items: Any) {
        for item in items {
            if item > 0 {
                count += 1
            } else {
                count -= 1
            }
        }
    }
    Text "x"
}
"#,
    );
    assert!(
        script.contains("for (const item_item of plt.iter(p_items))"),
        "{script}"
    );
    assert!(script.contains("if (item_item > 0)"), "{script}");
    assert!(script.contains("else {"), "{script}");
}

#[test]
fn emits_translate_into_dispatch_calls() {
    let script = javascript(
        r#"
app Main {
    fn pick() {
        emit chosen(2)
    }
    Text "x"
}
"#,
    );
    assert!(script.contains("plt.emit(events.chosen, 2)"), "{script}");
}

#[test]
fn properties_split_between_attributes_and_dom_writes() {
    let script = javascript(
        r#"
app Main {
    state enabled = true
    Column {
        Input {
            placeholder: "type"
            value: "seed"
        }
        Button "save" {
            disabled: enabled
        }
    }
}
"#,
    );
    assert!(
        script
            .contains(r#"attrs: { "class": "plt-input", "placeholder": "type", "type": "text" }"#),
        "{script}"
    );
    assert!(script.contains("dom: { \"value\": \"seed\" }"), "{script}");
    assert!(
        script.contains(
            r#"attrs: { "class": "plt-button", "disabled": s.enabled.value ? "" : null }"#
        ),
        "{script}"
    );
}

#[test]
fn the_stylesheet_carries_layout_defaults() {
    let css = stylesheet("app Main { Column { } }");
    assert!(
        css.contains(".plt-column { display: flex; flex-direction: column;"),
        "{css}"
    );
    assert!(css.contains(":root {"), "{css}");
}

#[test]
fn style_blocks_reach_the_stylesheet() {
    let css = stylesheet(
        r##"
app Main { Text "x" }
style card {
    backgroundColor: "#111"
    hover {
        color: "#fff"
    }
}
"##,
    );
    assert!(css.contains("background-color: #111;"), "{css}");
    assert!(css.contains(".plt-root:hover {"), "{css}");
    assert!(css.contains("color: #fff;"), "{css}");
}

#[test]
fn apis_become_route_descriptors() {
    let script = javascript(
        r#"
api Notes {
    get all "/notes"
}
app Main { Text "x" }
"#,
    );
    assert!(
        script.contains("plt.route(\"get\", \"/notes\", \"Notes\")"),
        "{script}"
    );
    assert!(script.contains("export const $api"), "{script}");
}

#[test]
fn the_mount_entrypoint_replaces_the_root() {
    let script = javascript("app Main { Text \"hi\" }");
    assert!(script.contains("export function mount(target)"), "{script}");
    assert!(
        script.contains("target.replaceChildren(instance.root)"),
        "{script}"
    );
}

#[test]
fn member_accesses_survive_rewriting() {
    let script = javascript(
        r#"
app Main {
    state form = null
    Text form.name
}
"#,
    );
    assert!(script.contains("text: s.form.value.name"), "{script}");
}

#[test]
fn string_contents_are_not_rewritten() {
    let script = javascript(
        r#"
app Main {
    state title = "title"
    Text title
}
"#,
    );
    assert!(script.contains("title: plt.signal(\"title\")"), "{script}");
    assert!(script.contains("text: s.title.value"), "{script}");
}

#[test]
fn codegen_is_deterministic() {
    let source = "app Main { state count = 0 Button \"x\" { on click { count += 1 } } }";
    let first = platipus_compiler::codegen::Web
        .generate(&lower(source), None)
        .unwrap();
    let second = platipus_compiler::codegen::Web
        .generate(&lower(source), None)
        .unwrap();
    assert_eq!(first, second);
}

#[test]
fn the_web_module_is_reachable_through_its_own_type() {
    let module = lower("app Main { Text \"hi\" }");
    let produced = web::generate(&module, None).unwrap();
    assert_eq!(produced.len(), 6);
}

#[test]
fn a_fetch_call_is_lowered_to_the_plt_runtime() {
    let script = javascript(
        r#"
component Search {
    state response = 0
    state fetched = 0
    async fn run() {
        response = await fetch("data:text/plain,hi")
        fetched = response.status + ":" + response.text
    }
    Page {
        on mount { run() }
    }
}
app Main {
    Search { }
}
"#,
    );
    assert!(
        script.contains("s.response.value = await plt.fetch(\"data:text/plain,hi\")"),
        "{script}"
    );
    assert!(script.contains("async function f_run"), "{script}");
    assert!(
        script
            .contains("s.fetched.value = s.response.value.status + \":\" + s.response.value.text"),
        "{script}"
    );
}

#[test]
fn a_web_socket_handle_keeps_its_method_calls() {
    let script = javascript(
        r#"
component Chat {
    state socket = null
    state message = ""
    async fn run() {
        socket = await webSocket("ws://echo/room")
        socket.send("ping:1")
        message = await receive(socket)
    }
    Page {
        on mount { run() }
    }
}
app Main {
    Chat { }
}
"#,
    );
    assert!(
        script.contains("s.socket.value = await plt.webSocket(\"ws://echo/room\")"),
        "{script}"
    );
    assert!(
        script.contains("s.socket.value.send(\"ping:1\")"),
        "{script}"
    );
    assert!(
        script.contains("s.message.value = await plt.receive(s.socket.value)"),
        "{script}"
    );
}

#[test]
fn clipboard_file_and_storage_builtins_map_to_the_runtime() {
    let script = javascript(
        r#"
component Files {
    state clip = ""
    state picked = ""
    state stored = ""
    state ok = false
    state saved = ""
    async fn run() {
        clip = await readClipboard()
        picked = await openFile()
        ok = await writeClipboard("hello")
        saved = store("local", "theme", "dark")
        stored = load("local", "theme")
        drop("session", "theme")
    }
    Page {
        on mount { run() }
    }
}
app Main {
    Files { }
}
"#,
    );
    assert!(
        script.contains("s.clip.value = await plt.clipboardRead()"),
        "{script}"
    );
    assert!(
        script.contains("s.picked.value = await plt.openFile()"),
        "{script}"
    );
    assert!(
        script.contains("s.ok.value = await plt.clipboardWrite(\"hello\")"),
        "{script}"
    );
    assert!(
        script.contains("s.saved.value = plt.store(\"local\", \"theme\", \"dark\")"),
        "{script}"
    );
    assert!(
        script.contains("s.stored.value = plt.load(\"local\", \"theme\")"),
        "{script}"
    );
    assert!(
        script.contains("plt.drop(\"session\", \"theme\")"),
        "{script}"
    );
}

#[test]
fn a_handler_body_may_await() {
    let script = javascript(
        r#"
component Panel {
    state result = ""
    Button {
        on click {
            result = await readClipboard()
        }
    }
}
app Main {
    Panel { }
}
"#,
    );
    assert!(
        script.contains("async () => { s.result.value = await plt.clipboardRead() }"),
        "{script}"
    );
}

#[test]
fn canvas_builtins_lower_to_the_plt_runtime() {
    let script = javascript(
        r#"
component Stage {
    state c = null
    state n = 0
    state label = ""
    async fn loop() {
        fill(c, n, 10, 20, 20, "tomato")
        n = n + 1
        drawText(c, "frame", 4, 4)
        await nextFrame()
        loop()
    }
    Canvas {
        id: "board"
        width: 320
        height: 200
    }
    on mount {
        c = canvas("board")
        clear(c)
        loop()
    }
}
app Main {
    Stage { }
}
"#,
    );
    assert!(
        script.contains("s.c.value = plt.canvas(\"board\")"),
        "{script}"
    );
    assert!(
        script.contains("plt.fill(s.c.value, s.n.value, 10, 20, 20, \"tomato\")"),
        "{script}"
    );
    assert!(script.contains("plt.clear(s.c.value)"), "{script}");
    assert!(
        script.contains("plt.drawText(s.c.value, \"frame\", 4, 4)"),
        "{script}"
    );
    assert!(script.contains("await plt.nextFrame()"), "{script}");
}

#[test]
fn an_editor_is_contenteditable_with_binding_and_builtins() {
    let script = javascript(
        r#"
component Writing {
    state body = "hi"
    state sel = 0
    Column {
        Editor {
            value: body
            bind value: body
            on input { sel = selection().start }
        }
        Button "Bold" {
            on click { exec("bold") }
        }
    }
}
app Main {
    Writing { }
}
"#,
    );
    assert!(script.contains("\"contenteditable\": \"true\""), "{script}");
    assert!(script.contains("plt.twoWay"), "{script}");
    assert!(
        script.contains("s.sel.value = plt.selection().start"),
        "{script}"
    );
    assert!(script.contains("plt.exec(\"bold\")"), "{script}");
}

#[test]
fn a_code_editor_with_highlight_gets_an_overlay() {
    let script = javascript(
        r#"
component Code {
    state doc = "fn hello() { return 1 }"
    Column {
        Button "Tab" {
            on click { indent() }
        }
        CodeEditor {
            value: doc
            language: "plt"
            highlight: "plt"
        }
    }
}
app Main {
    Code { }
}
"#,
    );
    assert!(script.contains("s.doc.value"), "{script}");
    assert!(script.contains("plt.indent()"), "{script}");
    assert!(script.contains("function highlightHtml"), "{script}");
    assert!(script.contains("plt-tok"), "{script}");
    assert!(script.contains("function ensureCodeOverlay"), "{script}");
}

#[test]
fn data_grid_builtins_sort_and_page_lists() {
    let script = javascript(
        r#"
component Grid {
    state rows = []
    fn refresh() {
        rows = page(sortBy(rows, "qty", "desc"), 0, 5)
    }
    on mount { refresh() }
    Column {
        for row in rows {
            Text row.name
        }
    }
}
app Main {
    Grid { }
}
"#,
    );
    assert!(
        script
            .contains("s.rows.value = plt.page(plt.sortBy(s.rows.value, \"qty\", \"desc\"), 0, 5)"),
        "{script}"
    );
}

#[test]
fn swipe_is_composed_from_pointer_events() {
    let script = javascript(
        r#"
component Cards {
    state dir = "none"
    Column {
        transition: "0.3s"
        on swipe {
            dir = event.dir
        }
        Text dir
    }
}
app Main {
    Cards { }
}
"#,
    );
    assert!(script.contains("\"plt:swipe\": async (event)"), "{script}");
    assert!(
        script.contains("channelKey(name) === \"swipe\""),
        "{script}"
    );
    assert!(script.contains("function listenForSwipe"), "{script}");
    let css = stylesheet(
        r#"
style card {
    transition: "0.3s"
}
app Main {
    Container { }
}
"#,
    );
    assert!(css.contains("transition: 0.3s;"), "{css}");
}

#[test]
fn let_while_and_counted_ranges_compile_to_plain_javascript() {
    let script = javascript(
        r#"
app Main {
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
    Column { Text "x" }
}
"#,
    );
    assert!(script.contains("let t_total = 0"), "{script}");
    assert!(
        script.contains("for (const item_item of plt.iter(p_items))"),
        "{script}"
    );
    assert!(
        script.contains("for (let t_i = 0, t_i__end = 3; t_i < t_i__end; t_i++)"),
        "{script}"
    );
    assert!(script.contains("while (t_total > 0)"), "{script}");
    assert!(script.contains("t_total += 1"), "{script}");
    assert!(script.contains("s.count.value += 1"), "{script}");
}

#[test]
fn a_template_range_loops_through_the_runtime_range_helper() {
    let script = javascript(
        r#"
app Main {
    Column {
        for i in 0..4 {
            Text "row"
        }
    }
}
"#,
    );
    assert!(
        script.contains("plt.each(plt.range(0, 4, false), (t_item_i) =>"),
        "{script}"
    );
}

#[test]
fn iterating_a_number_is_refused_at_runtime() {
    let script = javascript(
        r#"
app Main {
    state count = 0
    fn run(items: Any) {
        for item in items {
            count += 1
        }
    }
    Column { Text "x" }
}
"#,
    );
    assert!(
        script.contains("a `for` loop cannot iterate over a number"),
        "{script}"
    );
}

#[test]
fn a_lambda_compiles_to_a_plain_arrow_function() {
    let script = javascript(
        r#"
app Main {
    state count = 0
    fn bump() {
        let f = x => x * 2
        let g = (a, b) => a + b
        count = f(3) + g(1, 2)
    }
    Column { Text "x" }
}
"#,
    );
    assert!(script.contains("let t_f = (l0_x) => l0_x * 2"), "{script}");
    assert!(script.contains("let t_g = (l1_a, l1_b) => l1_a + l1_b"), "{script}");
    assert!(script.contains("s.count.value = t_f(3) + t_g(1, 2)"), "{script}");
}

#[test]
fn bitwise_expressions_reach_the_runtime_intact() {
    let script = javascript(
        r#"
app Main {
    state masked = 5 & 3
    state shifted = 1 << 2 + 3
    state inverted = ~0 & 0xFF
    Column { Text masked Text shifted Text inverted }
}
"#,
    );
    assert!(script.contains("plt.signal(5 & 3)"), "{script}");
    assert!(script.contains("plt.signal(~0 & 255)"), "{script}");
    // The outermost expression keeps the target's own grouping. JavaScript binds
    // `+` tighter than `<<`, so this reads the same way it was written; the
    // parentheses that actually matter are the ones `a_bitwise_operand_is_...`
    // below checks for.
    assert!(script.contains("plt.signal(1 << 2 + 3)"), "{script}");
}

#[test]
fn a_bitwise_operand_is_parenthesised_even_when_its_priority_allows_it() {
    // JavaScript binds `==` tighter than `&`, so an emitted `a & b == c` would
    // mean something different there than it does here. Every bitwise operand
    // therefore carries its own parentheses rather than relying on the two
    // precedence tables to agree.
    let script = javascript(
        r#"
app Main {
    state n = 5
    state k = 3
    state grouped = n & k == 1
    Column { Text grouped }
}
"#,
    );
    assert!(
        script.contains("plt.signal((s.n.value & s.k.value) == 1)"),
        "{script}"
    );
}


#[test]
fn an_integer_division_call_reaches_the_runtime_helper() {
    let script = javascript(
        r#"
app Main {
    state half = idiv(9, 2)
    Column { Text half }
}
"#,
    );
    assert!(script.contains("plt.signal(plt.idiv(9, 2))"), "{script}");
    assert!(script.contains("function idiv("), "{script}");
}

#[test]
fn a_code_point_lookup_reaches_the_runtime_helper() {
    let script = javascript(
        r#"
app Main {
    state code = codeAt("abc", 1)
    Column { Text code }
}
"#,
    );
    assert!(script.contains("plt.signal(plt.codeAt("), "{script}");
    assert!(script.contains("function codeAt("), "{script}");
}

#[test]
fn a_top_level_functions_tests_reach_them_through_the_programs_exports() {
    // An imported module brings its `test` blocks along, and those blocks call
    // the library's own top-level functions. The runner is a separate module, so
    // it has to pull each function in under the exported name rather than the
    // internal `f_` name the program body uses. The loader inlines an import
    // into the program text, so a function declared here stands in for one that
    // arrived from a package.
    let module = lower(
        r#"
fn double(n: Int) -> Int {
    return n * 2
}

app Main {
    state n = double(21)
    Column { Text n }
}

test libraryTests {
    expect double(2) == 4
}
"#,
    );
    let runner = platipus_compiler::codegen::Web
        .generate(&module, None)
        .expect("codegen failed")
        .into_iter()
        .find(|artifact| artifact.name == "tests.mjs")
        .expect("tests.mjs")
        .contents;
    assert!(runner.contains("const double = program.double;"), "{runner}");
    assert!(!runner.contains("f_double"), "{runner}");
}

#[test]
fn the_runtime_is_exported_even_when_a_program_declares_no_function() {
    let module = lower(
        r#"
app Main {
    state n = len([1, 2])
    Column { Text n }
}
"#,
    );
    let script = platipus_compiler::codegen::Web
        .generate(&module, None)
        .expect("codegen failed")
        .into_iter()
        .find(|artifact| artifact.name == "app.js")
        .expect("app.js")
        .contents;
    // The generated runner rewrites builtin calls to `plt.*`, so `__plt` cannot
    // be conditional on the program happening to declare a function.
    assert!(script.contains("export const __plt = plt;"), "{script}");
}

#[test]
fn a_layout_property_reaches_the_stylesheet_rather_than_an_attribute() {
    // The registry spells these the way CSS does and the language spells them in
    // camelCase, so the two have to be compared in one spelling. Emitting
    // `max-width="960px"` as an HTML attribute styles nothing, which is exactly
    // what a browser would do with it.
    let script = javascript(
        r#"
app Main {
    state n = 1
    Column {
        maxWidth: "960px"
        minHeight: "100vh"
        gap: 8
        Text n
    }
}
"#,
    );
    assert!(!script.contains("\"max-width\""), "{script}");
    assert!(!script.contains("\"min-height\""), "{script}");
    assert!(script.contains("plt-e"), "{script}");
    let module = lower(
        r#"
app Main {
    state n = 1
    Column {
        maxWidth: "960px"
        Text n
    }
}
"#,
    );
    let css = platipus_compiler::codegen::Web
        .generate(&module, None)
        .expect("codegen failed")
        .into_iter()
        .find(|artifact| artifact.name == "app.css")
        .expect("app.css")
        .contents;
    assert!(css.contains("max-width: 960px"), "{css}");
}

#[test]
fn an_attribute_that_is_not_a_layout_property_stays_an_attribute() {
    // The other half of the rule: a `Slider`'s `value` is the DOM property that
    // control expects, not a CSS declaration, even though the name looks like a
    // measurement.
    let script = javascript(
        r#"
app Main {
    state n = 5
    Slider {
        min: 0.0
        max: 10.0
        value: n * 1.0
    }
}
"#,
    );
    assert!(script.contains("dom: { \"value\""), "{script}");
    assert!(!script.contains("\"min-width\""), "{script}");
}
