use platipus_ir::{ElementKind, IrElement};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyTarget {
    Attribute,
    DomProperty,
    BooleanAttribute,
    Style,
}

/// Maps a Platipus element name onto its HTML tag.
pub fn tag_of(name: &str) -> &'static str {
    match name {
        "Page" => "div",
        "Container" => "div",
        "Column" => "div",
        "Row" => "div",
        "Stack" => "div",
        "Scroll" => "div",
        "Card" => "div",
        "Grid" => "div",
        "Panel" => "div",
        "Splitter" => "div",
        "Spacer" => "div",
        "Viewport" => "div",
        "Form" => "form",
        "Field" => "div",
        "Input" => "input",
        "Textarea" => "textarea",
        "Select" => "select",
        "Option" => "option",
        "Checkbox" => "input",
        "Radio" => "input",
        "Switch" => "input",
        "Slider" => "input",
        "Date" => "input",
        "Time" => "input",
        "File" => "input",
        "Color" => "input",
        "Button" => "button",
        "Link" => "a",
        "Menu" => "ul",
        "MenuItem" => "li",
        "Navigation" => "nav",
        "ContextMenu" => "div",
        "Sidebar" => "aside",
        "Toolbar" => "div",
        "Tabs" => "div",
        "Tab" => "div",
        "Modal" => "dialog",
        "Sheet" => "div",
        "Toast" => "div",
        "Tooltip" => "div",
        "Dialog" => "dialog",
        "Popover" => "div",
        "Text" => "span",
        "Image" => "img",
        "Icon" => "i",
        "Video" => "video",
        "Audio" => "audio",
        "Canvas" => "canvas",
        "List" => "ul",
        "Table" => "table",
        "Tree" => "ul",
        "DataGrid" => "table",
        "TableRow" => "tr",
        "Cell" => "td",
        "Header" => "th",
        "Editor" => "div",
        "CodeEditor" => "textarea",
        "Command" => "button",
        "Heading" => "h1",
        "Loader" => "div",
        "Progress" => "progress",
        "Spinner" => "div",
        _ => "div",
    }
}

pub fn is_component(element: &IrElement) -> bool {
    element.kind == ElementKind::Component
}

/// The `type` an input element carries unless the element states one itself.
/// HTML tells a checkbox, a slider, a date field, and a plain text field apart
/// by this attribute alone, so compiling them all to a bare `input` hands the
/// browser something it cannot read: every one of them arrives as a text box.
pub fn input_type_of(name: &str) -> Option<&'static str> {
    match name {
        "Input" => Some("text"),
        "Checkbox" | "Switch" => Some("checkbox"),
        "Radio" => Some("radio"),
        "Slider" => Some("range"),
        "Date" => Some("date"),
        "Time" => Some("time"),
        "File" => Some("file"),
        "Color" => Some("color"),
        _ => None,
    }
}

/// Style properties that belong to one element rather than to every layout
/// primitive, kept next to the shared list in the semantic checker because
/// they are only meaningful there.
const ELEMENT_STYLE_PROPERTIES: &[(&str, &str)] = &[
    ("Grid", "columns"),
    ("Text", "size"),
    ("Text", "weight"),
    ("Heading", "size"),
    ("Heading", "weight"),
];

/// Whether a property on this element is a CSS declaration rather than an
/// attribute. Only styled boxes take part: a `width: 100` on a `Video` stays an
/// HTML attribute, which is the spelling that element expects. The property
/// list is the one the semantic checker uses, so a name that is not a property
/// of the element is rejected before codegen ever sees it.
pub fn is_style_property(element: &str, property: &str) -> bool {
    let styled = crate::codegen::layout::is_layout(element)
        || platipus_semantic::element::builtins::BOX_ELEMENTS.contains(&element);
    if !styled {
        return false;
    }
    let shared = platipus_semantic::element::builtins::LAYOUT_PROPERTIES
        .iter()
        .any(|(candidate, _)| *candidate == property);
    let own = ELEMENT_STYLE_PROPERTIES
        .iter()
        .any(|(owner, candidate)| *owner == element && *candidate == property);
    shared || own
}

pub fn property_target(element: &str, property: &str) -> PropertyTarget {
    if is_style_property(element, property) {
        return PropertyTarget::Style;
    }
    match property {
        "disabled" | "checked" | "selected" | "multiple" | "required" | "autofocus" | "loading"
        | "readOnly" | "open" | "hidden" | "novalidate" | "autoplay" | "controls" | "loop"
        | "reversed" | "default" => PropertyTarget::BooleanAttribute,
        "value" | "muted" | "paused" | "currentTime" | "indeterminate" => {
            PropertyTarget::DomProperty
        }
        _ => PropertyTarget::Attribute,
    }
}

/// HTML attributes whose spelling is not simply the kebab-case form of the
/// Platipus property name.
const ATTRIBUTE_EXCEPTIONS: &[(&str, &str)] = &[
    ("maxlength", "maxlength"),
    ("max-length", "maxlength"),
    ("readonly", "readonly"),
    ("read-only", "readonly"),
    ("novalidate", "novalidate"),
    ("no-validate", "novalidate"),
    ("autofocus", "autofocus"),
    ("autoplay", "autoplay"),
    ("tabindex", "tabindex"),
    ("tab-index", "tabindex"),
    ("colspan", "colspan"),
    ("col-span", "colspan"),
    ("rowspan", "rowspan"),
    ("row-span", "rowspan"),
    ("accesskey", "accesskey"),
    ("access-key", "accesskey"),
];

pub fn attribute_name(property: &str) -> String {
    let kebab = camel_to_kebab(property);
    for (platipus, html) in ATTRIBUTE_EXCEPTIONS {
        if kebab == *platipus {
            return (*html).to_string();
        }
    }
    kebab
}

pub fn camel_to_kebab(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for ch in name.chars() {
        if ch.is_ascii_uppercase() {
            if !out.is_empty() {
                out.push('-');
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

pub fn accepts_text(element: &str) -> bool {
    tag_of(element) != "img"
}

pub fn class_of(element: &str) -> String {
    format!("plt-{}", camel_to_kebab(element))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_primitives_to_tags() {
        assert_eq!(tag_of("Button"), "button");
        assert_eq!(tag_of("Text"), "span");
        assert_eq!(tag_of("Navigation"), "nav");
        assert_eq!(tag_of("Sidebar"), "aside");
        assert_eq!(tag_of("Dialog"), "dialog");
        assert_eq!(tag_of("Video"), "video");
        assert_eq!(tag_of("Canvas"), "canvas");
        assert_eq!(tag_of("CodeEditor"), "textarea");
        assert_eq!(tag_of("TableRow"), "tr");
        assert_eq!(tag_of("Cell"), "td");
        assert_eq!(tag_of("Header"), "th");
        assert_eq!(tag_of("Spacer"), "div");
        assert_eq!(tag_of("Unknown"), "div");
    }

    #[test]
    fn converts_camel_to_kebab() {
        assert_eq!(camel_to_kebab("backgroundColor"), "background-color");
        assert_eq!(attribute_name("maxLength"), "maxlength");
        assert_eq!(attribute_name("readOnly"), "readonly");
        assert_eq!(attribute_name("colSpan"), "colspan");
    }

    #[test]
    fn boolean_properties_render_as_bare_attributes() {
        assert_eq!(
            property_target("Button", "loading"),
            PropertyTarget::BooleanAttribute
        );
        assert_eq!(
            property_target("Input", "value"),
            PropertyTarget::DomProperty
        );
        assert_eq!(
            property_target("Input", "placeholder"),
            PropertyTarget::Attribute
        );
    }

    #[test]
    fn layout_properties_become_css_declarations() {
        assert_eq!(property_target("Stack", "gap"), PropertyTarget::Style);
        assert_eq!(property_target("Grid", "columns"), PropertyTarget::Style);
    }

    #[test]
    fn only_layout_primitives_take_style_properties() {
        // The same spelling stays an attribute on a media element, whose
        // `width` is an HTML attribute rather than a layout hint.
        assert_eq!(property_target("Image", "width"), PropertyTarget::Attribute);
    }
}
