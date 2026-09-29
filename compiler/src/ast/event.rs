use super::expression::{Expression, Identifier};
use super::statement::Block;
use crate::diagnostics::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct EventHandlerDecl {
    pub name: Identifier,
    pub body: Block,
    pub span: Span,
}

impl EventHandlerDecl {
    pub fn new(name: Identifier, body: Block, span: Span) -> Self {
        Self { name, body, span }
    }

    pub fn is_lifecycle(&self) -> bool {
        lifecycle::LIFECYCLE_EVENTS.contains(&self.name.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmitStatement {
    pub name: Identifier,
    pub arguments: Vec<Expression>,
    pub span: Span,
}

impl EmitStatement {
    pub fn payload(&self) -> Option<&Expression> {
        self.arguments.first()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventCategory {
    Pointer,
    Keyboard,
    Input,
    Focus,
    Form,
    Drag,
    Clipboard,
    Media,
    Lifecycle,
    Animation,
    Application,
    Custom,
}

impl EventCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            EventCategory::Pointer => "pointer",
            EventCategory::Keyboard => "keyboard",
            EventCategory::Input => "input",
            EventCategory::Focus => "focus",
            EventCategory::Form => "form",
            EventCategory::Drag => "drag",
            EventCategory::Clipboard => "clipboard",
            EventCategory::Media => "media",
            EventCategory::Lifecycle => "lifecycle",
            EventCategory::Animation => "animation",
            EventCategory::Application => "application",
            EventCategory::Custom => "custom",
        }
    }

    /// Whether the events of this category reach an ancestor when a child is
    /// the origin. The pointer, keyboard, input, focus, and drag events bubble
    /// on their own. The remaining DOM events are bound on the capture phase
    /// instead, so an ancestor can still observe them (see `needs_capture`).
    pub const fn is_bubbling(self) -> bool {
        matches!(
            self,
            EventCategory::Pointer
                | EventCategory::Keyboard
                | EventCategory::Input
                | EventCategory::Focus
                | EventCategory::Drag
        )
    }
}

pub mod lifecycle {
    pub const LIFECYCLE_EVENTS: &[&str] = &["create", "mount", "update", "destroy"];

    pub fn is_lifecycle(name: &str) -> bool {
        LIFECYCLE_EVENTS.contains(&name)
    }
}

pub mod known {
    pub const ALL: &[&str] = &[
        "click",
        "doubleclick",
        "mousedown",
        "mouseup",
        "mousemove",
        "mouseover",
        "mouseout",
        "contextmenu",
        "wheel",
        "keydown",
        "keyup",
        "keypress",
        "input",
        "change",
        "submit",
        "reset",
        "focus",
        "blur",
        "focusin",
        "focusout",
        "select",
        "invalid",
        "drag",
        "dragstart",
        "dragend",
        "dragover",
        "drop",
        "scroll",
        "swipe",
        "create",
        "mount",
        "update",
        "destroy",
    ];

    pub fn is_known(name: &str) -> bool {
        ALL.contains(&name)
    }
}
