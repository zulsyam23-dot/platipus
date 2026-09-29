use crate::codegen::elements::camel_to_kebab;
use crate::ir::{IrStyleBlock, IrStyleState};

pub mod sheet;

pub fn property_name(name: &str) -> String {
    if name.starts_with("--") {
        return name.to_string();
    }
    camel_to_kebab(name)
}

/// CSS properties whose CSS spelling is not the kebab-case form of the
/// Platipus property name.
const PROPERTY_EXCEPTIONS: &[(&str, &str)] = &[
    ("align", "align-items"),
    ("justify", "justify-content"),
    ("columns", "grid-template-columns"),
    ("size", "font-size"),
    ("weight", "font-weight"),
];

/// Properties whose value is a length. A bare number on one of these is a
/// pixel count, because CSS rejects a unitless number and silently drops the
/// declaration.
const LENGTH_PROPERTIES: &[&str] = &[
    "border-radius",
    "font-size",
    "gap",
    "height",
    "margin",
    "max-height",
    "max-width",
    "min-height",
    "min-width",
    "padding",
    "top",
    "right",
    "bottom",
    "left",
    "width",
];

/// The CSS property name a Platipus property is emitted under.
pub fn css_property(property: &str) -> String {
    let name = property_name(property);
    for (platipus, css) in PROPERTY_EXCEPTIONS {
        if name == *platipus {
            return (*css).to_string();
        }
    }
    name
}

fn is_bare_number(value: &str) -> bool {
    !value.is_empty() && value.parse::<f64>().is_ok()
}

/// Removes the quotes that a Platipus string literal carries, because a
/// stylesheet writes the text itself. `align: "center"` styles with `center`;
/// a value that was not a string literal is already in stylesheet form.
fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(value)
}

/// The CSS value a Platipus property value is emitted as.
pub fn css_value(property: &str, value: &str) -> String {
    let name = css_property(property);
    let value = unquote(value);
    let bare = is_bare_number(value);
    if bare {
        if name == "grid-template-columns" {
            return format!("repeat({value}, minmax(0, 1fr))");
        }
        if LENGTH_PROPERTIES.contains(&name.as_str()) {
            return format!("{value}px");
        }
    }
    value.to_string()
}

/// A single CSS declaration, as a name and a value ready to be written out.
pub fn declaration(property: &str, value: &str) -> (String, String) {
    let name = css_property(property);
    let value = css_value(property, value);
    (name, value)
}

pub fn selector(state: IrStyleState) -> String {
    match state {
        IrStyleState::Normal => String::new(),
        IrStyleState::Hover => ":hover".to_string(),
        IrStyleState::Pressed => ":active".to_string(),
        IrStyleState::Focused => ":focus-within".to_string(),
        IrStyleState::Disabled => ":disabled".to_string(),
        IrStyleState::Selected => "[aria-selected=\"true\"]".to_string(),
    }
}

pub fn render_block(block: &IrStyleBlock, base: &str, depth: usize, out: &mut String) {
    let indent = "  ".repeat(depth);
    if !block.entries.is_empty() {
        out.push_str(&format!("{indent}{base} {{\n"));
        for entry in &block.entries {
            let (name, value) = declaration(&entry.name, &entry.value);
            out.push_str(&format!("{indent}  {name}: {value};\n"));
        }
        out.push_str(&format!("{indent}}}\n"));
    }
    for (state, nested) in &block.states {
        render_block(nested, &format!("{base}{}", selector(*state)), depth, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_nested_states() {
        let block = IrStyleBlock {
            name: None,
            entries: vec![crate::ir::IrStyleEntry {
                name: "backgroundColor".into(),
                value: "#111".into(),
                span: crate::diagnostics::Span::default(),
            }],
            states: vec![(
                IrStyleState::Hover,
                IrStyleBlock {
                    name: None,
                    entries: vec![crate::ir::IrStyleEntry {
                        name: "color".into(),
                        value: "#fff".into(),
                        span: crate::diagnostics::Span::default(),
                    }],
                    states: Vec::new(),
                    span: crate::diagnostics::Span::default(),
                },
            )],
            span: crate::diagnostics::Span::default(),
        };
        let mut out = String::new();
        render_block(&block, ".plt-root", 0, &mut out);
        assert_eq!(
            out,
            ".plt-root {\n  background-color: #111;\n}\n.plt-root:hover {\n  color: #fff;\n}\n"
        );
    }

    #[test]
    fn a_quoted_value_loses_its_quotes() {
        assert_eq!(css_value("align", "\"center\""), "center");
        assert_eq!(css_value("flex-direction", "\"column\""), "column");
        assert_eq!(css_value("background", "\"#111\""), "#111");
        assert_eq!(css_value("width", "\"50%\""), "50%");
    }

    #[test]
    fn a_bare_number_on_a_length_becomes_pixels() {
        assert_eq!(css_value("padding", "16"), "16px");
        assert_eq!(css_value("gap", "8"), "8px");
        assert_eq!(css_value("min-width", "240.5"), "240.5px");
    }

    #[test]
    fn a_value_that_already_carries_a_unit_is_untouched() {
        assert_eq!(css_value("padding", "1rem"), "1rem");
        assert_eq!(css_value("padding", "50%"), "50%");
        assert_eq!(
            css_value("padding", "calc(100% - 2rem)"),
            "calc(100% - 2rem)"
        );
        assert_eq!(css_value("width", "auto"), "auto");
    }

    #[test]
    fn a_bare_number_on_a_unitless_property_is_untouched() {
        assert_eq!(css_value("opacity", "0.5"), "0.5");
        assert_eq!(css_value("z-index", "3"), "3");
    }

    #[test]
    fn property_names_follow_the_stylesheet_spelling() {
        assert_eq!(css_property("backgroundColor"), "background-color");
        assert_eq!(css_property("align"), "align-items");
        assert_eq!(css_property("justify"), "justify-content");
        assert_eq!(css_property("columns"), "grid-template-columns");
        assert_eq!(css_property("size"), "font-size");
        assert_eq!(css_property("weight"), "font-weight");
        assert_eq!(css_value("size", "20"), "20px");
        assert_eq!(css_property("--brand-accent"), "--brand-accent");
    }

    #[test]
    fn a_column_count_becomes_a_track_list() {
        assert_eq!(css_value("columns", "3"), "repeat(3, minmax(0, 1fr))");
        assert_eq!(css_value("columns", "repeat(2, 120px)"), "repeat(2, 120px)");
    }

    #[test]
    fn render_block_adds_the_missing_unit() {
        let block = IrStyleBlock {
            name: None,
            entries: vec![crate::ir::IrStyleEntry {
                name: "padding".into(),
                value: "16".into(),
                span: crate::diagnostics::Span::default(),
            }],
            states: Vec::new(),
            span: crate::diagnostics::Span::default(),
        };
        let mut out = String::new();
        render_block(&block, ".plt-root", 0, &mut out);
        assert_eq!(out, ".plt-root {\n  padding: 16px;\n}\n");
    }
}
