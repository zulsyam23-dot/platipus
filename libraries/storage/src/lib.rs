//! Storage contract for Platipus.
//!
//! The language exposes `persistent` state and a `store()`/`load()`/`drop()`
//! vocabulary. This crate defines the abstraction those features lower to;
//! platform implementations (Web localStorage, desktop filesystem, native
//! key-value stores) live outside the domain layer and implement this trait.

use std::collections::HashMap;

/// A synchronous key-value store. Implementations may be in-memory, browser
/// storage, or a filesystem-backed store; callers only see the contract.
pub trait Storage {
    fn get(&self, key: &str) -> Option<&str>;
    fn set(&mut self, key: &str, value: String);
    fn remove(&mut self, key: &str);
    fn clear(&mut self);
}

/// The default, always-available implementation: process-local memory.
#[derive(Debug, Default, Clone)]
pub struct MemoryStorage {
    map: HashMap<String, String>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Storage for MemoryStorage {
    fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(String::as_str)
    }

    fn set(&mut self, key: &str, value: String) {
        self.map.insert(key.to_string(), value);
    }

    fn remove(&mut self, key: &str) {
        self.map.remove(key);
    }

    fn clear(&mut self) {
        self.map.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_storage_round_trips() {
        let mut storage = MemoryStorage::new();
        storage.set("k", "v".into());
        assert_eq!(storage.get("k"), Some("v"));
        storage.remove("k");
        assert_eq!(storage.get("k"), None);
    }
}
