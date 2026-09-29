use super::expression::{Expression, Identifier};
use crate::diagnostics::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StyleState {
    Normal,
    Hover,
    Pressed,
    Focused,
    Disabled,
    Selected,
}

impl StyleState {
    pub const fn as_str(self) -> &'static str {
        match self {
            StyleState::Normal => "normal",
            StyleState::Hover => "hover",
            StyleState::Pressed => "pressed",
            StyleState::Focused => "focused",
            StyleState::Disabled => "disabled",
            StyleState::Selected => "selected",
        }
    }

    pub const fn selector(self) -> &'static str {
        match self {
            StyleState::Normal => "",
            StyleState::Hover => ":hover",
            StyleState::Pressed => ":active",
            StyleState::Focused => ":focus",
            StyleState::Disabled => ":disabled",
            StyleState::Selected => "[aria-selected=\"true\"]",
        }
    }

    pub const fn all() -> &'static [StyleState] {
        &[
            StyleState::Normal,
            StyleState::Hover,
            StyleState::Pressed,
            StyleState::Focused,
            StyleState::Disabled,
            StyleState::Selected,
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StyleEntry {
    Property {
        name: Identifier,
        value: Expression,
        span: Span,
    },
    Reference {
        name: Identifier,
        span: Span,
    },
}

impl StyleEntry {
    pub fn name(&self) -> &str {
        match self {
            StyleEntry::Property { name, .. } | StyleEntry::Reference { name, .. } => name.as_str(),
        }
    }

    pub fn span(&self) -> Span {
        match self {
            StyleEntry::Property { span, .. } | StyleEntry::Reference { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyleBlock {
    pub entries: Vec<StyleEntry>,
    pub groups: Vec<StyleStateGroup>,
    pub span: Span,
}

impl StyleBlock {
    pub fn new(entries: Vec<StyleEntry>, groups: Vec<StyleStateGroup>, span: Span) -> Self {
        Self {
            entries,
            groups,
            span,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.groups.is_empty()
    }

    pub fn group(&self, state: StyleState) -> Option<&StyleStateGroup> {
        self.groups.iter().find(|group| group.state == state)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyleStateGroup {
    pub state: StyleState,
    pub block: StyleBlock,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyleDefinition {
    pub name: Identifier,
    pub block: StyleBlock,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Breakpoint {
    Mobile,
    Tablet,
    Desktop,
}

impl Breakpoint {
    pub const fn as_str(self) -> &'static str {
        match self {
            Breakpoint::Mobile => "mobile",
            Breakpoint::Tablet => "tablet",
            Breakpoint::Desktop => "desktop",
        }
    }

    /// The viewport width at which this breakpoint starts applying. `mobile` is
    /// the base layout, so it has no minimum of its own.
    pub const fn min_width(self) -> Option<&'static str> {
        match self {
            Breakpoint::Mobile => None,
            Breakpoint::Tablet => Some("48rem"),
            Breakpoint::Desktop => Some("64rem"),
        }
    }

    pub const fn all() -> &'static [Breakpoint] {
        &[Breakpoint::Mobile, Breakpoint::Tablet, Breakpoint::Desktop]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResponsiveEntry {
    pub breakpoint: Breakpoint,
    pub kind: ResponsiveEntryKind,
    pub span: Span,
}

/// A `breakpoint: value` entry inside a `responsive { }` block. A responsive
/// block only overrides how an element is styled, so there is no variant that
/// swaps one element for another.
#[derive(Debug, Clone, PartialEq)]
pub enum ResponsiveEntryKind {
    /// A style property override, lowered to a plain CSS value.
    Property { name: Identifier, value: Expression },
    /// The name of a top-level `style { }` block to apply at this breakpoint.
    Named { name: Identifier },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResponsiveBlock {
    pub entries: Vec<ResponsiveEntry>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeDecl {
    pub name: Identifier,
    pub tokens: Vec<ThemeToken>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThemeToken {
    pub path: Vec<Identifier>,
    pub value: Expression,
    pub span: Span,
}

impl ThemeToken {
    pub fn qualified(&self) -> String {
        self.path
            .iter()
            .map(|segment| segment.as_str())
            .collect::<Vec<_>>()
            .join(".")
    }
}
