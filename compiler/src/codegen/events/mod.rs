use crate::ir::IrHandler;

/// The lifecycle events a `on <name> { }` block may declare. The runtime
/// calls these at fixed points in an element's or component's life, so they are
/// not DOM events and are never registered as listeners.
pub const LIFECYCLE: &[&str] = &["create", "mount", "update", "destroy"];

/// Whether a handler is a lifecycle handler rather than an event listener.
pub fn is_lifecycle(name: &str) -> bool {
    LIFECYCLE.contains(&name)
}

/// Maps a Platipus event name onto a DOM event name. A lifecycle event is
/// absent on purpose: it has no DOM counterpart.
pub fn dom_event(event: &str) -> Option<&'static str> {
    match event {
        "click" => Some("click"),
        "doubleclick" => Some("dblclick"),
        "mousedown" => Some("mousedown"),
        "mouseup" => Some("mouseup"),
        "mousemove" => Some("mousemove"),
        "mouseover" => Some("mouseover"),
        "mouseout" => Some("mouseout"),
        "contextmenu" => Some("contextmenu"),
        "wheel" => Some("wheel"),
        "keydown" => Some("keydown"),
        "keyup" => Some("keyup"),
        "keypress" => Some("keypress"),
        "input" => Some("input"),
        "change" => Some("change"),
        "submit" => Some("submit"),
        "reset" => Some("reset"),
        "focus" => Some("focusin"),
        "blur" => Some("focusout"),
        "focusin" => Some("focusin"),
        "focusout" => Some("focusout"),
        "select" => Some("select"),
        "invalid" => Some("invalid"),
        "drag" => Some("drag"),
        "dragstart" => Some("dragstart"),
        "dragend" => Some("dragend"),
        "dragover" => Some("dragover"),
        "drop" => Some("drop"),
        "scroll" => Some("scroll"),
        _ => None,
    }
}

/// Whether a DOM event must be bound on the capture phase so an ancestor can
/// still observe it. The pointer, keyboard, input, focus, and drag events
/// bubble on their own; the rest of the DOM events (the form events and
/// `scroll`) do not always, so they are captured instead. A lifecycle or
/// custom event has no DOM listener, so it is never captured.
pub fn needs_capture(event: &str) -> bool {
    dom_event(event).is_some()
        && crate::semantic::events::category_of(event)
            .is_some_and(|category| !category.is_bubbling())
}

/// Returns the emitter channel a custom event is dispatched through.
pub fn custom_channel(event: &str) -> String {
    format!("plt:{}", event)
}

pub fn is_custom(handler: &IrHandler) -> bool {
    handler.is_custom
}

/// The channel a handler is dispatched through. A lifecycle handler has no
/// channel: the runtime invokes it directly.
pub fn dispatch_name(handler: &IrHandler) -> String {
    match dom_event(&handler.event) {
        Some(name) if !handler.is_custom => name.to_string(),
        _ => custom_channel(&handler.event),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lifecycle_event_has_no_dom_counterpart() {
        for name in LIFECYCLE {
            assert!(is_lifecycle(name));
            assert_eq!(dom_event(name), None, "`{name}` is not a DOM event");
        }
    }

    #[test]
    fn a_dom_event_is_not_a_lifecycle_event() {
        assert!(!is_lifecycle("click"));
        assert_eq!(dom_event("click"), Some("click"));
        assert_eq!(dom_event("nope"), None);
    }

    #[test]
    fn an_interaction_event_is_bound_in_the_bubbling_phase() {
        for name in [
            "click",
            "doubleclick",
            "wheel",
            "keydown",
            "keypress",
            "input",
            "change",
            "focus",
            "blur",
            "focusin",
            "focusout",
            "dragover",
            "drop",
        ] {
            assert!(!needs_capture(name), "`{name}` bubbles on its own");
        }
    }

    #[test]
    fn a_non_bubbling_event_is_captured_so_an_ancestor_can_observe_it() {
        for name in ["submit", "reset", "invalid", "select", "scroll"] {
            assert!(needs_capture(name), "`{name}` does not bubble on its own");
        }
        assert!(!needs_capture("nope"), "unknown events have no listener");
    }
}
