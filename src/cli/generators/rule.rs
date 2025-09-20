use anyhow::Result;
use std::fs;
use std::path::Path;

pub async fn generate_rule(name: &str) -> Result<()> {
    let rule_name = if name.ends_with("Rule") {
        name.to_string()
    } else {
        format!("{}Rule", name)
    };

    let dir_path = "src/app/rules";
    fs::create_dir_all(dir_path)?;

    let file_path = format!("{}/{}.rs", dir_path, to_snake_case(&rule_name));

    let content = generate_rule_template(&rule_name);

    fs::write(&file_path, content)?;

    update_rules_mod(&rule_name)?;

    println!("Rule created successfully: {}", file_path);
    Ok(())
}

fn generate_rule_template(rule_name: &str) -> String {
    format!(r#"use anyhow::Result;
use serde::{{Deserialize, Serialize}};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct {} {{
    // Add rule-specific configuration here
    pub message: String,
    pub parameters: HashMap<String, String>,
}}

impl {} {{
    pub fn new() -> Self {{
        Self {{
            message: "The field is invalid.".to_string(),
            parameters: HashMap::new(),
        }}
    }}

    pub fn message(mut self, message: impl Into<String>) -> Self {{
        self.message = message.into();
        self
    }}

    pub fn parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {{
        self.parameters.insert(key.into(), value.into());
        self
    }}

    // Main validation method
    pub fn passes(&self, attribute: &str, value: &ValidationValue) -> bool {{
        match self.validate(attribute, value) {{
            Ok(valid) => valid,
            Err(_) => false,
        }}
    }}

    // Validation logic - implement your custom rule here
    fn validate(&self, _attribute: &str, value: &ValidationValue) -> Result<bool> {{
        // Implement your validation logic here
        // Example validation rules:

        match value {{
            ValidationValue::String(s) => {{
                // Example: Validate that string contains only uppercase letters
                Ok(s.chars().all(|c| c.is_uppercase() || c.is_whitespace()))
            }},
            ValidationValue::Number(n) => {{
                // Example: Validate that number is positive
                Ok(*n > 0.0)
            }},
            ValidationValue::Integer(i) => {{
                // Example: Validate that integer is even
                Ok(*i % 2 == 0)
            }},
            ValidationValue::Boolean(b) => {{
                // Example: Validate that boolean is true
                Ok(*b)
            }},
            ValidationValue::Array(arr) => {{
                // Example: Validate that array is not empty
                Ok(!arr.is_empty())
            }},
            ValidationValue::Null => {{
                // Example: Null values are invalid
                Ok(false)
            }},
        }}
    }}

    // Get the validation error message
    pub fn get_message(&self, attribute: &str, value: &ValidationValue) -> String {{
        // Replace placeholders in the message
        self.message
            .replace(":attribute", attribute)
            .replace(":value", &format!("{{:?}}", value))
    }}
}}

impl Default for {} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

// Enum to represent different value types for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationValue {{
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Array(Vec<ValidationValue>),
    Null,
}}

impl From<String> for ValidationValue {{
    fn from(value: String) -> Self {{
        ValidationValue::String(value)
    }}
}}

impl From<&str> for ValidationValue {{
    fn from(value: &str) -> Self {{
        ValidationValue::String(value.to_string())
    }}
}}

impl From<f64> for ValidationValue {{
    fn from(value: f64) -> Self {{
        ValidationValue::Number(value)
    }}
}}

impl From<i64> for ValidationValue {{
    fn from(value: i64) -> Self {{
        ValidationValue::Integer(value)
    }}
}}

impl From<bool> for ValidationValue {{
    fn from(value: bool) -> Self {{
        ValidationValue::Boolean(value)
    }}
}}

impl From<Vec<ValidationValue>> for ValidationValue {{
    fn from(value: Vec<ValidationValue>) -> Self {{
        ValidationValue::Array(value)
    }}
}}

impl From<Vec<String>> for ValidationValue {{
    fn from(value: Vec<String>) -> Self {{
        ValidationValue::Array(value.into_iter().map(ValidationValue::String).collect())
    }}
}}

impl From<Vec<&str>> for ValidationValue {{
    fn from(value: Vec<&str>) -> Self {{
        ValidationValue::Array(value.into_iter().map(|s| ValidationValue::String(s.to_string())).collect())
    }}
}}

impl From<Vec<f64>> for ValidationValue {{
    fn from(value: Vec<f64>) -> Self {{
        ValidationValue::Array(value.into_iter().map(ValidationValue::Number).collect())
    }}
}}

impl From<Vec<i64>> for ValidationValue {{
    fn from(value: Vec<i64>) -> Self {{
        ValidationValue::Array(value.into_iter().map(ValidationValue::Integer).collect())
    }}
}}

impl From<Vec<bool>> for ValidationValue {{
    fn from(value: Vec<bool>) -> Self {{
        ValidationValue::Array(value.into_iter().map(ValidationValue::Boolean).collect())
    }}
}}

// Validation rule trait for type-safe validation
pub trait ValidationRule {{
    fn passes(&self, attribute: &str, value: &ValidationValue) -> bool;
    fn get_message(&self, attribute: &str, value: &ValidationValue) -> String;
}}

impl ValidationRule for {} {{
    fn passes(&self, attribute: &str, value: &ValidationValue) -> bool {{
        Self::passes(self, attribute, value)
    }}

    fn get_message(&self, attribute: &str, value: &ValidationValue) -> String {{
        Self::get_message(self, attribute, value)
    }}
}}

// Validator struct for applying multiple rules
#[derive(Debug, Clone)]
pub struct Validator {{
    pub rules: Vec<Box<dyn ValidationRule>>,
    pub errors: HashMap<String, Vec<String>>,
}}

impl Validator {{
    pub fn new() -> Self {{
        Self {{
            rules: Vec::new(),
            errors: HashMap::new(),
        }}
    }}

    pub fn add_rule(mut self, rule: Box<dyn ValidationRule>) -> Self {{
        self.rules.push(rule);
        self
    }}

    pub fn validate(&mut self, data: &HashMap<String, ValidationValue>) -> bool {{
        self.errors.clear();
        let mut is_valid = true;

        for (attribute, value) in data {{
            for rule in &self.rules {{
                if !rule.passes(attribute, value) {{
                    let message = rule.get_message(attribute, value);
                    self.errors
                        .entry(attribute.clone())
                        .or_insert_with(Vec::new)
                        .push(message);
                    is_valid = false;
                }}
            }}
        }}

        is_valid
    }}

    pub fn get_errors(&self) -> &HashMap<String, Vec<String>> {{
        &self.errors
    }}
}}

impl Default for Validator {{
    fn default() -> Self {{
        Self::new()
    }}
}}

// Example usage and testing
#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_rule_validation() {{
        let rule = {}::new()
            .message("The :attribute field must be valid.");

        // Test with valid value
        let valid_value = ValidationValue::String("HELLO WORLD".to_string());
        assert!(rule.passes("name", &valid_value));

        // Test with invalid value
        let invalid_value = ValidationValue::String("hello world".to_string());
        assert!(!rule.passes("name", &invalid_value));

        // Test error message
        let message = rule.get_message("name", &invalid_value);
        assert!(message.contains("name"));
    }}

    #[test]
    fn test_validator() {{
        let mut validator = Validator::new()
            .add_rule(Box::new({}::new()));

        let mut data = HashMap::new();
        data.insert("name".to_string(), ValidationValue::String("VALID".to_string()));
        data.insert("count".to_string(), ValidationValue::Integer(10));

        let is_valid = validator.validate(&data);
        assert!(is_valid);
        assert!(validator.get_errors().is_empty());
    }}
}}
"#, rule_name, rule_name, rule_name, rule_name, rule_name, rule_name)
}

fn update_rules_mod(rule_name: &str) -> Result<()> {
    let mod_path = "src/app/rules/mod.rs";
    let module_name = to_snake_case(rule_name);

    if !Path::new("src/app/rules").exists() {
        fs::create_dir_all("src/app/rules")?;
    }

    let mod_content = if Path::new(mod_path).exists() {
        let existing = fs::read_to_string(mod_path)?;
        if existing.contains(&format!("pub mod {};", module_name)) {
            return Ok(());
        }
        format!("{}\npub mod {};", existing.trim(), module_name)
    } else {
        format!("pub mod {};", module_name)
    };

    fs::write(mod_path, mod_content)?;
    Ok(())
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_is_upper = true;
        } else {
            result.push(c);
            prev_is_upper = false;
        }
    }

    result
}