use std::collections::BTreeMap;
use std::rc::Rc;

use crate::state::Value;

/// Identifies one registered listener so it can be removed again.
pub type ListenerId = u64;

type Handler = Rc<dyn Fn(&Value)>;

#[derive(Default)]
struct Channel {
    listeners: Vec<(ListenerId, Handler)>,
    next_token: ListenerId,
}

/// Routes events between the components of a program.
///
/// DOM events arrive through the generated bindings, custom events are emitted
/// by a component and delivered to whoever subscribed to that name.
#[derive(Default)]
pub struct EventBus {
    channels: BTreeMap<String, Channel>,
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("channels", &self.channels.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn on(
        &mut self,
        event: &str,
        handler: impl Fn(&Value) + 'static,
    ) -> ListenerId {
        let channel = self.channels.entry(event.to_string()).or_default();
        channel.next_token += 1;
        let token = channel.next_token;
        channel
            .listeners
            .push((token, Rc::new(handler) as Handler));
        token
    }

    pub fn off(&mut self, event: &str, token: ListenerId) -> bool {
        let Some(channel) = self.channels.get_mut(event) else {
            return false;
        };
        let before = channel.listeners.len();
        channel.listeners.retain(|(id, _)| *id != token);
        channel.listeners.len() != before
    }

    /// Delivers a payload to every listener of an event, in registration order.
    pub fn emit(&self, event: &str, payload: &Value) -> usize {
        let Some(channel) = self.channels.get(event) else {
            return 0;
        };
        let handlers: Vec<Handler> = channel
            .listeners
            .iter()
            .map(|(_, handler)| Rc::clone(handler))
            .collect();
        for handler in handlers {
            handler(payload);
        }
        channel.listeners.len()
    }

    pub fn has_listeners(&self, event: &str) -> bool {
        self.channels
            .get(event)
            .is_some_and(|channel| !channel.listeners.is_empty())
    }

    pub fn events(&self) -> Vec<&str> {
        self.channels.keys().map(String::as_str).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn delivers_payloads_to_listeners() {
        let mut bus = EventBus::new();
        let seen = Rc::new(RefCell::new(Vec::new()));
        let observer = Rc::clone(&seen);
        bus.on("picked", move |payload| observer.borrow_mut().push(payload.clone()));
        bus.emit("picked", &Value::int(1));
        assert_eq!(*seen.borrow(), vec![Value::int(1)]);
    }

    #[test]
    fn several_listeners_receive_the_event() {
        let mut bus = EventBus::new();
        let total = Rc::new(RefCell::new(0));
        for _ in 0..3 {
            let counter = Rc::clone(&total);
            bus.on("tick", move |_| *counter.borrow_mut() += 1);
        }
        assert_eq!(bus.emit("tick", &Value::Null), 3);
        assert_eq!(*total.borrow(), 3);
    }

    #[test]
    fn removing_a_listener_stops_delivery() {
        let mut bus = EventBus::new();
        let seen = Rc::new(RefCell::new(0));
        let observer = Rc::clone(&seen);
        let token = bus.on("tick", move |_| *observer.borrow_mut() += 1);
        bus.emit("tick", &Value::Null);
        assert!(bus.off("tick", token));
        bus.emit("tick", &Value::Null);
        assert_eq!(*seen.borrow(), 1);
    }

    #[test]
    fn emitting_an_unused_event_is_a_no_op() {
        let bus = EventBus::new();
        assert_eq!(bus.emit("nobody", &Value::Null), 0);
        assert!(!bus.has_listeners("nobody"));
    }

    #[test]
    fn channels_are_listed_in_order() {
        let mut bus = EventBus::new();
        bus.on("b", |_| {});
        bus.on("a", |_| {});
        assert_eq!(bus.events(), vec!["a", "b"]);
    }
}
