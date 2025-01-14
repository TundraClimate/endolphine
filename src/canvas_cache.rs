use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::RwLock};

static CACHE: Lazy<RwLock<HashMap<(u16, u8), String>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub fn insert(key: (u16, u8), tag: String) {
    CACHE.write().unwrap().insert(key, tag);
}

pub fn cache_match(key: (u16, u8), tag: &str) -> bool {
    CACHE.read().unwrap().get(&key).map(|c| c.as_ref()) == Some(tag)
}

pub fn clear() {
    CACHE.write().unwrap().clear();
}
