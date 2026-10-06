/// Layout primitives that carry structural behaviour in the generated DOM,
/// mapped onto the base rules the stylesheet emits for them.
pub const LAYOUT_PRIMITIVES: &[(&str, &str)] = &[
    ("Page", "display: block; min-height: 100%;"),
    ("Container", "display: block;"),
    ("Column", "display: flex; flex-direction: column;"),
    ("Row", "display: flex; flex-direction: row;"),
    (
        "Stack",
        "display: flex; flex-direction: column; gap: 0.5rem;",
    ),
    ("Scroll", "overflow: auto;"),
    ("Card", "display: block;"),
    ("Grid", "display: grid;"),
    ("Panel", "display: block;"),
    ("Viewport", "overflow: auto;"),
    ("Splitter", "display: flex;"),
    ("Spacer", "display: block;"),
];

pub fn is_layout(element: &str) -> bool {
    LAYOUT_PRIMITIVES.iter().any(|(name, _)| *name == element)
}

pub fn base_rule(element: &str) -> Option<&'static str> {
    LAYOUT_PRIMITIVES
        .iter()
        .find(|(name, _)| *name == element)
        .map(|(_, rule)| *rule)
}

/// The minimum viewport width at which a `responsive { }` breakpoint applies,
/// named the way the block spells it. `mobile` is the base layout rather than a
/// minimum width, so it is absent here: its entries are written without a
/// `@media` wrapper.
pub const BREAKPOINTS: &[(&str, &str)] = &[("tablet", "48rem"), ("desktop", "64rem")];

/// The base breakpoint, whose entries apply at every viewport width.
pub const BASE_BREAKPOINT: &str = "mobile";

/// The `@media` minimum width for a breakpoint, or `None` for the base one.
pub fn breakpoint_of(name: &str) -> Option<&'static str> {
    BREAKPOINTS
        .iter()
        .find(|(breakpoint, _)| breakpoint.eq_ignore_ascii_case(name))
        .map(|(_, width)| *width)
}
