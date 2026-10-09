/// `maxWidth` to `max-width`.
///
/// An element property is written as an identifier, so it cannot carry the
/// hyphen its CSS spelling uses. This is the one conversion between the two, and
/// both the property lookup and the style block go through it.
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

#[derive(Debug, Clone)]
pub struct ElementProperty {
    pub name: String,
    pub r#type: String,
    pub required: bool,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct ElementDefinition {
    pub name: String,
    pub is_component: bool,
    pub properties: Vec<ElementProperty>,
    pub accepts_children: bool,
    pub text_content: bool,
    pub category: String,
}

impl ElementDefinition {
    pub fn new(name: &str, category: &str) -> Self {
        Self {
            name: name.to_string(),
            is_component: false,
            properties: Vec::new(),
            accepts_children: true,
            text_content: false,
            category: category.to_string(),
        }
    }

    pub fn with_property(mut self, name: &str, r#type: &str, required: bool) -> Self {
        self.properties.push(ElementProperty {
            name: name.to_string(),
            r#type: r#type.to_string(),
            required,
            description: String::new(),
        });
        self
    }

    pub fn text(mut self) -> Self {
        self.text_content = true;
        self
    }

    pub fn leaf(mut self) -> Self {
        self.accepts_children = false;
        self
    }
}

#[derive(Debug, Default)]
pub struct ElementRegistry {
    elements: Vec<ElementDefinition>,
}

impl ElementRegistry {
    pub fn new() -> Self {
        Self {
            elements: builtins::all().to_vec(),
        }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn register(&mut self, element: ElementDefinition) {
        self.elements
            .retain(|existing| existing.name != element.name);
        self.elements.push(element);
    }

    pub fn get(&self, name: &str) -> Option<&ElementDefinition> {
        self.elements.iter().find(|element| element.name == name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.elements.iter().map(|element| element.name.as_str())
    }

    pub fn all(&self) -> &[ElementDefinition] {
        &self.elements
    }

    pub fn accepts_children(&self, name: &str) -> bool {
        self.get(name).map(|e| e.accepts_children).unwrap_or(true)
    }

    pub fn accepts_text(&self, name: &str) -> bool {
        self.get(name).map(|e| e.text_content).unwrap_or(true)
    }

    pub fn unknown_property(&self, name: &str, property: &str) -> bool {
        match self.get(name) {
            Some(definition) => {
                // Properties are registered in their CSS spelling, which may
                // contain a hyphen, while the language has no way to write one:
                // a property name is an identifier. `maxWidth` is therefore the
                // only way to spell `max-width`, and the lookup has to make the
                // same conversion the style block already makes.
                let wanted = camel_to_kebab(property);
                !definition
                    .properties
                    .iter()
                    .any(|candidate| candidate.name == property || candidate.name == wanted)
            }
            None => false,
        }
    }

    pub fn missing_required(&self, name: &str, provided: &[String]) -> Vec<&str> {
        let Some(definition) = self.get(name) else {
            return Vec::new();
        };
        definition
            .properties
            .iter()
            .filter(|property| property.required && !provided.iter().any(|p| p == &property.name))
            .map(|property| property.name.as_str())
            .collect()
    }

    pub fn suggestions(&self, name: &str) -> Vec<&str> {
        let mut matches: Vec<&str> = self
            .names()
            .filter(|candidate| similar(candidate, name))
            .collect();
        matches.sort_unstable();
        matches
    }
}

fn similar(candidate: &str, name: &str) -> bool {
    if candidate.eq_ignore_ascii_case(name) {
        return true;
    }
    let lower_candidate = candidate.to_ascii_lowercase();
    let lower_name = name.to_ascii_lowercase();
    if lower_name.starts_with(&lower_candidate)
        || lower_candidate.starts_with(&lower_name)
        || lower_candidate.contains(&lower_name)
    {
        return true;
    }
    let longest = lower_candidate
        .chars()
        .count()
        .max(lower_name.chars().count());
    longest > 0 && edit_distance(&lower_candidate, &lower_name) * 3 <= longest
}

fn edit_distance(left: &str, right: &str) -> usize {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    let mut current = vec![0usize; right.len() + 1];
    for (row, left_char) in left.iter().enumerate() {
        current[0] = row + 1;
        for (column, right_char) in right.iter().enumerate() {
            let substitution = previous[column] + usize::from(left_char != right_char);
            let insertion = current[column] + 1;
            let deletion = previous[column + 1] + 1;
            current[column + 1] = substitution.min(insertion).min(deletion);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}

pub mod builtins {
    use super::{ElementDefinition, ElementRegistry};

    pub const ALL: &[(&str, &str)] = &[
        ("Page", "layout"),
        ("Container", "layout"),
        ("Text", "text"),
        ("Heading", "content"),
        ("Column", "layout"),
        ("Row", "layout"),
        ("Stack", "layout"),
        ("Scroll", "layout"),
        ("Card", "layout"),
        ("Grid", "layout"),
        ("Panel", "layout"),
        ("Splitter", "layout"),
        ("Spacer", "layout"),
        ("Viewport", "layout"),
        ("Form", "form"),
        ("Field", "form"),
        ("Input", "form"),
        ("Textarea", "form"),
        ("Select", "form"),
        ("Option", "form"),
        ("Checkbox", "form"),
        ("Radio", "form"),
        ("Switch", "form"),
        ("Slider", "form"),
        ("Date", "form"),
        ("Time", "form"),
        ("File", "form"),
        ("Color", "form"),
        ("Button", "form"),
        ("Link", "navigation"),
        ("Menu", "navigation"),
        ("MenuItem", "navigation"),
        ("Navigation", "navigation"),
        ("ContextMenu", "navigation"),
        ("Sidebar", "navigation"),
        ("Toolbar", "navigation"),
        ("Tabs", "navigation"),
        ("Tab", "navigation"),
        ("Modal", "overlay"),
        ("Sheet", "overlay"),
        ("Toast", "overlay"),
        ("Tooltip", "overlay"),
        ("Dialog", "overlay"),
        ("Popover", "overlay"),
        ("Image", "media"),
        ("Icon", "media"),
        ("Video", "media"),
        ("Audio", "media"),
        ("Canvas", "media"),
        ("List", "data"),
        ("Table", "data"),
        ("Tree", "data"),
        ("DataGrid", "data"),
        ("TableRow", "data"),
        ("Cell", "data"),
        ("Header", "data"),
        ("Loader", "data"),
        ("Progress", "feedback"),
        ("Spinner", "feedback"),
        ("Editor", "advanced"),
        ("CodeEditor", "advanced"),
        ("Command", "action"),
    ];

    /// Box-like primitives that are not layout categories but that take the
    /// same shared layout properties: they render a box that can be padded,
    /// sized, and coloured the way a `Column` or `Container` can. Media
    /// elements and form fields are absent, because on those `width` and
    /// `height` are HTML attributes with their own meaning.
    pub const BOX_ELEMENTS: &[&str] = &[
        "Text",
        "Heading",
        "Field",
        "Form",
        "Menu",
        "Tabs",
        "Tab",
        "Sheet",
        "Toast",
        "Tooltip",
        "Navigation",
        "Sidebar",
        "Toolbar",
        "Dialog",
        "Popover",
        "Tree",
        "DataGrid",
        "List",
        "Table",
        "Loader",
        "Spinner",
        "Modal",
        "ContextMenu",
        "Command",
        "Splitter",
        "Panel",
        "Spacer",
        "Viewport",
        "Editor",
        "CodeEditor",
    ];

    /// Properties every layout primitive accepts. On these elements a
    /// `padding: 16` is styling for the box the element generates, and the
    /// compiler emits it as a CSS declaration rather than an HTML attribute.
    /// Codegen reads this same list to decide how to route such a property, so
    /// the language and the emitters cannot drift apart.
    pub const LAYOUT_PROPERTIES: &[(&str, &str)] = &[
        ("align", "String"),
        ("background", "Color"),
        ("border-radius", "Length"),
        ("color", "Color"),
        ("flex-direction", "String"),
        ("gap", "Length"),
        ("height", "Length"),
        ("justify", "String"),
        ("margin", "Length"),
        ("max-height", "Length"),
        ("max-width", "Length"),
        ("min-height", "Length"),
        ("min-width", "Length"),
        ("overflow", "String"),
        ("padding", "Length"),
        ("transition", "String"),
        ("width", "Length"),
    ];

    pub fn all() -> Vec<ElementDefinition> {
        ALL.iter()
            .map(|(name, category)| {
                let mut definition = ElementDefinition::new(name, category);
                if *category == "layout" || BOX_ELEMENTS.contains(name) {
                    for (property, type_name) in LAYOUT_PROPERTIES {
                        definition = definition.with_property(property, type_name, false);
                    }
                }
                match *name {
                    "Column" | "Row" | "Stack" => {}
                    "Grid" => {
                        definition = definition.with_property("columns", "Int", false);
                    }
                    "Heading" => {
                        definition = definition
                            .text()
                            .with_property("level", "Int", false)
                            .with_property("size", "Length", false)
                            .with_property("weight", "Int", false);
                    }
                    "Text" => {
                        definition = definition
                            .text()
                            .with_property("size", "Length", false)
                            .with_property("weight", "Int", false);
                    }
                    "Link" => {
                        definition = definition
                            .text()
                            .with_property("href", "String", false)
                            .with_property("target", "String", false);
                    }
                    "MenuItem" | "Tab" => {
                        definition = definition.text();
                    }
                    "Input" | "Textarea" | "Select" => {
                        definition = definition
                            .with_property("value", "String", false)
                            .with_property("placeholder", "String", false);
                    }
                    "Date" | "Time" => {
                        definition = definition
                            .leaf()
                            .with_property("value", "String", false)
                            .with_property("min", "String", false)
                            .with_property("max", "String", false);
                    }
                    "File" => {
                        definition = definition
                            .leaf()
                            .with_property("accept", "String", false)
                            .with_property("multiple", "Bool", false);
                    }
                    "Color" => {
                        definition = definition.leaf().with_property("value", "String", false);
                    }
                    "Field" => {}
                    "Form" => {
                        definition = definition
                            .with_property("action", "String", false)
                            .with_property("method", "String", false)
                            .with_property("novalidate", "Bool", false);
                    }
                    "Tooltip" => {
                        definition = definition.text();
                    }
                    "Checkbox" | "Radio" => {
                        definition = definition
                            .leaf()
                            .with_property("checked", "Bool", false)
                            .with_property("name", "String", false)
                            .with_property("value", "String", false);
                    }
                    "Switch" => {
                        definition = definition.leaf().with_property("checked", "Bool", false);
                    }
                    "Slider" => {
                        definition = definition
                            .leaf()
                            .with_property("min", "Float", false)
                            .with_property("max", "Float", false)
                            .with_property("step", "Float", false)
                            .with_property("value", "Float", false);
                    }
                    "Button" => {
                        definition = definition
                            .text()
                            .with_property("disabled", "Bool", false)
                            .with_property("loading", "Bool", false);
                    }
                    "Command" => {
                        definition = definition
                            .text()
                            .with_property("shortcut", "String", false)
                            .with_property("disabled", "Bool", false);
                    }
                    "ContextMenu" | "Dialog" | "Popover" | "Sidebar" => {
                        definition = definition.with_property("open", "Bool", false);
                    }
                    "Image" => {
                        definition = definition.leaf().with_property("src", "String", true);
                    }
                    "Icon" => {
                        definition = definition.leaf().with_property("name", "String", true);
                    }
                    "Video" => {
                        definition = definition
                            .leaf()
                            .with_property("src", "String", true)
                            .with_property("poster", "String", false)
                            .with_property("controls", "Bool", false)
                            .with_property("autoplay", "Bool", false)
                            .with_property("loop", "Bool", false)
                            .with_property("muted", "Bool", false);
                    }
                    "Audio" => {
                        definition = definition
                            .leaf()
                            .with_property("src", "String", true)
                            .with_property("controls", "Bool", false)
                            .with_property("autoplay", "Bool", false)
                            .with_property("loop", "Bool", false);
                    }
                    "Canvas" => {
                        definition = definition
                            .leaf()
                            .with_property("width", "Int", false)
                            .with_property("height", "Int", false)
                            .with_property("id", "String", false);
                    }
                    "Progress" => {
                        definition = definition.leaf().with_property("value", "Float", true);
                    }
                    "Option" => {
                        definition = definition
                            .leaf()
                            .text()
                            .with_property("value", "String", true);
                    }
                    "Cell" | "Header" => {
                        definition = definition
                            .leaf()
                            .text()
                            .with_property("colSpan", "Int", false);
                    }
                    "CodeEditor" => {
                        definition = definition
                            .leaf()
                            .text()
                            .with_property("value", "String", false)
                            .with_property("placeholder", "String", false)
                            .with_property("language", "String", false)
                            .with_property("highlight", "String", false);
                    }
                    "Editor" => {
                        definition = definition
                            .leaf()
                            .text()
                            .with_property("value", "String", false);
                    }
                    "Loader" | "Spinner" => {
                        definition = definition.leaf();
                    }
                    _ => {}
                }
                definition
            })
            .collect()
    }

    pub fn registry() -> ElementRegistry {
        ElementRegistry::new()
    }
}
