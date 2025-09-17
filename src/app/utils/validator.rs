use std::collections::HashMap;
use anyhow::Result;
use regex::Regex;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub rule: String,
    pub message: String,
    pub value: Option<String>,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, rule: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            rule: rule.into(),
            message: message.into(),
            value: None,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct ValidationErrors {
    pub errors: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn get_messages(&self) -> HashMap<String, Vec<String>> {
        let mut messages = HashMap::new();
        for error in &self.errors {
            messages
                .entry(error.field.clone())
                .or_insert_with(Vec::new)
                .push(error.message.clone());
        }
        messages
    }

    pub fn first(&self, field: &str) -> Option<&String> {
        self.errors
            .iter()
            .find(|e| e.field == field)
            .map(|e| &e.message)
    }
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub parameters: Vec<String>,
}

impl Rule {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parameters: Vec::new(),
        }
    }

    pub fn with_params(name: impl Into<String>, params: Vec<String>) -> Self {
        Self {
            name: name.into(),
            parameters: params,
        }
    }
}

pub struct Validator {
    data: HashMap<String, Value>,
    rules: HashMap<String, Vec<Rule>>,
    custom_messages: HashMap<String, String>,
}

impl Validator {
    pub fn new(data: HashMap<String, Value>) -> Self {
        Self {
            data,
            rules: HashMap::new(),
            custom_messages: HashMap::new(),
        }
    }

    pub fn make(data: HashMap<String, Value>, rules: HashMap<String, Vec<Rule>>) -> Self {
        Self {
            data,
            rules,
            custom_messages: HashMap::new(),
        }
    }

    pub fn rules(&mut self, field: impl Into<String>, rules: Vec<Rule>) -> &mut Self {
        self.rules.insert(field.into(), rules);
        self
    }

    pub fn messages(&mut self, messages: HashMap<String, String>) -> &mut Self {
        self.custom_messages.extend(messages);
        self
    }

    pub fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        for (field, rules) in &self.rules {
            let value = self.data.get(field);

            for rule in rules {
                if let Err(error) = self.validate_rule(field, value, rule) {
                    errors.add(error);
                }
            }
        }

        if errors.has_errors() {
            Err(errors)
        } else {
            Ok(())
        }
    }

    fn validate_rule(&self, field: &str, value: Option<&Value>, rule: &Rule) -> Result<(), ValidationError> {
        match rule.name.as_str() {
            "required" => self.validate_required(field, value),
            "string" => self.validate_string(field, value),
            "numeric" => self.validate_numeric(field, value),
            "integer" => self.validate_integer(field, value),
            "boolean" => self.validate_boolean(field, value),
            "email" => self.validate_email(field, value),
            "url" => self.validate_url(field, value),
            "min" => self.validate_min(field, value, &rule.parameters),
            "max" => self.validate_max(field, value, &rule.parameters),
            "between" => self.validate_between(field, value, &rule.parameters),
            "size" => self.validate_size(field, value, &rule.parameters),
            "in" => self.validate_in(field, value, &rule.parameters),
            "not_in" => self.validate_not_in(field, value, &rule.parameters),
            "alpha" => self.validate_alpha(field, value),
            "alpha_num" => self.validate_alpha_num(field, value),
            "alpha_dash" => self.validate_alpha_dash(field, value),
            "regex" => self.validate_regex(field, value, &rule.parameters),
            "confirmed" => self.validate_confirmed(field, value),
            "same" => self.validate_same(field, value, &rule.parameters),
            "different" => self.validate_different(field, value, &rule.parameters),
            "unique" => self.validate_unique(field, value, &rule.parameters),
            "exists" => self.validate_exists(field, value, &rule.parameters),
            "date" => self.validate_date(field, value),
            "date_format" => self.validate_date_format(field, value, &rule.parameters),
            "before" => self.validate_before(field, value, &rule.parameters),
            "after" => self.validate_after(field, value, &rule.parameters),
            "json" => self.validate_json(field, value),
            "array" => self.validate_array(field, value),
            "nullable" => Ok(()), // Always passes
            "accepted" => self.validate_accepted(field, value),
            "active_url" => self.validate_active_url(field, value),
            "bail" => Ok(()), // Implemented at validator level
            "digits" => self.validate_digits(field, value, &rule.parameters),
            "digits_between" => self.validate_digits_between(field, value, &rule.parameters),
            "distinct" => self.validate_distinct(field, value),
            "ends_with" => self.validate_ends_with(field, value, &rule.parameters),
            "starts_with" => self.validate_starts_with(field, value, &rule.parameters),
            "filled" => self.validate_filled(field, value),
            "gt" => self.validate_gt(field, value, &rule.parameters),
            "gte" => self.validate_gte(field, value, &rule.parameters),
            "lt" => self.validate_lt(field, value, &rule.parameters),
            "lte" => self.validate_lte(field, value, &rule.parameters),
            "ip" => self.validate_ip(field, value),
            "ipv4" => self.validate_ipv4(field, value),
            "ipv6" => self.validate_ipv6(field, value),
            "mac_address" => self.validate_mac_address(field, value),
            "multiple_of" => self.validate_multiple_of(field, value, &rule.parameters),
            "not_regex" => self.validate_not_regex(field, value, &rule.parameters),
            "present" => self.validate_present(field, value),
            "prohibited" => self.validate_prohibited(field, value),
            "prohibited_if" => self.validate_prohibited_if(field, value, &rule.parameters),
            "prohibited_unless" => self.validate_prohibited_unless(field, value, &rule.parameters),
            "required_if" => self.validate_required_if(field, value, &rule.parameters),
            "required_unless" => self.validate_required_unless(field, value, &rule.parameters),
            "required_with" => self.validate_required_with(field, value, &rule.parameters),
            "required_with_all" => self.validate_required_with_all(field, value, &rule.parameters),
            "required_without" => self.validate_required_without(field, value, &rule.parameters),
            "required_without_all" => self.validate_required_without_all(field, value, &rule.parameters),
            "sometimes" => Ok(()), // Conditional validation, implemented at validator level
            "timezone" => self.validate_timezone(field, value),
            "uuid" => self.validate_uuid(field, value),
            // File validation rules
            "file" => self.validate_file(field, value),
            "image" => self.validate_image(field, value),
            "mimes" => self.validate_mimes(field, value, &rule.parameters),
            "mimetypes" => self.validate_mimetypes(field, value, &rule.parameters),
            "dimensions" => self.validate_dimensions(field, value, &rule.parameters),
            // Array validation rules
            "array_max" => self.validate_array_max(field, value, &rule.parameters),
            "array_min" => self.validate_array_min(field, value, &rule.parameters),
            // Date validation rules
            "after_or_equal" => self.validate_after_or_equal(field, value, &rule.parameters),
            "before_or_equal" => self.validate_before_or_equal(field, value, &rule.parameters),
            "date_equals" => self.validate_date_equals(field, value, &rule.parameters),
            _ => Err(ValidationError::new(field, &rule.name, format!("Unknown validation rule: {}", rule.name))),
        }
    }

    fn get_custom_message(&self, field: &str, rule: &str) -> Option<String> {
        let keys = [
            format!("{}.{}", field, rule),
            rule.to_string(),
        ];

        for key in &keys {
            if let Some(message) = self.custom_messages.get(key) {
                return Some(message.clone());
            }
        }

        None
    }

    fn error_message(&self, field: &str, rule: &str, default: &str) -> String {
        self.get_custom_message(field, rule)
            .unwrap_or_else(|| default.replace(":attribute", field))
    }

    fn validate_required(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        match value {
            Some(Value::Null) | None => Err(ValidationError::new(
                field,
                "required",
                self.error_message(field, "required", "The :attribute field is required.")
            )),
            Some(Value::String(s)) if s.is_empty() => Err(ValidationError::new(
                field,
                "required",
                self.error_message(field, "required", "The :attribute field is required.")
            )),
            Some(Value::Array(arr)) if arr.is_empty() => Err(ValidationError::new(
                field,
                "required",
                self.error_message(field, "required", "The :attribute field is required.")
            )),
            Some(Value::Object(obj)) if obj.is_empty() => Err(ValidationError::new(
                field,
                "required",
                self.error_message(field, "required", "The :attribute field is required.")
            )),
            _ => Ok(()),
        }
    }

    fn validate_string(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(val) = value {
            if !val.is_string() {
                return Err(ValidationError::new(
                    field,
                    "string",
                    self.error_message(field, "string", "The :attribute must be a string.")
                ));
            }
        }
        Ok(())
    }

    fn validate_numeric(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(val) = value {
            match val {
                Value::Number(_) => Ok(()),
                Value::String(s) => {
                    if s.parse::<f64>().is_err() {
                        Err(ValidationError::new(
                            field,
                            "numeric",
                            self.error_message(field, "numeric", "The :attribute must be a number.")
                        ))
                    } else {
                        Ok(())
                    }
                },
                _ => Err(ValidationError::new(
                    field,
                    "numeric",
                    self.error_message(field, "numeric", "The :attribute must be a number.")
                )),
            }
        } else {
            Ok(())
        }
    }

    fn validate_integer(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(val) = value {
            match val {
                Value::Number(n) => {
                    if n.is_f64() && n.as_f64().unwrap().fract() != 0.0 {
                        Err(ValidationError::new(
                            field,
                            "integer",
                            self.error_message(field, "integer", "The :attribute must be an integer.")
                        ))
                    } else {
                        Ok(())
                    }
                },
                Value::String(s) => {
                    if s.parse::<i64>().is_err() {
                        Err(ValidationError::new(
                            field,
                            "integer",
                            self.error_message(field, "integer", "The :attribute must be an integer.")
                        ))
                    } else {
                        Ok(())
                    }
                },
                _ => Err(ValidationError::new(
                    field,
                    "integer",
                    self.error_message(field, "integer", "The :attribute must be an integer.")
                )),
            }
        } else {
            Ok(())
        }
    }

    fn validate_boolean(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(val) = value {
            match val {
                Value::Bool(_) => Ok(()),
                Value::String(s) => {
                    let s_lower = s.to_lowercase();
                    if ["true", "false", "1", "0", "yes", "no", "on", "off"].contains(&s_lower.as_str()) {
                        Ok(())
                    } else {
                        Err(ValidationError::new(
                            field,
                            "boolean",
                            self.error_message(field, "boolean", "The :attribute field must be true or false.")
                        ))
                    }
                },
                Value::Number(n) => {
                    if n.as_i64() == Some(0) || n.as_i64() == Some(1) {
                        Ok(())
                    } else {
                        Err(ValidationError::new(
                            field,
                            "boolean",
                            self.error_message(field, "boolean", "The :attribute field must be true or false.")
                        ))
                    }
                },
                _ => Err(ValidationError::new(
                    field,
                    "boolean",
                    self.error_message(field, "boolean", "The :attribute field must be true or false.")
                )),
            }
        } else {
            Ok(())
        }
    }

    fn validate_email(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(Value::String(email)) = value {
            let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
            if !email_regex.is_match(email) {
                return Err(ValidationError::new(
                    field,
                    "email",
                    self.error_message(field, "email", "The :attribute must be a valid email address.")
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "email",
                self.error_message(field, "email", "The :attribute must be a valid email address.")
            ));
        }
        Ok(())
    }

    fn validate_url(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(Value::String(url)) = value {
            let url_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
            if !url_regex.is_match(url) {
                return Err(ValidationError::new(
                    field,
                    "url",
                    self.error_message(field, "url", "The :attribute format is invalid.")
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "url",
                self.error_message(field, "url", "The :attribute format is invalid.")
            ));
        }
        Ok(())
    }

    fn validate_min(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "min", "Min rule requires a parameter."));
        }

        let min_val: f64 = params[0].parse().map_err(|_| {
            ValidationError::new(field, "min", "Min rule parameter must be a number.")
        })?;

        if let Some(val) = value {
            let size = self.get_size(val)?;
            if size < min_val {
                return Err(ValidationError::new(
                    field,
                    "min",
                    self.error_message(field, "min", &format!("The :attribute must be at least {}.", min_val))
                ));
            }
        }
        Ok(())
    }

    fn validate_max(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "max", "Max rule requires a parameter."));
        }

        let max_val: f64 = params[0].parse().map_err(|_| {
            ValidationError::new(field, "max", "Max rule parameter must be a number.")
        })?;

        if let Some(val) = value {
            let size = self.get_size(val)?;
            if size > max_val {
                return Err(ValidationError::new(
                    field,
                    "max",
                    self.error_message(field, "max", &format!("The :attribute may not be greater than {}.", max_val))
                ));
            }
        }
        Ok(())
    }

    fn validate_between(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.len() < 2 {
            return Err(ValidationError::new(field, "between", "Between rule requires two parameters."));
        }

        let min_val: f64 = params[0].parse().map_err(|_| {
            ValidationError::new(field, "between", "Between rule parameters must be numbers.")
        })?;

        let max_val: f64 = params[1].parse().map_err(|_| {
            ValidationError::new(field, "between", "Between rule parameters must be numbers.")
        })?;

        if let Some(val) = value {
            let size = self.get_size(val)?;
            if size < min_val || size > max_val {
                return Err(ValidationError::new(
                    field,
                    "between",
                    self.error_message(field, "between", &format!("The :attribute must be between {} and {}.", min_val, max_val))
                ));
            }
        }
        Ok(())
    }

    fn validate_size(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "size", "Size rule requires a parameter."));
        }

        let size_val: f64 = params[0].parse().map_err(|_| {
            ValidationError::new(field, "size", "Size rule parameter must be a number.")
        })?;

        if let Some(val) = value {
            let size = self.get_size(val)?;
            if size != size_val {
                return Err(ValidationError::new(
                    field,
                    "size",
                    self.error_message(field, "size", &format!("The :attribute must be {}.", size_val))
                ));
            }
        }
        Ok(())
    }

    fn get_size(&self, value: &Value) -> Result<f64, ValidationError> {
        match value {
            Value::String(s) => Ok(s.len() as f64),
            Value::Number(n) => Ok(n.as_f64().unwrap_or(0.0)),
            Value::Array(arr) => Ok(arr.len() as f64),
            Value::Object(obj) => Ok(obj.len() as f64),
            _ => Ok(0.0),
        }
    }

    fn validate_in(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if let Some(val) = value {
            let val_str = match val {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => return Err(ValidationError::new(
                    field,
                    "in",
                    self.error_message(field, "in", "The selected :attribute is invalid.")
                )),
            };

            if !params.contains(&val_str) {
                return Err(ValidationError::new(
                    field,
                    "in",
                    self.error_message(field, "in", "The selected :attribute is invalid.")
                ));
            }
        }
        Ok(())
    }

    fn validate_not_in(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if let Some(val) = value {
            let val_str = match val {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => return Ok(()),
            };

            if params.contains(&val_str) {
                return Err(ValidationError::new(
                    field,
                    "not_in",
                    self.error_message(field, "not_in", "The selected :attribute is invalid.")
                ));
            }
        }
        Ok(())
    }

    fn validate_alpha(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(Value::String(s)) = value {
            if !s.chars().all(|c| c.is_alphabetic()) {
                return Err(ValidationError::new(
                    field,
                    "alpha",
                    self.error_message(field, "alpha", "The :attribute may only contain letters.")
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "alpha",
                self.error_message(field, "alpha", "The :attribute may only contain letters.")
            ));
        }
        Ok(())
    }

    fn validate_alpha_num(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(Value::String(s)) = value {
            if !s.chars().all(|c| c.is_alphanumeric()) {
                return Err(ValidationError::new(
                    field,
                    "alpha_num",
                    self.error_message(field, "alpha_num", "The :attribute may only contain letters and numbers.")
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "alpha_num",
                self.error_message(field, "alpha_num", "The :attribute may only contain letters and numbers.")
            ));
        }
        Ok(())
    }

    fn validate_alpha_dash(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(Value::String(s)) = value {
            if !s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                return Err(ValidationError::new(
                    field,
                    "alpha_dash",
                    self.error_message(field, "alpha_dash", "The :attribute may only contain letters, numbers, dashes and underscores.")
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "alpha_dash",
                self.error_message(field, "alpha_dash", "The :attribute may only contain letters, numbers, dashes and underscores.")
            ));
        }
        Ok(())
    }

    fn validate_regex(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "regex", "Regex rule requires a pattern parameter."));
        }

        let pattern = &params[0];
        let regex = Regex::new(pattern).map_err(|_| {
            ValidationError::new(field, "regex", "Invalid regex pattern.")
        })?;

        if let Some(Value::String(s)) = value {
            if !regex.is_match(s) {
                return Err(ValidationError::new(
                    field,
                    "regex",
                    self.error_message(field, "regex", "The :attribute format is invalid.")
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "regex",
                self.error_message(field, "regex", "The :attribute format is invalid.")
            ));
        }
        Ok(())
    }

    fn validate_confirmed(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        let confirmation_field = format!("{}_confirmation", field);
        let confirmation_value = self.data.get(&confirmation_field);

        match (value, confirmation_value) {
            (Some(val), Some(conf_val)) => {
                if val != conf_val {
                    Err(ValidationError::new(
                        field,
                        "confirmed",
                        self.error_message(field, "confirmed", "The :attribute confirmation does not match.")
                    ))
                } else {
                    Ok(())
                }
            },
            (Some(_), None) => Err(ValidationError::new(
                field,
                "confirmed",
                self.error_message(field, "confirmed", "The :attribute confirmation does not match.")
            )),
            _ => Ok(()),
        }
    }

    fn validate_same(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "same", "Same rule requires a field parameter."));
        }

        let other_field = &params[0];
        let other_value = self.data.get(other_field);

        if value != other_value {
            return Err(ValidationError::new(
                field,
                "same",
                self.error_message(field, "same", &format!("The :attribute and {} must match.", other_field))
            ));
        }
        Ok(())
    }

    fn validate_different(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "different", "Different rule requires a field parameter."));
        }

        let other_field = &params[0];
        let other_value = self.data.get(other_field);

        if value == other_value {
            return Err(ValidationError::new(
                field,
                "different",
                self.error_message(field, "different", &format!("The :attribute and {} must be different.", other_field))
            ));
        }
        Ok(())
    }

    fn validate_unique(&self, _field: &str, _value: Option<&Value>, _params: &[String]) -> Result<(), ValidationError> {
        // This would require database access to implement properly
        // For now, we'll just return Ok() as a placeholder
        Ok(())
    }

    fn validate_exists(&self, _field: &str, _value: Option<&Value>, _params: &[String]) -> Result<(), ValidationError> {
        // This would require database access to implement properly
        // For now, we'll just return Ok() as a placeholder
        Ok(())
    }

    fn validate_date(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(Value::String(date_str)) = value {
            // Try parsing common date formats
            let formats = [
                "%Y-%m-%d",
                "%Y-%m-%d %H:%M:%S",
                "%Y/%m/%d",
                "%d/%m/%Y",
                "%m/%d/%Y",
                "%Y-%m-%dT%H:%M:%S",
                "%Y-%m-%dT%H:%M:%SZ",
            ];

            let mut valid = false;
            for format in &formats {
                if chrono::NaiveDateTime::parse_from_str(date_str, format).is_ok() ||
                   chrono::NaiveDate::parse_from_str(date_str, format).is_ok() {
                    valid = true;
                    break;
                }
            }

            if !valid {
                return Err(ValidationError::new(
                    field,
                    "date",
                    self.error_message(field, "date", "The :attribute is not a valid date.")
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "date",
                self.error_message(field, "date", "The :attribute is not a valid date.")
            ));
        }
        Ok(())
    }

    fn validate_date_format(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "date_format", "Date format rule requires a format parameter."));
        }

        let format = &params[0];

        if let Some(Value::String(date_str)) = value {
            if chrono::NaiveDateTime::parse_from_str(date_str, format).is_err() &&
               chrono::NaiveDate::parse_from_str(date_str, format).is_err() {
                return Err(ValidationError::new(
                    field,
                    "date_format",
                    self.error_message(field, "date_format", &format!("The :attribute does not match the format {}.", format))
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "date_format",
                self.error_message(field, "date_format", &format!("The :attribute does not match the format {}.", format))
            ));
        }
        Ok(())
    }

    fn validate_before(&self, field: &str, _value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "before", "Before rule requires a date parameter."));
        }

        // This is a simplified implementation
        // In a real implementation, you'd parse both dates and compare them
        Ok(())
    }

    fn validate_after(&self, field: &str, _value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "after", "After rule requires a date parameter."));
        }

        // This is a simplified implementation
        // In a real implementation, you'd parse both dates and compare them
        Ok(())
    }

    fn validate_json(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(Value::String(json_str)) = value {
            if serde_json::from_str::<Value>(json_str).is_err() {
                return Err(ValidationError::new(
                    field,
                    "json",
                    self.error_message(field, "json", "The :attribute must be a valid JSON string.")
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "json",
                self.error_message(field, "json", "The :attribute must be a valid JSON string.")
            ));
        }
        Ok(())
    }

    fn validate_array(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(val) = value {
            if !val.is_array() {
                return Err(ValidationError::new(
                    field,
                    "array",
                    self.error_message(field, "array", "The :attribute must be an array.")
                ));
            }
        }
        Ok(())
    }

    // Additional Basic Validation Rules

    fn validate_accepted(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        match value {
            Some(Value::Bool(true)) => Ok(()),
            Some(Value::String(s)) => {
                let s_lower = s.to_lowercase();
                if ["yes", "on", "1", "true"].contains(&s_lower.as_str()) {
                    Ok(())
                } else {
                    Err(ValidationError::new(
                        field,
                        "accepted",
                        self.error_message(field, "accepted", "The :attribute must be accepted.")
                    ))
                }
            },
            Some(Value::Number(n)) => {
                if n.as_i64() == Some(1) {
                    Ok(())
                } else {
                    Err(ValidationError::new(
                        field,
                        "accepted",
                        self.error_message(field, "accepted", "The :attribute must be accepted.")
                    ))
                }
            },
            _ => Err(ValidationError::new(
                field,
                "accepted",
                self.error_message(field, "accepted", "The :attribute must be accepted.")
            )),
        }
    }

    fn validate_active_url(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(Value::String(url)) = value {
            // Basic URL validation with common schemes
            let url_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
            if !url_regex.is_match(url) {
                return Err(ValidationError::new(
                    field,
                    "active_url",
                    self.error_message(field, "active_url", "The :attribute is not a valid URL.")
                ));
            }
            // In a real implementation, you might want to make an HTTP request to verify the URL is active
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "active_url",
                self.error_message(field, "active_url", "The :attribute is not a valid URL.")
            ));
        }
        Ok(())
    }

    fn validate_digits(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "digits", "Digits rule requires a parameter."));
        }

        let digits_count: usize = params[0].parse().map_err(|_| {
            ValidationError::new(field, "digits", "Digits rule parameter must be a number.")
        })?;

        if let Some(Value::String(s)) = value {
            if s.chars().all(|c| c.is_numeric()) && s.len() == digits_count {
                Ok(())
            } else {
                Err(ValidationError::new(
                    field,
                    "digits",
                    self.error_message(field, "digits", &format!("The :attribute must be {} digits.", digits_count))
                ))
            }
        } else if let Some(Value::Number(n)) = value {
            let num_str = n.to_string();
            if num_str.chars().all(|c| c.is_numeric()) && num_str.len() == digits_count {
                Ok(())
            } else {
                Err(ValidationError::new(
                    field,
                    "digits",
                    self.error_message(field, "digits", &format!("The :attribute must be {} digits.", digits_count))
                ))
            }
        } else if value.is_some() {
            Err(ValidationError::new(
                field,
                "digits",
                self.error_message(field, "digits", &format!("The :attribute must be {} digits.", digits_count))
            ))
        } else {
            Ok(())
        }
    }

    fn validate_digits_between(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.len() < 2 {
            return Err(ValidationError::new(field, "digits_between", "Digits between rule requires two parameters."));
        }

        let min_digits: usize = params[0].parse().map_err(|_| {
            ValidationError::new(field, "digits_between", "Digits between rule parameters must be numbers.")
        })?;

        let max_digits: usize = params[1].parse().map_err(|_| {
            ValidationError::new(field, "digits_between", "Digits between rule parameters must be numbers.")
        })?;

        if let Some(val) = value {
            let digit_count = match val {
                Value::String(s) if s.chars().all(|c| c.is_numeric()) => s.len(),
                Value::Number(n) => n.to_string().chars().filter(|c| c.is_numeric()).count(),
                _ => return Err(ValidationError::new(
                    field,
                    "digits_between",
                    self.error_message(field, "digits_between", &format!("The :attribute must be between {} and {} digits.", min_digits, max_digits))
                )),
            };

            if digit_count >= min_digits && digit_count <= max_digits {
                Ok(())
            } else {
                Err(ValidationError::new(
                    field,
                    "digits_between",
                    self.error_message(field, "digits_between", &format!("The :attribute must be between {} and {} digits.", min_digits, max_digits))
                ))
            }
        } else {
            Ok(())
        }
    }

    fn validate_distinct(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        // This validation rule is typically used for array elements
        // For single values, it always passes
        if let Some(Value::Array(arr)) = value {
            let mut seen = std::collections::HashSet::new();
            for item in arr {
                if !seen.insert(item) {
                    return Err(ValidationError::new(
                        field,
                        "distinct",
                        self.error_message(field, "distinct", "The :attribute field has a duplicate value.")
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_ends_with(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "ends_with", "Ends with rule requires at least one parameter."));
        }

        if let Some(Value::String(s)) = value {
            let ends_with_any = params.iter().any(|suffix| s.ends_with(suffix));
            if !ends_with_any {
                return Err(ValidationError::new(
                    field,
                    "ends_with",
                    self.error_message(field, "ends_with", &format!("The :attribute must end with one of the following: {}.", params.join(", ")))
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "ends_with",
                self.error_message(field, "ends_with", &format!("The :attribute must end with one of the following: {}.", params.join(", ")))
            ));
        }
        Ok(())
    }

    fn validate_starts_with(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "starts_with", "Starts with rule requires at least one parameter."));
        }

        if let Some(Value::String(s)) = value {
            let starts_with_any = params.iter().any(|prefix| s.starts_with(prefix));
            if !starts_with_any {
                return Err(ValidationError::new(
                    field,
                    "starts_with",
                    self.error_message(field, "starts_with", &format!("The :attribute must start with one of the following: {}.", params.join(", ")))
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "starts_with",
                self.error_message(field, "starts_with", &format!("The :attribute must start with one of the following: {}.", params.join(", ")))
            ));
        }
        Ok(())
    }

    fn validate_filled(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(val) = value {
            match val {
                Value::Null => Err(ValidationError::new(
                    field,
                    "filled",
                    self.error_message(field, "filled", "The :attribute field must have a value when present.")
                )),
                Value::String(s) if s.is_empty() => Err(ValidationError::new(
                    field,
                    "filled",
                    self.error_message(field, "filled", "The :attribute field must have a value when present.")
                )),
                Value::Array(arr) if arr.is_empty() => Err(ValidationError::new(
                    field,
                    "filled",
                    self.error_message(field, "filled", "The :attribute field must have a value when present.")
                )),
                Value::Object(obj) if obj.is_empty() => Err(ValidationError::new(
                    field,
                    "filled",
                    self.error_message(field, "filled", "The :attribute field must have a value when present.")
                )),
                _ => Ok(()),
            }
        } else {
            Ok(()) // Field not present is OK for filled rule
        }
    }

    fn validate_gt(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "gt", "Greater than rule requires a parameter."));
        }

        let compare_field = &params[0];
        let compare_value = self.data.get(compare_field);

        if let (Some(val), Some(comp_val)) = (value, compare_value) {
            let val_size = self.get_size(val)?;
            let comp_size = self.get_size(comp_val)?;

            if val_size <= comp_size {
                return Err(ValidationError::new(
                    field,
                    "gt",
                    self.error_message(field, "gt", &format!("The :attribute must be greater than {}.", compare_field))
                ));
            }
        }
        Ok(())
    }

    fn validate_gte(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "gte", "Greater than or equal rule requires a parameter."));
        }

        let compare_field = &params[0];
        let compare_value = self.data.get(compare_field);

        if let (Some(val), Some(comp_val)) = (value, compare_value) {
            let val_size = self.get_size(val)?;
            let comp_size = self.get_size(comp_val)?;

            if val_size < comp_size {
                return Err(ValidationError::new(
                    field,
                    "gte",
                    self.error_message(field, "gte", &format!("The :attribute must be greater than or equal to {}.", compare_field))
                ));
            }
        }
        Ok(())
    }

    fn validate_lt(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "lt", "Less than rule requires a parameter."));
        }

        let compare_field = &params[0];
        let compare_value = self.data.get(compare_field);

        if let (Some(val), Some(comp_val)) = (value, compare_value) {
            let val_size = self.get_size(val)?;
            let comp_size = self.get_size(comp_val)?;

            if val_size >= comp_size {
                return Err(ValidationError::new(
                    field,
                    "lt",
                    self.error_message(field, "lt", &format!("The :attribute must be less than {}.", compare_field))
                ));
            }
        }
        Ok(())
    }

    fn validate_lte(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "lte", "Less than or equal rule requires a parameter."));
        }

        let compare_field = &params[0];
        let compare_value = self.data.get(compare_field);

        if let (Some(val), Some(comp_val)) = (value, compare_value) {
            let val_size = self.get_size(val)?;
            let comp_size = self.get_size(comp_val)?;

            if val_size > comp_size {
                return Err(ValidationError::new(
                    field,
                    "lte",
                    self.error_message(field, "lte", &format!("The :attribute must be less than or equal to {}.", compare_field))
                ));
            }
        }
        Ok(())
    }

    fn validate_ip(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(Value::String(ip)) = value {
            let ipv4_regex = Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();
            let ipv6_regex = Regex::new(r"^(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}$|^::1$|^::$").unwrap();

            if !ipv4_regex.is_match(ip) && !ipv6_regex.is_match(ip) {
                return Err(ValidationError::new(
                    field,
                    "ip",
                    self.error_message(field, "ip", "The :attribute must be a valid IP address.")
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "ip",
                self.error_message(field, "ip", "The :attribute must be a valid IP address.")
            ));
        }
        Ok(())
    }

    fn validate_ipv4(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(Value::String(ip)) = value {
            let ipv4_regex = Regex::new(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$").unwrap();
            if !ipv4_regex.is_match(ip) {
                return Err(ValidationError::new(
                    field,
                    "ipv4",
                    self.error_message(field, "ipv4", "The :attribute must be a valid IPv4 address.")
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "ipv4",
                self.error_message(field, "ipv4", "The :attribute must be a valid IPv4 address.")
            ));
        }
        Ok(())
    }

    fn validate_ipv6(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(Value::String(ip)) = value {
            let ipv6_regex = Regex::new(r"^(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}$|^::1$|^::$|^([0-9a-fA-F]{1,4}:){1,7}:$|^:((:[0-9a-fA-F]{1,4}){1,7}|:)$|^([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}$").unwrap();
            if !ipv6_regex.is_match(ip) {
                return Err(ValidationError::new(
                    field,
                    "ipv6",
                    self.error_message(field, "ipv6", "The :attribute must be a valid IPv6 address.")
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "ipv6",
                self.error_message(field, "ipv6", "The :attribute must be a valid IPv6 address.")
            ));
        }
        Ok(())
    }

    fn validate_mac_address(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(Value::String(mac)) = value {
            let mac_regex = Regex::new(r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$").unwrap();
            if !mac_regex.is_match(mac) {
                return Err(ValidationError::new(
                    field,
                    "mac_address",
                    self.error_message(field, "mac_address", "The :attribute must be a valid MAC address.")
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "mac_address",
                self.error_message(field, "mac_address", "The :attribute must be a valid MAC address.")
            ));
        }
        Ok(())
    }

    fn validate_multiple_of(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "multiple_of", "Multiple of rule requires a parameter."));
        }

        let multiple: f64 = params[0].parse().map_err(|_| {
            ValidationError::new(field, "multiple_of", "Multiple of rule parameter must be a number.")
        })?;

        if let Some(val) = value {
            let num = match val {
                Value::Number(n) => n.as_f64().unwrap_or(0.0),
                Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
                _ => return Err(ValidationError::new(
                    field,
                    "multiple_of",
                    self.error_message(field, "multiple_of", &format!("The :attribute must be a multiple of {}.", multiple))
                )),
            };

            if num % multiple != 0.0 {
                return Err(ValidationError::new(
                    field,
                    "multiple_of",
                    self.error_message(field, "multiple_of", &format!("The :attribute must be a multiple of {}.", multiple))
                ));
            }
        }
        Ok(())
    }

    fn validate_not_regex(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "not_regex", "Not regex rule requires a pattern parameter."));
        }

        let pattern = &params[0];
        let regex = Regex::new(pattern).map_err(|_| {
            ValidationError::new(field, "not_regex", "Invalid regex pattern.")
        })?;

        if let Some(Value::String(s)) = value {
            if regex.is_match(s) {
                return Err(ValidationError::new(
                    field,
                    "not_regex",
                    self.error_message(field, "not_regex", "The :attribute format is invalid.")
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "not_regex",
                self.error_message(field, "not_regex", "The :attribute format is invalid.")
            ));
        }
        Ok(())
    }

    fn validate_present(&self, field: &str, _value: Option<&Value>) -> Result<(), ValidationError> {
        if self.data.contains_key(field) {
            Ok(())
        } else {
            Err(ValidationError::new(
                field,
                "present",
                self.error_message(field, "present", "The :attribute field must be present.")
            ))
        }
    }

    fn validate_prohibited(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        match value {
            Some(Value::Null) | None => Ok(()),
            Some(Value::String(s)) if s.is_empty() => Ok(()),
            Some(Value::Array(arr)) if arr.is_empty() => Ok(()),
            Some(Value::Object(obj)) if obj.is_empty() => Ok(()),
            _ => Err(ValidationError::new(
                field,
                "prohibited",
                self.error_message(field, "prohibited", "The :attribute field is prohibited.")
            )),
        }
    }

    fn validate_prohibited_if(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.len() < 2 {
            return Err(ValidationError::new(field, "prohibited_if", "Prohibited if rule requires two parameters."));
        }

        let other_field = &params[0];
        let other_value = &params[1];

        if let Some(other_val) = self.data.get(other_field) {
            let other_str = match other_val {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => "".to_string(),
            };

            if other_str == *other_value {
                return self.validate_prohibited(field, value);
            }
        }
        Ok(())
    }

    fn validate_prohibited_unless(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.len() < 2 {
            return Err(ValidationError::new(field, "prohibited_unless", "Prohibited unless rule requires two parameters."));
        }

        let other_field = &params[0];
        let other_value = &params[1];

        if let Some(other_val) = self.data.get(other_field) {
            let other_str = match other_val {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => "".to_string(),
            };

            if other_str != *other_value {
                return self.validate_prohibited(field, value);
            }
        } else {
            return self.validate_prohibited(field, value);
        }
        Ok(())
    }

    fn validate_required_if(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.len() < 2 {
            return Err(ValidationError::new(field, "required_if", "Required if rule requires two parameters."));
        }

        let other_field = &params[0];
        let other_value = &params[1];

        if let Some(other_val) = self.data.get(other_field) {
            let other_str = match other_val {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => "".to_string(),
            };

            if other_str == *other_value {
                return self.validate_required(field, value);
            }
        }
        Ok(())
    }

    fn validate_required_unless(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.len() < 2 {
            return Err(ValidationError::new(field, "required_unless", "Required unless rule requires two parameters."));
        }

        let other_field = &params[0];
        let other_value = &params[1];

        if let Some(other_val) = self.data.get(other_field) {
            let other_str = match other_val {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => "".to_string(),
            };

            if other_str != *other_value {
                return self.validate_required(field, value);
            }
        } else {
            return self.validate_required(field, value);
        }
        Ok(())
    }

    fn validate_required_with(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "required_with", "Required with rule requires at least one parameter."));
        }

        for param in params {
            if let Some(other_val) = self.data.get(param) {
                match other_val {
                    Value::Null => continue,
                    Value::String(s) if s.is_empty() => continue,
                    Value::Array(arr) if arr.is_empty() => continue,
                    Value::Object(obj) if obj.is_empty() => continue,
                    _ => return self.validate_required(field, value),
                }
            }
        }
        Ok(())
    }

    fn validate_required_with_all(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "required_with_all", "Required with all rule requires at least one parameter."));
        }

        let all_present = params.iter().all(|param| {
            if let Some(other_val) = self.data.get(param) {
                match other_val {
                    Value::Null => false,
                    Value::String(s) if s.is_empty() => false,
                    Value::Array(arr) if arr.is_empty() => false,
                    Value::Object(obj) if obj.is_empty() => false,
                    _ => true,
                }
            } else {
                false
            }
        });

        if all_present {
            return self.validate_required(field, value);
        }
        Ok(())
    }

    fn validate_required_without(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "required_without", "Required without rule requires at least one parameter."));
        }

        for param in params {
            let other_val = self.data.get(param);
            let is_empty = match other_val {
                None => true,
                Some(Value::Null) => true,
                Some(Value::String(s)) if s.is_empty() => true,
                Some(Value::Array(arr)) if arr.is_empty() => true,
                Some(Value::Object(obj)) if obj.is_empty() => true,
                _ => false,
            };

            if is_empty {
                return self.validate_required(field, value);
            }
        }
        Ok(())
    }

    fn validate_required_without_all(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "required_without_all", "Required without all rule requires at least one parameter."));
        }

        let all_empty = params.iter().all(|param| {
            let other_val = self.data.get(param);
            match other_val {
                None => true,
                Some(Value::Null) => true,
                Some(Value::String(s)) if s.is_empty() => true,
                Some(Value::Array(arr)) if arr.is_empty() => true,
                Some(Value::Object(obj)) if obj.is_empty() => true,
                _ => false,
            }
        });

        if all_empty {
            return self.validate_required(field, value);
        }
        Ok(())
    }

    fn validate_timezone(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(Value::String(tz)) = value {
            // Basic timezone validation - in a real implementation you'd use a timezone library
            let common_timezones = [
                "UTC", "GMT", "America/New_York", "America/Chicago", "America/Denver", "America/Los_Angeles",
                "Europe/London", "Europe/Paris", "Europe/Berlin", "Asia/Tokyo", "Asia/Shanghai", "Australia/Sydney"
            ];

            if !common_timezones.contains(&tz.as_str()) && !tz.starts_with("Etc/") && !tz.contains("/") {
                return Err(ValidationError::new(
                    field,
                    "timezone",
                    self.error_message(field, "timezone", "The :attribute must be a valid timezone.")
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "timezone",
                self.error_message(field, "timezone", "The :attribute must be a valid timezone.")
            ));
        }
        Ok(())
    }

    fn validate_uuid(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        if let Some(Value::String(uuid_str)) = value {
            let uuid_regex = Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$").unwrap();
            if !uuid_regex.is_match(uuid_str) {
                return Err(ValidationError::new(
                    field,
                    "uuid",
                    self.error_message(field, "uuid", "The :attribute must be a valid UUID.")
                ));
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "uuid",
                self.error_message(field, "uuid", "The :attribute must be a valid UUID.")
            ));
        }
        Ok(())
    }

    // File Validation Rules (basic implementations)

    fn validate_file(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        // In a real implementation, this would check if the value represents a file upload
        // For now, we'll just check if it's an object with file-like properties
        if let Some(Value::Object(obj)) = value {
            if obj.contains_key("filename") || obj.contains_key("name") || obj.contains_key("type") {
                Ok(())
            } else {
                Err(ValidationError::new(
                    field,
                    "file",
                    self.error_message(field, "file", "The :attribute must be a file.")
                ))
            }
        } else if value.is_some() {
            Err(ValidationError::new(
                field,
                "file",
                self.error_message(field, "file", "The :attribute must be a file.")
            ))
        } else {
            Ok(())
        }
    }

    fn validate_image(&self, field: &str, value: Option<&Value>) -> Result<(), ValidationError> {
        // First validate it's a file
        self.validate_file(field, value)?;

        if let Some(Value::Object(obj)) = value {
            if let Some(Value::String(mime_type)) = obj.get("type").or_else(|| obj.get("mime_type")) {
                if !mime_type.starts_with("image/") {
                    return Err(ValidationError::new(
                        field,
                        "image",
                        self.error_message(field, "image", "The :attribute must be an image.")
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_mimes(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "mimes", "Mimes rule requires at least one parameter."));
        }

        if let Some(Value::Object(obj)) = value {
            if let Some(Value::String(filename)) = obj.get("filename").or_else(|| obj.get("name")) {
                let extension = filename.split('.').last().unwrap_or("").to_lowercase();
                if !params.iter().any(|ext| ext.to_lowercase() == extension) {
                    return Err(ValidationError::new(
                        field,
                        "mimes",
                        self.error_message(field, "mimes", &format!("The :attribute must be a file of type: {}.", params.join(", ")))
                    ));
                }
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "mimes",
                self.error_message(field, "mimes", &format!("The :attribute must be a file of type: {}.", params.join(", ")))
            ));
        }
        Ok(())
    }

    fn validate_mimetypes(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "mimetypes", "Mimetypes rule requires at least one parameter."));
        }

        if let Some(Value::Object(obj)) = value {
            if let Some(Value::String(mime_type)) = obj.get("type").or_else(|| obj.get("mime_type")) {
                if !params.contains(mime_type) {
                    return Err(ValidationError::new(
                        field,
                        "mimetypes",
                        self.error_message(field, "mimetypes", &format!("The :attribute must be a file of type: {}.", params.join(", ")))
                    ));
                }
            }
        } else if value.is_some() {
            return Err(ValidationError::new(
                field,
                "mimetypes",
                self.error_message(field, "mimetypes", &format!("The :attribute must be a file of type: {}.", params.join(", ")))
            ));
        }
        Ok(())
    }

    fn validate_dimensions(&self, field: &str, value: Option<&Value>, _params: &[String]) -> Result<(), ValidationError> {
        // Basic implementation - in reality you'd parse dimension constraints
        if let Some(Value::Object(obj)) = value {
            if obj.get("width").is_some() && obj.get("height").is_some() {
                Ok(())
            } else {
                Err(ValidationError::new(
                    field,
                    "dimensions",
                    self.error_message(field, "dimensions", "The :attribute has invalid image dimensions.")
                ))
            }
        } else if value.is_some() {
            Err(ValidationError::new(
                field,
                "dimensions",
                self.error_message(field, "dimensions", "The :attribute has invalid image dimensions.")
            ))
        } else {
            Ok(())
        }
    }

    // Array Validation Rules

    fn validate_array_max(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "array_max", "Array max rule requires a parameter."));
        }

        let max_count: usize = params[0].parse().map_err(|_| {
            ValidationError::new(field, "array_max", "Array max rule parameter must be a number.")
        })?;

        if let Some(Value::Array(arr)) = value {
            if arr.len() > max_count {
                return Err(ValidationError::new(
                    field,
                    "array_max",
                    self.error_message(field, "array_max", &format!("The :attribute may not have more than {} items.", max_count))
                ));
            }
        }
        Ok(())
    }

    fn validate_array_min(&self, field: &str, value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "array_min", "Array min rule requires a parameter."));
        }

        let min_count: usize = params[0].parse().map_err(|_| {
            ValidationError::new(field, "array_min", "Array min rule parameter must be a number.")
        })?;

        if let Some(Value::Array(arr)) = value {
            if arr.len() < min_count {
                return Err(ValidationError::new(
                    field,
                    "array_min",
                    self.error_message(field, "array_min", &format!("The :attribute must have at least {} items.", min_count))
                ));
            }
        }
        Ok(())
    }

    // Date Validation Rules

    fn validate_after_or_equal(&self, field: &str, _value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "after_or_equal", "After or equal rule requires a date parameter."));
        }
        // Simplified implementation - in reality you'd parse and compare dates
        Ok(())
    }

    fn validate_before_or_equal(&self, field: &str, _value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "before_or_equal", "Before or equal rule requires a date parameter."));
        }
        // Simplified implementation - in reality you'd parse and compare dates
        Ok(())
    }

    fn validate_date_equals(&self, field: &str, _value: Option<&Value>, params: &[String]) -> Result<(), ValidationError> {
        if params.is_empty() {
            return Err(ValidationError::new(field, "date_equals", "Date equals rule requires a date parameter."));
        }
        // Simplified implementation - in reality you'd parse and compare dates
        Ok(())
    }
}

// Helper functions for easier rule creation
pub fn required() -> Rule {
    Rule::new("required")
}

pub fn string() -> Rule {
    Rule::new("string")
}

pub fn numeric() -> Rule {
    Rule::new("numeric")
}

pub fn integer() -> Rule {
    Rule::new("integer")
}

pub fn boolean() -> Rule {
    Rule::new("boolean")
}

pub fn email() -> Rule {
    Rule::new("email")
}

pub fn url() -> Rule {
    Rule::new("url")
}

pub fn min(value: impl ToString) -> Rule {
    Rule::with_params("min", vec![value.to_string()])
}

pub fn max(value: impl ToString) -> Rule {
    Rule::with_params("max", vec![value.to_string()])
}

pub fn between(min: impl ToString, max: impl ToString) -> Rule {
    Rule::with_params("between", vec![min.to_string(), max.to_string()])
}

pub fn size(value: impl ToString) -> Rule {
    Rule::with_params("size", vec![value.to_string()])
}

pub fn in_list(values: Vec<impl ToString>) -> Rule {
    Rule::with_params("in", values.into_iter().map(|v| v.to_string()).collect())
}

pub fn not_in(values: Vec<impl ToString>) -> Rule {
    Rule::with_params("not_in", values.into_iter().map(|v| v.to_string()).collect())
}

pub fn alpha() -> Rule {
    Rule::new("alpha")
}

pub fn alpha_num() -> Rule {
    Rule::new("alpha_num")
}

pub fn alpha_dash() -> Rule {
    Rule::new("alpha_dash")
}

pub fn regex(pattern: impl ToString) -> Rule {
    Rule::with_params("regex", vec![pattern.to_string()])
}

pub fn confirmed() -> Rule {
    Rule::new("confirmed")
}

pub fn same(field: impl ToString) -> Rule {
    Rule::with_params("same", vec![field.to_string()])
}

pub fn different(field: impl ToString) -> Rule {
    Rule::with_params("different", vec![field.to_string()])
}

pub fn unique(table: impl ToString, column: Option<impl ToString>) -> Rule {
    let mut params = vec![table.to_string()];
    if let Some(col) = column {
        params.push(col.to_string());
    }
    Rule::with_params("unique", params)
}

pub fn exists(table: impl ToString, column: Option<impl ToString>) -> Rule {
    let mut params = vec![table.to_string()];
    if let Some(col) = column {
        params.push(col.to_string());
    }
    Rule::with_params("exists", params)
}

pub fn date() -> Rule {
    Rule::new("date")
}

pub fn date_format(format: impl ToString) -> Rule {
    Rule::with_params("date_format", vec![format.to_string()])
}

pub fn before(date: impl ToString) -> Rule {
    Rule::with_params("before", vec![date.to_string()])
}

pub fn after(date: impl ToString) -> Rule {
    Rule::with_params("after", vec![date.to_string()])
}

pub fn json() -> Rule {
    Rule::new("json")
}

pub fn array() -> Rule {
    Rule::new("array")
}

pub fn nullable() -> Rule {
    Rule::new("nullable")
}

// Additional validation rule helpers

pub fn accepted() -> Rule {
    Rule::new("accepted")
}

pub fn active_url() -> Rule {
    Rule::new("active_url")
}

pub fn bail() -> Rule {
    Rule::new("bail")
}

pub fn digits(count: impl ToString) -> Rule {
    Rule::with_params("digits", vec![count.to_string()])
}

pub fn digits_between(min: impl ToString, max: impl ToString) -> Rule {
    Rule::with_params("digits_between", vec![min.to_string(), max.to_string()])
}

pub fn distinct() -> Rule {
    Rule::new("distinct")
}

pub fn ends_with(values: Vec<impl ToString>) -> Rule {
    Rule::with_params("ends_with", values.into_iter().map(|v| v.to_string()).collect())
}

pub fn starts_with(values: Vec<impl ToString>) -> Rule {
    Rule::with_params("starts_with", values.into_iter().map(|v| v.to_string()).collect())
}

pub fn filled() -> Rule {
    Rule::new("filled")
}

pub fn gt(field: impl ToString) -> Rule {
    Rule::with_params("gt", vec![field.to_string()])
}

pub fn gte(field: impl ToString) -> Rule {
    Rule::with_params("gte", vec![field.to_string()])
}

pub fn lt(field: impl ToString) -> Rule {
    Rule::with_params("lt", vec![field.to_string()])
}

pub fn lte(field: impl ToString) -> Rule {
    Rule::with_params("lte", vec![field.to_string()])
}

pub fn ip() -> Rule {
    Rule::new("ip")
}

pub fn ipv4() -> Rule {
    Rule::new("ipv4")
}

pub fn ipv6() -> Rule {
    Rule::new("ipv6")
}

pub fn mac_address() -> Rule {
    Rule::new("mac_address")
}

pub fn multiple_of(value: impl ToString) -> Rule {
    Rule::with_params("multiple_of", vec![value.to_string()])
}

pub fn not_regex(pattern: impl ToString) -> Rule {
    Rule::with_params("not_regex", vec![pattern.to_string()])
}

pub fn present() -> Rule {
    Rule::new("present")
}

pub fn prohibited() -> Rule {
    Rule::new("prohibited")
}

pub fn prohibited_if(field: impl ToString, value: impl ToString) -> Rule {
    Rule::with_params("prohibited_if", vec![field.to_string(), value.to_string()])
}

pub fn prohibited_unless(field: impl ToString, value: impl ToString) -> Rule {
    Rule::with_params("prohibited_unless", vec![field.to_string(), value.to_string()])
}

pub fn required_if(field: impl ToString, value: impl ToString) -> Rule {
    Rule::with_params("required_if", vec![field.to_string(), value.to_string()])
}

pub fn required_unless(field: impl ToString, value: impl ToString) -> Rule {
    Rule::with_params("required_unless", vec![field.to_string(), value.to_string()])
}

pub fn required_with(fields: Vec<impl ToString>) -> Rule {
    Rule::with_params("required_with", fields.into_iter().map(|f| f.to_string()).collect())
}

pub fn required_with_all(fields: Vec<impl ToString>) -> Rule {
    Rule::with_params("required_with_all", fields.into_iter().map(|f| f.to_string()).collect())
}

pub fn required_without(fields: Vec<impl ToString>) -> Rule {
    Rule::with_params("required_without", fields.into_iter().map(|f| f.to_string()).collect())
}

pub fn required_without_all(fields: Vec<impl ToString>) -> Rule {
    Rule::with_params("required_without_all", fields.into_iter().map(|f| f.to_string()).collect())
}

pub fn sometimes() -> Rule {
    Rule::new("sometimes")
}

pub fn timezone() -> Rule {
    Rule::new("timezone")
}

pub fn uuid() -> Rule {
    Rule::new("uuid")
}

// File validation rules

pub fn file() -> Rule {
    Rule::new("file")
}

pub fn image() -> Rule {
    Rule::new("image")
}

pub fn mimes(extensions: Vec<impl ToString>) -> Rule {
    Rule::with_params("mimes", extensions.into_iter().map(|e| e.to_string()).collect())
}

pub fn mimetypes(types: Vec<impl ToString>) -> Rule {
    Rule::with_params("mimetypes", types.into_iter().map(|t| t.to_string()).collect())
}

pub fn dimensions(constraints: Vec<impl ToString>) -> Rule {
    Rule::with_params("dimensions", constraints.into_iter().map(|c| c.to_string()).collect())
}

// Array validation rules

pub fn array_max(count: impl ToString) -> Rule {
    Rule::with_params("array_max", vec![count.to_string()])
}

pub fn array_min(count: impl ToString) -> Rule {
    Rule::with_params("array_min", vec![count.to_string()])
}

// Date validation rules

pub fn after_or_equal(date: impl ToString) -> Rule {
    Rule::with_params("after_or_equal", vec![date.to_string()])
}

pub fn before_or_equal(date: impl ToString) -> Rule {
    Rule::with_params("before_or_equal", vec![date.to_string()])
}

pub fn date_equals(date: impl ToString) -> Rule {
    Rule::with_params("date_equals", vec![date.to_string()])
}