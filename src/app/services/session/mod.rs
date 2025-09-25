use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

pub mod drivers;
pub mod session_manager;
pub mod session_store;

pub use session_manager::SessionManager;
pub use session_store::SessionStore;

#[async_trait]
pub trait SessionHandler: Send + Sync {
    async fn read(&self, session_id: &str) -> Result<Option<String>>;
    async fn write(&self, session_id: &str, data: &str) -> Result<()>;
    async fn destroy(&self, session_id: &str) -> Result<()>;
    async fn gc(&self, lifetime: u64) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub data: HashMap<String, Value>,
    pub is_dirty: bool,
    pub is_started: bool,
}

impl Session {
    pub fn new(id: String) -> Self {
        Self {
            id,
            data: HashMap::new(),
            is_dirty: false,
            is_started: false,
        }
    }

    pub fn put(&mut self, key: &str, value: Value) {
        self.data.insert(key.to_string(), value);
        self.is_dirty = true;
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.get(key)?.as_str().map(|s| s.to_string())
    }

    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.get(key)?.as_i64()
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get(key)?.as_bool()
    }

    pub fn has(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn forget(&mut self, key: &str) -> Option<Value> {
        let value = self.data.remove(key);
        if value.is_some() {
            self.is_dirty = true;
        }
        value
    }

    pub fn flush(&mut self) {
        self.data.clear();
        self.is_dirty = true;
    }

    pub fn flash(&mut self, key: &str, value: Value) {
        self.put(&format!("_flash.new.{}", key), value);
    }

    pub fn now(&mut self, key: &str, value: Value) {
        self.put(&format!("_flash.old.{}", key), value);
    }

    pub fn reflash(&mut self) {
        let flash_data: HashMap<String, Value> = self.data
            .iter()
            .filter(|(k, _)| k.starts_with("_flash.old."))
            .map(|(k, v)| (k.replace("_flash.old.", "_flash.new."), v.clone()))
            .collect();

        for (key, value) in flash_data {
            self.put(&key, value);
        }
    }

    pub fn keep(&mut self, keys: Vec<&str>) {
        for key in keys {
            if let Some(value) = self.get(&format!("_flash.old.{}", key)).cloned() {
                self.put(&format!("_flash.new.{}", key), value);
            }
        }
    }

    pub fn all(&self) -> &HashMap<String, Value> {
        &self.data
    }

    pub fn only(&self, keys: Vec<&str>) -> HashMap<String, Value> {
        keys.into_iter()
            .filter_map(|key| self.get(key).map(|v| (key.to_string(), v.clone())))
            .collect()
    }

    pub fn except(&self, keys: Vec<&str>) -> HashMap<String, Value> {
        self.data
            .iter()
            .filter(|(key, _)| !keys.contains(&key.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn token(&mut self) -> String {
        if let Some(token) = self.get_string("_token") {
            token
        } else {
            let token = generate_token();
            self.put("_token", Value::String(token.clone()));
            token
        }
    }

    pub fn regenerate_token(&mut self) -> String {
        let token = generate_token();
        self.put("_token", Value::String(token.clone()));
        token
    }

    pub fn age_flash_data(&mut self) {
        let old_keys: Vec<String> = self.data
            .keys()
            .filter(|k| k.starts_with("_flash.old."))
            .cloned()
            .collect();

        let old_keys_count = old_keys.len();
        for key in &old_keys {
            self.data.remove(key);
        }

        let new_keys: Vec<String> = self.data
            .keys()
            .filter(|k| k.starts_with("_flash.new."))
            .cloned()
            .collect();

        let new_keys_count = new_keys.len();
        for key in &new_keys {
            if let Some(value) = self.data.remove(key) {
                let old_key = key.replace("_flash.new.", "_flash.old.");
                self.data.insert(old_key, value);
            }
        }

        if old_keys_count > 0 || new_keys_count > 0 {
            self.is_dirty = true;
        }
    }
}

fn generate_token() -> String {
    use rand::{distributions::Alphanumeric, Rng};
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(40)
        .map(char::from)
        .collect()
}