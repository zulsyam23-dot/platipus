use std::collections::BTreeMap;

use crate::codegen::elements::{
    PropertyTarget, accepts_text, attribute_name, class_of, input_type_of, is_component,
    is_style_property, property_target, tag_of,
};
use crate::codegen::events::{dispatch_name, is_lifecycle, needs_capture};
use crate::codegen::expression::{Scope, rewrite};
use crate::codegen::statement::block;
use crate::ir::element::ElementItem;
use crate::ir::{
    IrElement, IrHandler, IrResponsiveBlock, IrStatement, IrStyleBlock, StatementKind,
};

/// Class name the stylesheet target uses for an element-scoped `style { }`
/// block. Derived from the block's own source span so the JavaScript and the
/// stylesheet agree without sharing mutable state.
pub fn inline_style_class(style: &IrStyleBlock) -> String {
    format!("plt-s{}", style.span.start)
}

/// The element-scoped `style { }` block, if the element declares one.
pub fn inline_style(element: &IrElement) -> Option<&IrStyleBlock> {
    element
        .body
        .as_ref()?
        .items
        .iter()
        .find_map(|item| match item {
            ElementItem::Style(block) => Some(block),
            _ => None,
        })
}

/// The declarations an element spells as properties rather than attributes.
/// A component element is excluded: its properties are inputs for the
/// component, not styling for the generated node.
pub fn style_properties(element: &IrElement) -> Vec<&crate::ir::IrProperty> {
    if is_component(element) {
        return Vec::new();
    }
    element
        .properties
        .iter()
        .filter(|property| is_style_property(&element.name, &property.name))
        .collect()
}

/// The `responsive { }` block, if the element declares one.
pub fn responsive_block(element: &IrElement) -> Option<&IrResponsiveBlock> {
    element
        .body
        .as_ref()?
        .items
        .iter()
        .find_map(|item| match item {
            ElementItem::Responsive(block) => Some(block),
            _ => None,
        })
}

/// Class name the stylesheet target uses for the rules an element carries on
/// its own. Derived from the element's own source span, the same way
/// [`inline_style_class`] derives one from a style block, so the JavaScript and
/// the stylesheet agree.
///
/// The primitive's own class is shared by every instance of that primitive, so
/// an override written under it would restyle all of them. Any element that
/// carries an override therefore also carries a class of its own.
pub fn override_class(element: &IrElement) -> Option<String> {
    let scoped = !style_properties(element).is_empty() || responsive_block(element).is_some();
    scoped.then(|| format!("plt-e{}", element.span.start))
}

/// Renders an element tree as a JavaScript expression.
pub fn render(element: &IrElement, scope: &Scope) -> String {
    if is_component(element) {
        return render_component(element, scope);
    }
    let mut attrs: BTreeMap<String, String> = BTreeMap::new();
    let mut dom: BTreeMap<String, String> = BTreeMap::new();
    let mut classes: Vec<String> = Vec::new();
    // Every builtin element carries a class of its own, not only the layout ones.
    // The base stylesheet targets these, so a `Button` is styled as a button
    // without the rules leaking onto any other markup on the page.
    classes.push(class_of(&element.name));
    // An element-scoped `style { }` block becomes a generated class whose rules
    // the stylesheet target emits under the same name.
    if let Some(style) = inline_style(element) {
        classes.push(inline_style_class(style));
    }
    // Style properties and a `responsive { }` block become rules on a class
    // generated from the element's own source position, so a value that differs
    // per instance cannot collide with the rule shared by every instance of the
    // same primitive.
    if let Some(class) = override_class(element) {
        classes.push(class);
    }
    for property in &element.properties {
        if element.name == "Heading" && property.name == "level" {
            continue;
        }
        collect_property(
            &element.name,
            &property.name,
            &property.value.value,
            scope,
            &mut attrs,
            &mut dom,
        );
    }
    // An `Editor` is a contenteditable box, so the node has to say so for the
    // bindings and the event object to read and write its text.
    if element.name == "Editor" {
        attrs.insert("contenteditable".to_string(), "\"true\"".to_string());
    }
    // Which control an element is comes from its `type` attribute, so one that
    // does not state a type gets the one its name implies. An element that does
    // state one keeps it.
    if let Some(kind) = input_type_of(&element.name) {
        attrs
            .entry("type".to_string())
            .or_insert_with(|| format!("{:?}", kind));
    }
    if !classes.is_empty() {
        attrs.insert("class".to_string(), format!("{:?}", classes.join(" ")));
    }
    let mut props: Vec<String> = Vec::new();
    if !attrs.is_empty() {
        props.push(format!("attrs: {}", object_literal(&attrs)));
    }
    if !dom.is_empty() {
        props.push(format!("dom: {}", object_literal(&dom)));
    }
    if let Some(text) = &element.text {
        if accepts_text(&element.name) {
            props.push(format!("text: {}", rewrite(&text.value, scope)));
        }
    }
    if let Some(handlers) = handler_object(element, scope) {
        props.push(format!("on: {handlers}"));
    }
    if let Some(lifecycle) = lifecycle_object(element, scope) {
        props.push(format!("life: {lifecycle}"));
    }
    if let Some(bindings) = binding_object(element, scope) {
        props.push(format!("bind: {bindings}"));
    }
    let children = children(element, scope);
    let tag = if element.name == "Heading" {
        heading_tag(&element.properties)
    } else {
        tag_of(&element.name)
    };
    if props.is_empty() && children.is_empty() {
        return format!("plt.el({tag:?})");
    }
    if children.is_empty() {
        return format!("plt.el({tag:?}, {{ {} }})", props.join(", "));
    }
    format!(
        "plt.el({tag:?}, {{ {} }}, [{}])",
        props.join(", "),
        children.join(", ")
    )
}

/// The HTML heading tag for a `Heading`, chosen from its static `level`
/// property. A dynamic or missing level stays `h1`: the tag is fixed at
/// compile time because the virtual DOM cannot retarget an existing node.
fn heading_tag(properties: &[crate::ir::IrProperty]) -> &'static str {
    let Some(raw) = properties
        .iter()
        .find(|property| property.name == "level")
        .map(|property| &property.value.value)
    else {
        return "h1";
    };
    let level = raw.trim().parse::<u8>().unwrap_or(1).clamp(1, 6);
    match level {
        1 => "h1",
        2 => "h2",
        3 => "h3",
        4 => "h4",
        5 => "h5",
        _ => "h6",
    }
}

fn object_literal(entries: &BTreeMap<String, String>) -> String {
    let body = entries
        .iter()
        .map(|(key, value)| format!("{key:?}: {value}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ {body} }}")
}

/// Merges handler bodies into one object literal keyed by event or lifecycle
/// name, so repeated handlers for one name compose instead of overwriting one
/// another.
///
/// A handler body that reads the `event` identifier receives the normalized
/// object (`.value`, `.key`, `.position`, and `stopPropagation`), built by
/// `plt.event`. A lifecycle body never takes the parameter, and a custom event
/// body receives the payload that was emitted, untouched. An entry marked with
/// capture is emitted as a two-element array so the runtime binds it in the
/// capture phase, letting an ancestor observe an event that does not bubble.
pub fn merge_handlers(entries: &[(String, String, bool)]) -> Option<String> {
    if entries.is_empty() {
        return None;
    }
    let mut merged: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut captured: BTreeMap<String, bool> = BTreeMap::new();
    for (name, body, capture) in entries {
        merged.entry(name.clone()).or_default().push(body.clone());
        if *capture {
            captured.insert(name.clone(), true);
        }
    }
    let fields = merged
        .iter()
        .map(|(name, bodies)| {
            let body = bodies.join("; ");
            // Every handler body runs as an async function so that `await`s in
            // it are legal; a body that has none behaves exactly like before.
            let handler = if is_lifecycle(name) {
                format!("async () => {{ {body} }}")
            } else if name.starts_with("plt:") {
                // A custom event delivers a payload, not a platform event.
                format!("async ({}) => {{ {body} }}", event_parameter(&body))
            } else if event_parameter(&body).is_empty() {
                format!("async () => {{ {body} }}")
            } else {
                // The raw platform event is folded into the normalized object
                // the body reads. The parameter keeps an unrelated name so the
                // handler's `const event` binding does not shadow it.
                format!("async (raw) => {{ const event = plt.event(raw); {body} }}")
            };
            if captured.contains_key(name) {
                format!("{name:?}: [{handler}, true]")
            } else {
                format!("{name:?}: {handler}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!("{{ {fields} }}"))
}

/// The listener bodies of a set of handlers, keyed by the channel they are
/// dispatched through and whether that channel must be listened on in the
/// capture phase.
fn listener_entries(handlers: &[&IrHandler], scope: &Scope) -> Vec<(String, String, bool)> {
    handlers
        .iter()
        .map(|handler| {
            (
                dispatch_name(handler),
                block(&handler.body, scope),
                needs_capture(&handler.event),
            )
        })
        .collect()
}

/// The lifecycle bodies of a set of handlers, keyed by lifecycle name.
pub fn lifecycle_entries(handlers: &[&IrHandler], scope: &Scope) -> Vec<(String, String, bool)> {
    handlers
        .iter()
        .map(|handler| (handler.event.clone(), block(&handler.body, scope), false))
        .collect()
}

/// Every `on click { }` for the same event becomes one listener. A lifecycle
/// handler is left out: the runtime calls it directly instead of binding it to
/// a DOM event.
fn handler_object(element: &IrElement, scope: &Scope) -> Option<String> {
    merge_handlers(&listener_entries(&listeners(&element.handlers), scope))
}

/// The listeners among a set of handlers, in other words the ones bound to a
/// DOM or emitter channel.
pub fn listeners(handlers: &[IrHandler]) -> Vec<&IrHandler> {
    handlers
        .iter()
        .filter(|handler| !is_lifecycle(&handler.event))
        .collect()
}

/// The `create`, `mount`, `update`, and `destroy` handlers an element declares,
/// which the runtime looks up by name at each point of its life.
fn lifecycle_object(element: &IrElement, scope: &Scope) -> Option<String> {
    merge_handlers(&lifecycle_entries(&lifecycle(&element.handlers), scope))
}

/// The lifecycle handlers among a set of handlers.
pub fn lifecycle(handlers: &[IrHandler]) -> Vec<&IrHandler> {
    handlers
        .iter()
        .filter(|handler| is_lifecycle(&handler.event))
        .collect()
}

/// An event handler only declares the `event` parameter when its body uses it.
fn event_parameter(body: &str) -> &'static str {
    if uses_event_identifier(body) {
        "event"
    } else {
        ""
    }
}

fn uses_event_identifier(body: &str) -> bool {
    let chars: Vec<char> = body.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        if crate::codegen::expression::is_identifier_start(chars[index]) {
            let start = index;
            while index < chars.len()
                && crate::codegen::expression::is_identifier_continue(chars[index])
            {
                index += 1;
            }
            let word: String = chars[start..index].iter().collect();
            if word == "event" && !preceded_by_member(&chars, start) {
                return true;
            }
            continue;
        }
        if chars[index] == '"' || chars[index] == '\'' {
            let quote = chars[index];
            index += 1;
            while index < chars.len() {
                let next = chars[index];
                index += 1;
                if next == quote {
                    break;
                }
            }
            continue;
        }
        index += 1;
    }
    false
}

fn preceded_by_member(source: &[char], start: usize) -> bool {
    let mut cursor = start;
    while cursor > 0 {
        cursor -= 1;
        if source[cursor].is_whitespace() {
            continue;
        }
        return source[cursor] == '.';
    }
    false
}

/// `bind name: expr` becomes a two-way link: the runtime seeds the field from
/// the bound box and writes user edits back into it.
fn binding_object(element: &IrElement, scope: &Scope) -> Option<String> {
    let mut entries: Vec<String> = Vec::new();
    for binding in &element.bindings {
        let access = rewrite(&binding.value.value, scope);
        // The lowered text reads through `.value`; the link needs the box.
        let Some(box_path) = access.strip_suffix(".value") else {
            continue;
        };
        entries.push(format!(
            "{:?}: plt.twoWay(() => {box_path}, (value) => {{ {box_path}.value = value }})",
            binding.name
        ));
    }
    if entries.is_empty() {
        return None;
    }
    Some(format!("{{ {} }}", entries.join(", ")))
}

fn collect_property(
    element: &str,
    name: &str,
    value: &str,
    scope: &Scope,
    attrs: &mut BTreeMap<String, String>,
    dom: &mut BTreeMap<String, String>,
) {
    if value.is_empty() {
        return;
    }
    let value = rewrite(value, scope);
    match property_target(element, name) {
        PropertyTarget::Attribute => {
            attrs.insert(attribute_name(name), value);
        }
        PropertyTarget::BooleanAttribute => {
            attrs.insert(attribute_name(name), format!("{value} ? \"\" : null"));
        }
        PropertyTarget::DomProperty => {
            dom.insert(name.to_string(), value);
        }
        // Style properties are not attributes. They are written to the
        // stylesheet under `property_class`, which the element carries.
        PropertyTarget::Style => {}
    }
}

fn render_component(element: &IrElement, scope: &Scope) -> String {
    let inputs: Vec<String> = element
        .properties
        .iter()
        .map(|property| {
            format!(
                "{}: {}",
                property.name,
                rewrite(&property.value.value, scope)
            )
        })
        .collect();
    let children = children(element, scope);
    let listeners = handler_object(element, scope);
    let name = format!("{:?}", element.name);
    let inputs_literal = if inputs.is_empty() {
        "{}".to_string()
    } else {
        format!("{{ {} }}", inputs.join(", "))
    };
    let children_literal = if children.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", children.join(", "))
    };
    // The trailing listener argument is only written when the parent listens.
    match listeners {
        None if inputs.is_empty() && children.is_empty() => {
            format!("plt.child({name}, {{}})")
        }
        None if children.is_empty() => {
            format!("plt.child({name}, {inputs_literal})")
        }
        None => format!("plt.child({name}, {inputs_literal}, {children_literal})"),
        Some(listeners) => {
            format!("plt.child({name}, {inputs_literal}, {children_literal}, {listeners})")
        }
    }
}

/// Renders the children of an element. Control flow in an element body becomes
/// a reactive node, so `if` and `for` are templates rather than plain
/// statements.
pub fn children(element: &IrElement, scope: &Scope) -> Vec<String> {
    let Some(body) = element.body.as_ref() else {
        return Vec::new();
    };
    render_items(&body.items, scope)
}

/// Renders a list of body items as the children of whatever holds them.
///
/// A style block becomes a class on the element, and a responsive block becomes
/// stylesheet rules; neither is a child node, so both are skipped.
pub fn render_items(items: &[ElementItem], scope: &Scope) -> Vec<String> {
    items
        .iter()
        .filter_map(|item| match item {
            ElementItem::Child(child) => Some(render(child, scope)),
            ElementItem::Statement(statement) => branch_expression(statement, scope),
            ElementItem::Style(_) | ElementItem::Responsive(_) => None,
        })
        .collect()
}

/// Renders a statement used in a template position.
fn branch_expression(statement: &IrStatement, scope: &Scope) -> Option<String> {
    match &statement.kind {
        StatementKind::NoOp => None,
        StatementKind::Block(body) => Some(node_group(
            &body
                .iter()
                .filter_map(|nested| branch_expression(nested, scope))
                .collect::<Vec<_>>(),
        )),
        StatementKind::Render(element) => Some(render(element, scope)),
        StatementKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let condition = rewrite(condition, scope);
            let then = node_group(
                &then_branch
                    .iter()
                    .filter_map(|nested| branch_expression(nested, scope))
                    .collect::<Vec<_>>(),
            );
            match else_branch {
                Some(else_branch) => {
                    let otherwise = node_group(
                        &else_branch
                            .iter()
                            .filter_map(|nested| branch_expression(nested, scope))
                            .collect::<Vec<_>>(),
                    );
                    Some(format!(
                        "plt.branch({condition}, () => {then}, () => {otherwise})"
                    ))
                }
                None => Some(format!("plt.branch({condition}, () => {then})")),
            }
        }
        StatementKind::For {
            binding,
            iterable,
            body,
        } => {
            let mut inner = scope.clone();
            inner.bind(
                binding,
                crate::codegen::state::local_access(&format!("item_{binding}")),
            );
            let item = node_group(
                &body
                    .iter()
                    .filter_map(|nested| branch_expression(nested, &inner))
                    .collect::<Vec<_>>(),
            );
            Some(format!(
                "plt.each({}, ({}) => {item})",
                rewrite(iterable, scope),
                crate::codegen::state::local_access(&format!("item_{binding}")),
            ))
        }
        // Side effects and declarations never produce a node.
        _ => None,
    }
}

/// Collapses a branch body into a single node expression.
fn node_group(nodes: &[String]) -> String {
    match nodes {
        [] => "null".to_string(),
        [single] => single.clone(),
        many => format!("plt.fragment([{}])", many.join(", ")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::IrProperty;

    fn property(name: &str, value: &str) -> IrProperty {
        IrProperty {
            name: name.into(),
            value: crate::ir::IrExpression {
                value: value.into(),
                dependencies: Vec::new(),
                span: crate::diagnostics::Span::default(),
            },
            span: crate::diagnostics::Span::default(),
        }
    }

    #[test]
    fn a_level_picks_the_matching_heading_tag() {
        assert_eq!(heading_tag(&[property("level", "1")]), "h1");
        assert_eq!(heading_tag(&[property("level", "3")]), "h3");
        assert_eq!(heading_tag(&[property("level", "6")]), "h6");
    }

    #[test]
    fn an_unknown_level_falls_back_to_h1() {
        assert_eq!(heading_tag(&[property("level", "9")]), "h6");
        assert_eq!(heading_tag(&[property("level", "n")]), "h1");
        assert_eq!(heading_tag(&[]), "h1");
    }

    #[test]
    fn a_body_that_reads_event_is_given_the_normalized_object() {
        let merged =
            merge_handlers(&[("input".into(), "s.field.value = event.value".into(), false)])
                .expect("handlers");
        assert!(merged.contains("const event = plt.event(raw);"), "{merged}");
        assert!(merged.contains("\"input\": async (raw) => {"), "{merged}");
    }

    #[test]
    fn a_body_that_ignores_event_stays_parameterless() {
        let merged = merge_handlers(&[("click".into(), "s.count.value += 1".into(), false)])
            .expect("handlers");
        assert!(!merged.contains("plt.event("), "{merged}");
        assert!(merged.contains("\"click\": async () => {"), "{merged}");
    }

    #[test]
    fn a_non_bubbling_event_is_wrapped_in_a_capture_descriptor() {
        let merged =
            merge_handlers(&[("scroll".into(), "s.y.value = event.position.y".into(), true)])
                .expect("handlers");
        assert!(merged.contains("\"scroll\": ["), "{merged}");
        assert!(merged.contains(", true]"), "{merged}");
        assert!(merged.contains("const event = plt.event(raw);"), "{merged}");
    }

    #[test]
    fn a_custom_handler_receives_the_payload_untouched() {
        let merged = merge_handlers(&[("plt:picked".into(), "s.note.value = event".into(), false)])
            .expect("handlers");
        assert!(!merged.contains("plt.event("), "{merged}");
        assert!(
            merged.contains("\"plt:picked\": async (event) => {"),
            "{merged}"
        );
    }

    #[test]
    fn a_lifecycle_handler_takes_no_parameter() {
        let merged = merge_handlers(&[("mount".into(), "s.count.value = 1".into(), false)])
            .expect("handlers");
        assert!(merged.contains("\"mount\": async () => {"), "{merged}");
        assert!(!merged.contains("plt.event("), "{merged}");
    }
}
