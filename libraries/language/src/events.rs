pub use crate::ast::EventCategory;
pub use crate::ast::known::ALL;

pub fn is_known(name: &str) -> bool {
    crate::ast::known::is_known(name)
}

pub const POINTER_EVENTS: &[&str] = &[
    "click",
    "doubleclick",
    "mousedown",
    "mouseup",
    "mousemove",
    "mouseover",
    "mouseout",
    "contextmenu",
    "wheel",
];

pub const KEYBOARD_EVENTS: &[&str] = &[
    "keydown", "keyup", "keypress", "input", "change", "submit", "reset", "focus", "blur",
    "focusin", "focusout", "select", "invalid",
];

pub const DRAG_EVENTS: &[&str] = &["drag", "dragstart", "dragend", "dragover", "drop", "scroll"];

pub const LIFECYCLE_EVENTS: &[&str] = &["create", "mount", "update", "destroy"];


pub fn category_of(name: &str) -> Option<EventCategory> {
    if POINTER_EVENTS.contains(&name) {
        Some(EventCategory::Pointer)
    } else if name == "input" || name == "change" {
        Some(EventCategory::Input)
    } else if name == "submit" || name == "reset" || name == "invalid" {
        Some(EventCategory::Form)
    } else if name.starts_with("key") {
        Some(EventCategory::Keyboard)
    } else if name.starts_with("drag") || name == "drop" {
        Some(EventCategory::Drag)
    } else if name == "focus" || name == "blur" || name == "focusin" || name == "focusout" {
        Some(EventCategory::Focus)
    } else if LIFECYCLE_EVENTS.contains(&name) {
        Some(EventCategory::Lifecycle)
    } else if name == "scroll" {
        Some(EventCategory::Custom)
    } else if name == "select" {
        Some(EventCategory::Form)
    } else {
        None
    }
}
