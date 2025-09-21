use crate::app::validation::{ValidationRules, ValidationData, ValidationErrors};
use crate::app::validation::rules::*;
use serde_json::Value;
use std::collections::HashMap;
use sqlx::PgPool;

pub struct Validator {
    data: ValidationData,
    rules: ValidationRules,
    db: Option<PgPool>,
}

impl Validator {
    pub fn new(data: ValidationData, rules: ValidationRules) -> Self {
        Self {
            data,
            rules,
            db: None,
        }
    }

    pub fn with_db(mut self, db: PgPool) -> Self {
        self.db = Some(db);
        self
    }

    pub async fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        for (field, field_rules) in &self.rules {
            self.validate_field(field, field_rules, &mut errors).await;
        }

        if errors.has_errors() {
            errors.finalize();
            Err(errors)
        } else {
            Ok(())
        }
    }

    async fn validate_field(&self, field: &str, field_rules: &[String], errors: &mut ValidationErrors) {
        let value = self.get_nested_value(field);

        for rule_str in field_rules {
            if let Err(validation_error) = self.validate_rule(field, rule_str, &value).await {
                errors.add(field, &validation_error.rule, &validation_error.message);
            }
        }

        // Handle array validation for nested fields
        if let Value::Array(arr) = &value {
            for (index, item) in arr.iter().enumerate() {
                let indexed_field = format!("{}[{}]", field, index);
                if item.is_object() {
                    self.validate_nested_object(&indexed_field, item, field_rules, errors).await;
                }
            }
        }
    }

    async fn validate_nested_object(&self, base_field: &str, obj: &Value, _rules: &[String], errors: &mut ValidationErrors) {
        // Add recursion depth limit to prevent stack overflow
        const MAX_DEPTH: usize = 10;
        let depth = base_field.matches('.').count() + base_field.matches('[').count();

        if depth >= MAX_DEPTH {
            return;
        }

        if let Value::Object(map) = obj {
            for (key, value) in map {
                let nested_field = format!("{}.{}", base_field, key);

                // Check if we have rules for this nested field
                if let Some(nested_rules) = self.rules.get(&format!("*.{}", key)) {
                    if let Err(validation_error) = self.validate_nested_rule(&nested_field, nested_rules, value).await {
                        errors.add(&nested_field, &validation_error.rule, &validation_error.message);
                    }
                }

                // Check for deeper nesting
                if value.is_object() || value.is_array() {
                    self.validate_deep_nested(&nested_field, value, errors).await;
                }
            }
        }
    }

    fn validate_deep_nested<'a>(&'a self, base_field: &'a str, value: &'a Value, errors: &'a mut ValidationErrors) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            // Add recursion depth limit to prevent stack overflow
            const MAX_DEPTH: usize = 10;
            let depth = base_field.matches('.').count() + base_field.matches('[').count();

            if depth >= MAX_DEPTH {
                return;
            }

            match value {
                Value::Object(map) => {
                    for (key, val) in map {
                        let nested_field = format!("{}.{}", base_field, key);

                        // Check for wildcard rules that match this path
                        for (rule_field, rules) in &self.rules {
                            if self.matches_wildcard_pattern(rule_field, &nested_field) {
                                if let Err(validation_error) = self.validate_nested_rule(&nested_field, rules, val).await {
                                    errors.add(&nested_field, &validation_error.rule, &validation_error.message);
                                }
                            }
                        }

                        if val.is_object() || val.is_array() {
                            self.validate_deep_nested(&nested_field, val, errors).await;
                        }
                    }
                },
                Value::Array(arr) => {
                    for (index, item) in arr.iter().enumerate() {
                        let indexed_field = format!("{}[{}]", base_field, index);

                        if item.is_object() || item.is_array() {
                            self.validate_deep_nested(&indexed_field, item, errors).await;
                        }
                    }
                },
                _ => {}
            }
        })
    }

    fn matches_wildcard_pattern(&self, pattern: &str, field: &str) -> bool {
        // Handle patterns like "authorization.role", "users.*.name", etc.
        if pattern == field {
            return true;
        }

        let pattern_parts: Vec<&str> = pattern.split('.').collect();
        let field_parts: Vec<&str> = field.split('.').collect();

        if pattern_parts.len() != field_parts.len() {
            return false;
        }

        for (pattern_part, field_part) in pattern_parts.iter().zip(field_parts.iter()) {
            if *pattern_part != "*" && *pattern_part != *field_part {
                // Check for array index patterns like [0], [1], etc.
                if field_part.contains('[') && field_part.contains(']') {
                    let base_field = field_part.split('[').next().unwrap_or("");
                    if *pattern_part != "*" && *pattern_part != base_field {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }

        true
    }

    async fn validate_nested_rule(&self, field: &str, rules: &[String], value: &Value) -> Result<(), crate::app::validation::errors::ValidationError> {
        for rule_str in rules {
            self.validate_rule(field, rule_str, value).await?;
        }
        Ok(())
    }

    async fn validate_rule(&self, field: &str, rule_str: &str, value: &Value) -> Result<(), crate::app::validation::errors::ValidationError> {
        let rule: Box<dyn Rule> = self.parse_rule(rule_str)?;
        rule.validate(field, value, &self.data, self.db.as_ref()).await
    }

    fn parse_rule(&self, rule_str: &str) -> Result<Box<dyn Rule>, crate::app::validation::errors::ValidationError> {
        // Rules with parameters
        if rule_str.starts_with("min:") {
            let min_value = rule_str[4..].parse::<usize>()
                .map_err(|_| crate::app::validation::errors::ValidationError::new("parse_error", "Invalid min value"))?;
            Ok(Box::new(MinRule::new(min_value)))
        } else if rule_str.starts_with("max:") {
            let max_value = rule_str[4..].parse::<usize>()
                .map_err(|_| crate::app::validation::errors::ValidationError::new("parse_error", "Invalid max value"))?;
            Ok(Box::new(MaxRule::new(max_value)))
        } else if rule_str.starts_with("between:") {
            let range = &rule_str[8..];
            let parts: Vec<&str> = range.split(',').collect();
            if parts.len() == 2 {
                let min = parts[0].parse::<f64>()
                    .map_err(|_| crate::app::validation::errors::ValidationError::new("parse_error", "Invalid between min value"))?;
                let max = parts[1].parse::<f64>()
                    .map_err(|_| crate::app::validation::errors::ValidationError::new("parse_error", "Invalid between max value"))?;
                Ok(Box::new(BetweenRule::new(min, max)))
            } else {
                Err(crate::app::validation::errors::ValidationError::new("parse_error", "Between rule requires min,max format"))
            }
        } else if rule_str.starts_with("size:") {
            let size_value = rule_str[5..].parse::<usize>()
                .map_err(|_| crate::app::validation::errors::ValidationError::new("parse_error", "Invalid size value"))?;
            Ok(Box::new(SizeRule::new(size_value)))
        } else if rule_str.starts_with("digits:") {
            let digits_value = rule_str[7..].parse::<usize>()
                .map_err(|_| crate::app::validation::errors::ValidationError::new("parse_error", "Invalid digits value"))?;
            Ok(Box::new(DigitsRule::new(digits_value)))
        } else if rule_str.starts_with("digits_between:") {
            let range = &rule_str[15..];
            let parts: Vec<&str> = range.split(',').collect();
            if parts.len() == 2 {
                let min = parts[0].parse::<usize>()
                    .map_err(|_| crate::app::validation::errors::ValidationError::new("parse_error", "Invalid digits_between min value"))?;
                let max = parts[1].parse::<usize>()
                    .map_err(|_| crate::app::validation::errors::ValidationError::new("parse_error", "Invalid digits_between max value"))?;
                Ok(Box::new(DigitsBetweenRule::new(min, max)))
            } else {
                Err(crate::app::validation::errors::ValidationError::new("parse_error", "Digits between rule requires min,max format"))
            }
        } else if rule_str.starts_with("unique:") {
            let table_info = &rule_str[7..];
            if table_info.contains(',') {
                let parts: Vec<&str> = table_info.split(',').collect();
                Ok(Box::new(UniqueRule::with_column(parts[0], parts[1])))
            } else {
                Ok(Box::new(UniqueRule::new(table_info)))
            }
        } else if rule_str.starts_with("exists:") {
            let table_info = &rule_str[7..];
            if table_info.contains(',') {
                let parts: Vec<&str> = table_info.split(',').collect();
                Ok(Box::new(ExistsRule::with_column(parts[0], parts[1])))
            } else {
                Ok(Box::new(ExistsRule::new(table_info)))
            }
        } else if rule_str.starts_with("in:") {
            let values_str = &rule_str[3..];
            let values: Vec<String> = values_str.split(',').map(|s| s.to_string()).collect();
            Ok(Box::new(InRule::new(values)))
        } else if rule_str.starts_with("not_in:") {
            let values_str = &rule_str[7..];
            let values: Vec<String> = values_str.split(',').map(|s| s.to_string()).collect();
            Ok(Box::new(NotInRule::new(values)))
        } else if rule_str.starts_with("mimes:") {
            let types_str = &rule_str[6..];
            let types: Vec<String> = types_str.split(',').map(|s| s.to_string()).collect();
            Ok(Box::new(MimesRule::new(types)))
        } else if rule_str.starts_with("before:") {
            let date = &rule_str[7..];
            Ok(Box::new(BeforeRule::new(date)))
        } else if rule_str.starts_with("after:") {
            let date = &rule_str[6..];
            Ok(Box::new(AfterRule::new(date)))
        } else if rule_str.starts_with("date_format:") {
            let format = &rule_str[12..];
            Ok(Box::new(DateFormatRule::new(format)))
        } else if rule_str.starts_with("required_if:") {
            let params = &rule_str[12..];
            if let Some(comma_pos) = params.find(',') {
                let field = &params[..comma_pos];
                let value = &params[comma_pos + 1..];
                Ok(Box::new(RequiredIfRule::new(field, value)))
            } else {
                Err(crate::app::validation::errors::ValidationError::new("parse_error", "Required if rule requires field,value format"))
            }
        } else if rule_str.starts_with("required_unless:") {
            let params = &rule_str[16..];
            if let Some(comma_pos) = params.find(',') {
                let field = &params[..comma_pos];
                let value = &params[comma_pos + 1..];
                Ok(Box::new(RequiredUnlessRule::new(field, value)))
            } else {
                Err(crate::app::validation::errors::ValidationError::new("parse_error", "Required unless rule requires field,value format"))
            }
        } else if rule_str.starts_with("same:") {
            let other_field = &rule_str[5..];
            Ok(Box::new(SameRule::new(other_field)))
        } else if rule_str.starts_with("different:") {
            let other_field = &rule_str[10..];
            Ok(Box::new(DifferentRule::new(other_field)))
        } else if rule_str.starts_with("regex:") {
            let pattern = &rule_str[6..];
            Ok(Box::new(RegexRule::new(pattern)))
        } else if rule_str.starts_with("starts_with:") {
            let prefix = &rule_str[12..];
            Ok(Box::new(StartsWithRule::new(prefix)))
        } else if rule_str.starts_with("ends_with:") {
            let suffix = &rule_str[10..];
            Ok(Box::new(EndsWithRule::new(suffix)))
        } else {
            // Simple rules without parameters
            match rule_str {
                "required" => Ok(Box::new(RequiredRule)),
                "string" => Ok(Box::new(StringRule)),
                "email" => Ok(Box::new(EmailRule)),
                "numeric" => Ok(Box::new(NumericRule)),
                "integer" => Ok(Box::new(IntegerRule)),
                "boolean" => Ok(Box::new(BooleanRule)),
                "array" => Ok(Box::new(ArrayRule)),
                "alpha" => Ok(Box::new(AlphaRule)),
                "alpha_dash" => Ok(Box::new(AlphaDashRule)),
                "alpha_num" => Ok(Box::new(AlphaNumRule)),
                "date" => Ok(Box::new(DateRule)),
                "url" => Ok(Box::new(UrlRule)),
                "uuid" => Ok(Box::new(UuidRule)),
                "ulid" => Ok(Box::new(UlidRule)),
                "json" => Ok(Box::new(JsonRule)),
                "ip" => Ok(Box::new(IpRule)),
                "file" => Ok(Box::new(FileRule)),
                "image" => Ok(Box::new(ImageRule)),
                "confirmed" => Ok(Box::new(ConfirmedRule)),
                _ => Err(crate::app::validation::errors::ValidationError::new("unknown_rule", &format!("Unknown validation rule: {}", rule_str))),
            }
        }
    }

    fn get_nested_value(&self, field: &str) -> Value {
        if field.contains('.') {
            self.get_nested_value_recursive(&self.data, field)
        } else {
            self.data.get(field).cloned().unwrap_or(Value::Null)
        }
    }

    fn get_nested_value_recursive(&self, data: &HashMap<String, Value>, field: &str) -> Value {
        let parts: Vec<&str> = field.split('.').collect();
        let mut current = Value::Object(data.clone().into_iter().collect());

        for part in parts {
            match current {
                Value::Object(ref map) => {
                    if let Some(value) = map.get(part) {
                        current = value.clone();
                    } else {
                        return Value::Null;
                    }
                },
                Value::Array(ref arr) => {
                    // Handle array index notation like users[0]
                    if part.contains('[') && part.contains(']') {
                        let bracket_start = part.find('[').unwrap();
                        let bracket_end = part.find(']').unwrap();
                        let _array_field = &part[..bracket_start];
                        let index_str = &part[bracket_start + 1..bracket_end];

                        if let Ok(index) = index_str.parse::<usize>() {
                            if let Some(value) = arr.get(index) {
                                current = value.clone();
                            } else {
                                return Value::Null;
                            }
                        } else {
                            return Value::Null;
                        }
                    } else {
                        return Value::Null;
                    }
                },
                _ => return Value::Null,
            }
        }

        current
    }
}

// Helper function to create validator from JSON-like data
pub fn make_validator(data: serde_json::Value, rules: ValidationRules) -> Validator {
    let validation_data: ValidationData = match data {
        Value::Object(map) => map.into_iter().collect(),
        _ => HashMap::new(),
    };

    Validator::new(validation_data, rules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_basic_validation() {
        let data = json!({
            "title": "Test Post",
            "body": "This is a test body"
        });

        let mut rules = HashMap::new();
        rules.insert("title".to_string(), vec!["required".to_string(), "string".to_string(), "max:255".to_string()]);
        rules.insert("body".to_string(), vec!["required".to_string()]);

        let validator = make_validator(data, rules);
        let result = validator.validate().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validation_errors() {
        let data = json!({
            "title": "",
            "body": null
        });

        let mut rules = HashMap::new();
        rules.insert("title".to_string(), vec!["required".to_string()]);
        rules.insert("body".to_string(), vec!["required".to_string()]);

        let validator = make_validator(data, rules);
        let result = validator.validate().await;

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.has_errors());
        assert!(errors.errors.contains_key("title"));
        assert!(errors.errors.contains_key("body"));
    }

    #[tokio::test]
    async fn test_nested_validation() {
        let data = json!({
            "user": {
                "name": "John",
                "email": "invalid-email"
            },
            "authorization": {
                "role": ""
            }
        });

        let mut rules = HashMap::new();
        rules.insert("user.name".to_string(), vec!["required".to_string(), "string".to_string()]);
        rules.insert("user.email".to_string(), vec!["required".to_string(), "email".to_string()]);
        rules.insert("authorization.role".to_string(), vec!["required".to_string()]);

        let validator = make_validator(data, rules);
        let result = validator.validate().await;

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.errors.contains_key("user.email"));
        assert!(errors.errors.contains_key("authorization.role"));
    }
}