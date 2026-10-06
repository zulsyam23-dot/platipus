use crate::codegen::components::{
    inline_style, inline_style_class, override_class, style_properties,
};
use crate::codegen::style::declaration;
use platipus_ir::element::{ElementItem, IrResponsiveBlock, IrResponsiveEntry};
use platipus_ir::{IrElement, IrModule, IrStatement, StatementKind};

pub fn render(module: &IrModule) -> String {
    // The base stylesheet comes first so a program's own rules, and the cascade
    // inside a `style` block, both land on top of it.
    let mut out = crate::codegen::style::sheet::stylesheet();
    // Theme switching has no runtime yet, so the first theme a program declares
    // is the palette it is built with. The rest are still emitted under their own
    // attribute, which is where a host page selects one.
    for (index, theme) in module.themes.iter().enumerate() {
        let name = theme.name.to_lowercase();
        let mut selectors = vec![format!("[data-plt-theme=\"{name}\"]")];
        if index == 0 {
            selectors.push(":root".to_string());
        }
        out.push_str(&format!("{} {{\n", selectors.join(",\n")));
        for token in &theme.tokens {
            // A token path names a custom property rather than a CSS property.
            // Written as a plain property it is dropped by the browser, which
            // leaves a theme that parses and then does nothing at all.
            out.push_str(&format!(
                "  {}: {};\n",
                custom_property(&token.path),
                token.value
            ));
        }
        out.push_str("}\n");
    }
    // A top-level `style { }` block styles the application root wrapper.
    for block in &module.styles {
        crate::codegen::style::render_block(block, ".plt-root", 0, &mut out);
    }
    for component in &module.components {
        // Every root, not just the first. A component body may hold several
        // siblings, and taking one of them leaves the rest with a class in the
        // markup that no rule in the sheet names: the element is then styled by
        // nothing, which is invisible until it is the one being looked at.
        collect(&component.body, module, &mut out);
        // A function may build an element and return it, so the rules for that
        // element have to be in the stylesheet too.
        for function in &component.functions {
            let mut nested = Vec::new();
            rendered_elements(&function.body, &mut nested);
            for element in nested {
                collect_element(element, module, &mut out);
            }
        }
    }
    out
}

/// The custom property a theme token sets.
///
/// A theme has to name the same tokens the base stylesheet reads, or it overrides
/// nothing: the rules below are written in terms of `--plt-bg` and `--plt-text`,
/// so a token called `background` has to land on `--plt-bg` to be visible. The
/// table is the short vocabulary a program is expected to use; anything else
/// passes through as a namespaced property, which is how a program introduces a
/// token of its own for its own rules to read.
fn custom_property(path: &str) -> String {
    const ALIASES: &[(&str, &str)] = &[
        ("background", "bg"),
        ("foreground", "text"),
        ("surface", "surface"),
        ("surfaceAlt", "surface-2"),
        ("border", "border"),
        ("muted", "muted"),
        ("accent", "accent"),
        ("accentText", "accent-text"),
        ("danger", "danger"),
        ("success", "success"),
        ("radius", "radius"),
        ("gap", "gap"),
    ];
    for (name, token) in ALIASES {
        if path == *name {
            return format!("--plt-{token}");
        }
    }
    let mut name = String::from("--plt");
    for segment in path.split('.') {
        name.push('-');
        name.push_str(&crate::codegen::elements::camel_to_kebab(segment));
    }
    name
}

/// The selectors an element's own rules are emitted under: the classes
/// generated for this instance, in specificity order, or the primitive's
/// layout class when the element generates none.
///
/// A generated class is used in preference to the primitive's class because
/// that class is shared by every instance of the primitive: a rule written
/// under it would restyle all of them, not just this element.
fn selectors_of(element: &IrElement) -> Vec<String> {
    let mut selectors: Vec<String> = Vec::new();
    if let Some(style) = inline_style(element) {
        selectors.push(format!(".{}", inline_style_class(style)));
    }
    if let Some(class) = override_class(element) {
        selectors.push(format!(".{}", class));
    }
    if selectors.is_empty() {
        selectors.push(format!(
            ".{}",
            crate::codegen::elements::class_of(&element.name)
        ));
    }
    selectors
}

fn collect(items: &[ElementItem], module: &IrModule, out: &mut String) {
    for element in nested_elements(items) {
        collect_element(element, module, out);
    }
}

/// Emits every rule that describes one element, then recurses into the elements
/// it contains.
fn collect_element(element: &IrElement, module: &IrModule, out: &mut String) {
    // Style properties become declarations on a class generated for this
    // element, rather than attributes on the node.
    let properties = style_properties(element);
    if !properties.is_empty() {
        if let Some(class) = override_class(element) {
            out.push_str(&format!(".{class} {{\n"));
            for property in properties {
                let (name, value) = declaration(&property.name, &property.value.value);
                out.push_str(&format!("  {name}: {value};\n"));
            }
            out.push_str("}\n");
        }
    }
    // A `style { }` block and a `responsive { }` block are siblings in the
    // element body rather than children: both describe the element that owns
    // them, so they are read here instead of from the child list.
    if let Some(body) = element.body.as_deref() {
        for item in &body.items {
            match item {
                ElementItem::Style(style) => crate::codegen::style::render_block(
                    style,
                    &format!(".{}", inline_style_class(style)),
                    0,
                    out,
                ),
                ElementItem::Responsive(block) => {
                    render_responsive(block, &selectors_of(element), module, out)
                }
                _ => {}
            }
        }
    }
    if let Some(body) = element.body.as_deref() {
        collect(&body.items, module, out);
    }
}

/// The elements directly contained in a body, including the ones a template
/// position such as `if` or `for` renders. An element inside a template is an
/// ordinary element and needs the same rules as one written inline.
fn nested_elements(items: &[ElementItem]) -> Vec<&IrElement> {
    let mut nested = Vec::new();
    for item in items {
        match item {
            ElementItem::Child(element) => nested.push(element),
            ElementItem::Statement(statement) => {
                rendered_elements(std::slice::from_ref(statement), &mut nested)
            }
            ElementItem::Style(_) | ElementItem::Responsive(_) => {}
        }
    }
    nested
}

/// Collects the elements a statement renders, following nested statements but
/// not descending into an element body, which the caller walks itself.
fn rendered_elements<'a>(statements: &'a [IrStatement], nested: &mut Vec<&'a IrElement>) {
    for statement in statements {
        match &statement.kind {
            StatementKind::Render(element) => nested.push(element),
            StatementKind::If {
                then_branch,
                else_branch,
                ..
            } => {
                rendered_elements(then_branch, nested);
                if let Some(else_branch) = else_branch {
                    rendered_elements(else_branch, nested);
                }
            }
            StatementKind::For { body, .. } | StatementKind::Block(body) => {
                rendered_elements(body, nested)
            }
            StatementKind::Try { binding: _, body, handler } => {
                rendered_elements(body, nested);
                rendered_elements(handler, nested);
            }
            _ => {}
        }
    }
}

fn render_responsive(
    block: &IrResponsiveBlock,
    selectors: &[String],
    module: &IrModule,
    out: &mut String,
) {
    // Group the entries by breakpoint so each `@media` block is emitted once.
    let mut order: Vec<&str> = Vec::new();
    let mut grouped: Vec<(&str, Vec<&IrResponsiveEntry>)> = Vec::new();
    for entry in &block.entries {
        let breakpoint = entry_breakpoint(entry);
        match order.iter().position(|name| *name == breakpoint) {
            Some(index) => grouped[index].1.push(entry),
            None => {
                order.push(breakpoint);
                grouped.push((breakpoint, vec![entry]));
            }
        }
    }
    for (breakpoint, entries) in grouped {
        // `mobile` is the base layout, so its entries are written as plain
        // rules and win over the primitive's own base rule by source order.
        let media = breakpoint_width(breakpoint);
        match media {
            Some(width) => out.push_str(&format!("@media (min-width: {width}) {{\n")),
            None => out.push_str("/* mobile */\n"),
        }
        let indent = if media.is_some() { "  " } else { "" };
        for entry in entries {
            match entry {
                IrResponsiveEntry::Property { name, value, .. } => {
                    let (name, value) = declaration(name, value);
                    for selector in selectors {
                        out.push_str(&format!("{indent}{selector} {{ {name}: {value}; }}\n"));
                    }
                }
                IrResponsiveEntry::Named { name, .. } => {
                    let Some(styled) = module
                        .styles
                        .iter()
                        .find(|block| block.name.as_deref() == Some(name.as_str()))
                    else {
                        continue;
                    };
                    for selector in selectors {
                        crate::codegen::style::render_block(
                            styled,
                            selector,
                            usize::from(media.is_some()),
                            out,
                        );
                    }
                }
            }
        }
        if media.is_some() {
            out.push_str("}\n");
        }
    }
}

fn entry_breakpoint(entry: &IrResponsiveEntry) -> &str {
    match entry {
        IrResponsiveEntry::Property { breakpoint, .. }
        | IrResponsiveEntry::Named { breakpoint, .. } => breakpoint,
    }
}

/// The `@media` minimum width for a breakpoint, or `None` for `mobile`, whose
/// entries are the base rules.
fn breakpoint_width(name: &str) -> Option<&'static str> {
    if name.eq_ignore_ascii_case(crate::codegen::layout::BASE_BREAKPOINT) {
        return None;
    }
    crate::codegen::layout::breakpoint_of(name)
}
