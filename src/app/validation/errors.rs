use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub rule: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(rule: &str, message: &str) -> Self {
        Self {
            rule: rule.to_string(),
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrors {
    pub message: String,
    pub errors: HashMap<String, HashMap<String, String>>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self {
            message: String::new(),
            errors: HashMap::new(),
        }
    }

    pub fn add(&mut self, field: &str, rule: &str, message: &str) {
        self.errors
            .entry(field.to_string())
            .or_insert_with(HashMap::new)
            .insert(rule.to_string(), message.to_string());
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn finalize(&mut self) {
        if self.has_errors() {
            let error_count = self.errors.values().map(|e| e.len()).sum::<usize>();
            let first_error = self.errors.values()
                .next()
                .and_then(|errors| errors.values().next())
                .map(|s| s.as_str())
                .unwrap_or("Validation failed");

            if error_count > 1 {
                self.message = format!("{}. (and {} more errors)", first_error, error_count - 1);
            } else {
                self.message = first_error.to_string();
            }
        }
    }

    pub fn first(&self) -> Option<String> {
        self.errors.values()
            .next()
            .and_then(|errors| errors.values().next())
            .cloned()
    }
}

impl Default for ValidationErrors {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValidationErrors {}