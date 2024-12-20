use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::RwLock};

static CACHE: Lazy<RwLock<HashMap<u16, String>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub fn insert(key: u16, tag: String) {
    CACHE.write().unwrap().insert(key, tag);
}

pub fn get(key: u16) -> String {
    CACHE
        .read()
        .unwrap()
        .get(&key)
        .unwrap_or(&String::new())
        .clone()
}

pub fn clear() {
    CACHE.write().unwrap().clear();
}

pub fn contain_key(key: u16) -> bool {
    CACHE.read().unwrap().contains_key(&key)
}
