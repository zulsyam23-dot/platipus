use crate::ast::StateKind as SemanticStateKind;
use crate::diagnostics::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct IrExpression {
    pub value: String,
    pub dependencies: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrProperty {
    pub name: String,
    pub value: IrExpression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrStyleEntry {
    pub name: String,
    pub value: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IrStyleBlock {
    /// Present for a top-level `style Name { }` definition, absent for an
    /// inline `style { }` block and for nested style states.
    pub name: Option<String>,
    pub entries: Vec<IrStyleEntry>,
    pub states: Vec<(IrStyleState, IrStyleBlock)>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IrStyleState {
    Normal,
    Hover,
    Pressed,
    Focused,
    Disabled,
    Selected,
}

impl IrStyleState {
    pub const fn as_str(self) -> &'static str {
        match self {
            IrStyleState::Normal => "normal",
            IrStyleState::Hover => "hover",
            IrStyleState::Pressed => "pressed",
            IrStyleState::Focused => "focused",
            IrStyleState::Disabled => "disabled",
            IrStyleState::Selected => "selected",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "normal" => Some(IrStyleState::Normal),
            "hover" => Some(IrStyleState::Hover),
            "pressed" => Some(IrStyleState::Pressed),
            "focused" => Some(IrStyleState::Focused),
            "disabled" => Some(IrStyleState::Disabled),
            "selected" => Some(IrStyleState::Selected),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IrEventCategory(pub crate::ast::EventCategory);

impl IrEventCategory {
    pub const fn as_str(self) -> &'static str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IrStateKind {
    Local,
    Shared,
    Global,
    Persistent,
    Derived,
}

impl From<SemanticStateKind> for IrStateKind {
    fn from(kind: SemanticStateKind) -> Self {
        match kind {
            SemanticStateKind::Local => IrStateKind::Local,
            SemanticStateKind::Shared => IrStateKind::Shared,
            SemanticStateKind::Global => IrStateKind::Global,
            SemanticStateKind::Persistent => IrStateKind::Persistent,
            SemanticStateKind::Derived => IrStateKind::Derived,
        }
    }
}

impl IrStateKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            IrStateKind::Local => "local",
            IrStateKind::Shared => "shared",
            IrStateKind::Global => "global",
            IrStateKind::Persistent => "persistent",
            IrStateKind::Derived => "derived",
        }
    }

    pub const fn is_reactive(self) -> bool {
        matches!(self, IrStateKind::Local | IrStateKind::Shared)
    }
}
