use crate::app::validation::errors::ValidationError;
use serde_json::Value;
use std::collections::HashMap;
use regex::Regex;
use crate::database::DbPool;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};

#[async_trait]
pub trait Rule: Send + Sync {
    async fn validate(&self, field: &str, value: &Value, data: &HashMap<String, Value>, db: Option<&DbPool>) -> Result<(), ValidationError>;
}

pub struct RequiredRule;

#[async_trait]
impl Rule for RequiredRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::Null => Err(ValidationError::new("required", &format!("{} is required.", field))),
            Value::String(s) if s.trim().is_empty() => Err(ValidationError::new("required", &format!("{} is required.", field))),
            Value::Array(arr) if arr.is_empty() => Err(ValidationError::new("required", &format!("{} is required.", field))),
            Value::Object(obj) if obj.is_empty() => Err(ValidationError::new("required", &format!("{} is required.", field))),
            _ => Ok(()),
        }
    }
}

pub struct StringRule;

#[async_trait]
impl Rule for StringRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(_) => Ok(()),
            Value::Null => Ok(()), // Allow null values, use Required rule to enforce presence
            _ => Err(ValidationError::new("string", &format!("The {} must be a string.", field))),
        }
    }
}

pub struct MinRule {
    pub min: usize,
}

impl MinRule {
    pub fn new(min: usize) -> Self {
        Self { min }
    }
}

#[async_trait]
impl Rule for MinRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                if s.len() < self.min {
                    Err(ValidationError::new("min", &format!("The {} must be at least {} characters.", field, self.min)))
                } else {
                    Ok(())
                }
            },
            Value::Array(arr) => {
                if arr.len() < self.min {
                    Err(ValidationError::new("min", &format!("The {} must have at least {} items.", field, self.min)))
                } else {
                    Ok(())
                }
            },
            Value::Number(n) => {
                if let Some(num) = n.as_i64() {
                    if (num as usize) < self.min {
                        Err(ValidationError::new("min", &format!("The {} must be at least {}.", field, self.min)))
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            },
            Value::Null => Ok(()), // Allow null values
            _ => Ok(()),
        }
    }
}

pub struct MaxRule {
    pub max: usize,
}

impl MaxRule {
    pub fn new(max: usize) -> Self {
        Self { max }
    }
}

#[async_trait]
impl Rule for MaxRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                if s.len() > self.max {
                    Err(ValidationError::new("max", &format!("The {} may not be greater than {} characters.", field, self.max)))
                } else {
                    Ok(())
                }
            },
            Value::Array(arr) => {
                if arr.len() > self.max {
                    Err(ValidationError::new("max", &format!("The {} may not have more than {} items.", field, self.max)))
                } else {
                    Ok(())
                }
            },
            Value::Number(n) => {
                if let Some(num) = n.as_i64() {
                    if (num as usize) > self.max {
                        Err(ValidationError::new("max", &format!("The {} may not be greater than {}.", field, self.max)))
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            },
            Value::Null => Ok(()), // Allow null values
            _ => Ok(()),
        }
    }
}

pub struct EmailRule;

#[async_trait]
impl Rule for EmailRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
                if email_regex.is_match(s) {
                    Ok(())
                } else {
                    Err(ValidationError::new("email", &format!("The {} must be a valid email address.", field)))
                }
            },
            Value::Null => Ok(()), // Allow null values
            _ => Err(ValidationError::new("email", &format!("The {} must be a valid email address.", field))),
        }
    }
}

pub struct NumericRule;

#[async_trait]
impl Rule for NumericRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::Number(_) => Ok(()),
            Value::String(s) => {
                if s.parse::<f64>().is_ok() {
                    Ok(())
                } else {
                    Err(ValidationError::new("numeric", &format!("The {} must be a number.", field)))
                }
            },
            Value::Null => Ok(()), // Allow null values
            _ => Err(ValidationError::new("numeric", &format!("The {} must be a number.", field))),
        }
    }
}

pub struct IntegerRule;

#[async_trait]
impl Rule for IntegerRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::Number(n) => {
                if n.is_i64() {
                    Ok(())
                } else {
                    Err(ValidationError::new("integer", &format!("The {} must be an integer.", field)))
                }
            },
            Value::String(s) => {
                if s.parse::<i64>().is_ok() {
                    Ok(())
                } else {
                    Err(ValidationError::new("integer", &format!("The {} must be an integer.", field)))
                }
            },
            Value::Null => Ok(()), // Allow null values
            _ => Err(ValidationError::new("integer", &format!("The {} must be an integer.", field))),
        }
    }
}

pub struct BooleanRule;

#[async_trait]
impl Rule for BooleanRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::Bool(_) => Ok(()),
            Value::String(s) => {
                if s == "true" || s == "false" || s == "1" || s == "0" {
                    Ok(())
                } else {
                    Err(ValidationError::new("boolean", &format!("The {} field must be true or false.", field)))
                }
            },
            Value::Number(n) => {
                if let Some(num) = n.as_i64() {
                    if num == 0 || num == 1 {
                        Ok(())
                    } else {
                        Err(ValidationError::new("boolean", &format!("The {} field must be true or false.", field)))
                    }
                } else {
                    Err(ValidationError::new("boolean", &format!("The {} field must be true or false.", field)))
                }
            },
            Value::Null => Ok(()), // Allow null values
            _ => Err(ValidationError::new("boolean", &format!("The {} field must be true or false.", field))),
        }
    }
}

pub struct ArrayRule;

#[async_trait]
impl Rule for ArrayRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::Array(_) => Ok(()),
            Value::Null => Ok(()), // Allow null values
            _ => Err(ValidationError::new("array", &format!("The {} must be an array.", field))),
        }
    }
}

pub struct UniqueRule {
    pub table: String,
    pub column: Option<String>,
}

impl UniqueRule {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            column: None,
        }
    }

    pub fn with_column(table: &str, column: &str) -> Self {
        Self {
            table: table.to_string(),
            column: Some(column.to_string()),
        }
    }
}

#[async_trait]
impl Rule for UniqueRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, db: Option<&DbPool>) -> Result<(), ValidationError> {
        if let Some(db) = db {
            let default_field = field.to_string();
            let column = self.column.as_ref().unwrap_or(&default_field);
            let str_value = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => return Ok(()), // Allow null values
                _ => return Err(ValidationError::new("unique", &format!("The {} field must be a valid value for uniqueness check.", field))),
            };

            let query = format!("SELECT COUNT(*) as count FROM {} WHERE {} = $1", self.table, column);

            match sqlx::query_scalar::<_, i64>(&query)
                .bind(&str_value)
                .fetch_one(db)
                .await
            {
                Ok(count) => {
                    if count > 0 {
                        Err(ValidationError::new("unique", &format!("The {} has already been taken.", field)))
                    } else {
                        Ok(())
                    }
                },
                Err(_) => Err(ValidationError::new("unique", &format!("Unable to validate uniqueness for {}.", field))),
            }
        } else {
            Err(ValidationError::new("unique", "Database connection required for unique validation."))
        }
    }
}

// String validation rules
pub struct AlphaRule;

#[async_trait]
impl Rule for AlphaRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                if s.chars().all(|c| c.is_alphabetic()) {
                    Ok(())
                } else {
                    Err(ValidationError::new("alpha", &format!("The {} may only contain letters.", field)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("alpha", &format!("The {} may only contain letters.", field))),
        }
    }
}

pub struct AlphaDashRule;

#[async_trait]
impl Rule for AlphaDashRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                if s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                    Ok(())
                } else {
                    Err(ValidationError::new("alpha_dash", &format!("The {} may only contain letters, numbers, dashes and underscores.", field)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("alpha_dash", &format!("The {} may only contain letters, numbers, dashes and underscores.", field))),
        }
    }
}

pub struct AlphaNumRule;

#[async_trait]
impl Rule for AlphaNumRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                if s.chars().all(|c| c.is_alphanumeric()) {
                    Ok(())
                } else {
                    Err(ValidationError::new("alpha_num", &format!("The {} may only contain letters and numbers.", field)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("alpha_num", &format!("The {} may only contain letters and numbers.", field))),
        }
    }
}

pub struct BetweenRule {
    pub min: f64,
    pub max: f64,
}

impl BetweenRule {
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }
}

#[async_trait]
impl Rule for BetweenRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::Number(n) => {
                if let Some(num) = n.as_f64() {
                    if num >= self.min && num <= self.max {
                        Ok(())
                    } else {
                        Err(ValidationError::new("between", &format!("The {} must be between {} and {}.", field, self.min, self.max)))
                    }
                } else {
                    Err(ValidationError::new("between", &format!("The {} must be a valid number.", field)))
                }
            },
            Value::String(s) => {
                let len = s.len() as f64;
                if len >= self.min && len <= self.max {
                    Ok(())
                } else {
                    Err(ValidationError::new("between", &format!("The {} must be between {} and {} characters.", field, self.min as usize, self.max as usize)))
                }
            },
            Value::Array(arr) => {
                let len = arr.len() as f64;
                if len >= self.min && len <= self.max {
                    Ok(())
                } else {
                    Err(ValidationError::new("between", &format!("The {} must have between {} and {} items.", field, self.min as usize, self.max as usize)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("between", &format!("The {} must be between {} and {}.", field, self.min, self.max))),
        }
    }
}

pub struct DigitsRule {
    pub digits: usize,
}

impl DigitsRule {
    pub fn new(digits: usize) -> Self {
        Self { digits }
    }
}

#[async_trait]
impl Rule for DigitsRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                if s.chars().all(|c| c.is_ascii_digit()) && s.len() == self.digits {
                    Ok(())
                } else {
                    Err(ValidationError::new("digits", &format!("The {} must be {} digits.", field, self.digits)))
                }
            },
            Value::Number(n) => {
                let num_str = n.to_string();
                if num_str.chars().all(|c| c.is_ascii_digit()) && num_str.len() == self.digits {
                    Ok(())
                } else {
                    Err(ValidationError::new("digits", &format!("The {} must be {} digits.", field, self.digits)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("digits", &format!("The {} must be {} digits.", field, self.digits))),
        }
    }
}

pub struct DigitsBetweenRule {
    pub min: usize,
    pub max: usize,
}

impl DigitsBetweenRule {
    pub fn new(min: usize, max: usize) -> Self {
        Self { min, max }
    }
}

#[async_trait]
impl Rule for DigitsBetweenRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                if s.chars().all(|c| c.is_ascii_digit()) && s.len() >= self.min && s.len() <= self.max {
                    Ok(())
                } else {
                    Err(ValidationError::new("digits_between", &format!("The {} must be between {} and {} digits.", field, self.min, self.max)))
                }
            },
            Value::Number(n) => {
                let num_str = n.to_string();
                if num_str.chars().all(|c| c.is_ascii_digit()) && num_str.len() >= self.min && num_str.len() <= self.max {
                    Ok(())
                } else {
                    Err(ValidationError::new("digits_between", &format!("The {} must be between {} and {} digits.", field, self.min, self.max)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("digits_between", &format!("The {} must be between {} and {} digits.", field, self.min, self.max))),
        }
    }
}

pub struct SizeRule {
    pub size: usize,
}

impl SizeRule {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

#[async_trait]
impl Rule for SizeRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                if s.len() == self.size {
                    Ok(())
                } else {
                    Err(ValidationError::new("size", &format!("The {} must be {} characters.", field, self.size)))
                }
            },
            Value::Array(arr) => {
                if arr.len() == self.size {
                    Ok(())
                } else {
                    Err(ValidationError::new("size", &format!("The {} must have {} items.", field, self.size)))
                }
            },
            Value::Number(n) => {
                if let Some(num) = n.as_u64() {
                    if num == self.size as u64 {
                        Ok(())
                    } else {
                        Err(ValidationError::new("size", &format!("The {} must be {}.", field, self.size)))
                    }
                } else {
                    Err(ValidationError::new("size", &format!("The {} must be {}.", field, self.size)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("size", &format!("The {} must be size {}.", field, self.size))),
        }
    }
}

// Date validation rules
pub struct DateRule;

#[async_trait]
impl Rule for DateRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                // Try various date formats
                let formats = [
                    "%Y-%m-%d",
                    "%Y/%m/%d",
                    "%d-%m-%Y",
                    "%d/%m/%Y",
                    "%Y-%m-%d %H:%M:%S",
                    "%Y-%m-%dT%H:%M:%S",
                    "%Y-%m-%dT%H:%M:%SZ",
                    "%Y-%m-%dT%H:%M:%S%.3fZ",
                ];

                for format in &formats {
                    if NaiveDate::parse_from_str(s, format).is_ok() ||
                       NaiveDateTime::parse_from_str(s, format).is_ok() ||
                       DateTime::parse_from_rfc3339(s).is_ok() {
                        return Ok(());
                    }
                }

                Err(ValidationError::new("date", &format!("The {} is not a valid date.", field)))
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("date", &format!("The {} is not a valid date.", field))),
        }
    }
}

pub struct BeforeRule {
    pub date: String,
}

impl BeforeRule {
    pub fn new(date: &str) -> Self {
        Self {
            date: date.to_string(),
        }
    }
}

#[async_trait]
impl Rule for BeforeRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                let compare_date = if self.date == "today" {
                    Utc::now().naive_utc().date()
                } else if let Ok(date) = NaiveDate::parse_from_str(&self.date, "%Y-%m-%d") {
                    date
                } else {
                    return Err(ValidationError::new("before", &format!("Invalid comparison date for {} validation.", field)));
                };

                if let Ok(field_date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    if field_date < compare_date {
                        Ok(())
                    } else {
                        Err(ValidationError::new("before", &format!("The {} must be a date before {}.", field, self.date)))
                    }
                } else {
                    Err(ValidationError::new("before", &format!("The {} is not a valid date.", field)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("before", &format!("The {} must be a valid date.", field))),
        }
    }
}

pub struct AfterRule {
    pub date: String,
}

impl AfterRule {
    pub fn new(date: &str) -> Self {
        Self {
            date: date.to_string(),
        }
    }
}

#[async_trait]
impl Rule for AfterRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                let compare_date = if self.date == "today" {
                    Utc::now().naive_utc().date()
                } else if let Ok(date) = NaiveDate::parse_from_str(&self.date, "%Y-%m-%d") {
                    date
                } else {
                    return Err(ValidationError::new("after", &format!("Invalid comparison date for {} validation.", field)));
                };

                if let Ok(field_date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    if field_date > compare_date {
                        Ok(())
                    } else {
                        Err(ValidationError::new("after", &format!("The {} must be a date after {}.", field, self.date)))
                    }
                } else {
                    Err(ValidationError::new("after", &format!("The {} is not a valid date.", field)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("after", &format!("The {} must be a valid date.", field))),
        }
    }
}

pub struct DateFormatRule {
    pub format: String,
}

impl DateFormatRule {
    pub fn new(format: &str) -> Self {
        Self {
            format: format.to_string(),
        }
    }
}

#[async_trait]
impl Rule for DateFormatRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                if NaiveDate::parse_from_str(s, &self.format).is_ok() ||
                   NaiveDateTime::parse_from_str(s, &self.format).is_ok() {
                    Ok(())
                } else {
                    Err(ValidationError::new("date_format", &format!("The {} does not match the format {}.", field, self.format)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("date_format", &format!("The {} must be a valid date string.", field))),
        }
    }
}

// Array and list validation rules
pub struct InRule {
    pub values: Vec<String>,
}

impl InRule {
    pub fn new(values: Vec<String>) -> Self {
        Self { values }
    }
}

#[async_trait]
impl Rule for InRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        let str_value = match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => return Ok(()),
            _ => return Err(ValidationError::new("in", &format!("The selected {} is invalid.", field))),
        };

        if self.values.contains(&str_value) {
            Ok(())
        } else {
            Err(ValidationError::new("in", &format!("The selected {} is invalid.", field)))
        }
    }
}

pub struct NotInRule {
    pub values: Vec<String>,
}

impl NotInRule {
    pub fn new(values: Vec<String>) -> Self {
        Self { values }
    }
}

#[async_trait]
impl Rule for NotInRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        let str_value = match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => return Ok(()),
            _ => return Err(ValidationError::new("not_in", &format!("The {} field contains an invalid value.", field))),
        };

        if !self.values.contains(&str_value) {
            Ok(())
        } else {
            Err(ValidationError::new("not_in", &format!("The selected {} is invalid.", field)))
        }
    }
}

// Format validation rules
pub struct UrlRule;

#[async_trait]
impl Rule for UrlRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                let url_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
                if url_regex.is_match(s) {
                    Ok(())
                } else {
                    Err(ValidationError::new("url", &format!("The {} format is invalid.", field)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("url", &format!("The {} format is invalid.", field))),
        }
    }
}

pub struct UuidRule;

#[async_trait]
impl Rule for UuidRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                let uuid_regex = Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$").unwrap();
                if uuid_regex.is_match(s) {
                    Ok(())
                } else {
                    Err(ValidationError::new("uuid", &format!("The {} must be a valid UUID.", field)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("uuid", &format!("The {} must be a valid UUID.", field))),
        }
    }
}

pub struct UlidRule;

#[async_trait]
impl Rule for UlidRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                // ULID format: 26 characters, base32 encoded
                let ulid_regex = Regex::new(r"^[0123456789ABCDEFGHJKMNPQRSTVWXYZ]{26}$").unwrap();
                if s.len() == 26 && ulid_regex.is_match(s) {
                    Ok(())
                } else {
                    Err(ValidationError::new("ulid", &format!("The {} must be a valid ULID.", field)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("ulid", &format!("The {} must be a valid ULID.", field))),
        }
    }
}

pub struct JsonRule;

#[async_trait]
impl Rule for JsonRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                if serde_json::from_str::<serde_json::Value>(s).is_ok() {
                    Ok(())
                } else {
                    Err(ValidationError::new("json", &format!("The {} must be a valid JSON string.", field)))
                }
            },
            Value::Object(_) | Value::Array(_) => Ok(()), // Already valid JSON
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("json", &format!("The {} must be a valid JSON string.", field))),
        }
    }
}

pub struct IpRule;

#[async_trait]
impl Rule for IpRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                if s.parse::<std::net::IpAddr>().is_ok() {
                    Ok(())
                } else {
                    Err(ValidationError::new("ip", &format!("The {} must be a valid IP address.", field)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("ip", &format!("The {} must be a valid IP address.", field))),
        }
    }
}

pub struct RegexRule {
    pub pattern: String,
}

impl RegexRule {
    pub fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
        }
    }
}

#[async_trait]
impl Rule for RegexRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                match Regex::new(&self.pattern) {
                    Ok(regex) => {
                        if regex.is_match(s) {
                            Ok(())
                        } else {
                            Err(ValidationError::new("regex", &format!("The {} format is invalid.", field)))
                        }
                    },
                    Err(_) => Err(ValidationError::new("regex", &format!("Invalid regex pattern for {} validation.", field))),
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("regex", &format!("The {} format is invalid.", field))),
        }
    }
}

// File validation rules
pub struct FileRule;

#[async_trait]
impl Rule for FileRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                // Check if it's a valid file path or base64 encoded file
                if s.starts_with("data:") || std::path::Path::new(s).exists() {
                    Ok(())
                } else {
                    Err(ValidationError::new("file", &format!("The {} must be a file.", field)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("file", &format!("The {} must be a file.", field))),
        }
    }
}

pub struct ImageRule;

#[async_trait]
impl Rule for ImageRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                let image_types = ["jpeg", "jpg", "png", "gif", "bmp", "svg", "webp"];

                if s.starts_with("data:image/") {
                    Ok(())
                } else if let Some(extension) = std::path::Path::new(s).extension() {
                    if let Some(ext_str) = extension.to_str() {
                        if image_types.contains(&ext_str.to_lowercase().as_str()) {
                            Ok(())
                        } else {
                            Err(ValidationError::new("image", &format!("The {} must be an image.", field)))
                        }
                    } else {
                        Err(ValidationError::new("image", &format!("The {} must be an image.", field)))
                    }
                } else {
                    Err(ValidationError::new("image", &format!("The {} must be an image.", field)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("image", &format!("The {} must be an image.", field))),
        }
    }
}

pub struct MimesRule {
    pub types: Vec<String>,
}

impl MimesRule {
    pub fn new(types: Vec<String>) -> Self {
        Self { types }
    }
}

#[async_trait]
impl Rule for MimesRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                if s.starts_with("data:") {
                    // Extract MIME type from data URL
                    if let Some(mime_end) = s.find(';') {
                        let mime_type = &s[5..mime_end]; // Skip "data:"
                        if self.types.contains(&mime_type.to_string()) {
                            Ok(())
                        } else {
                            Err(ValidationError::new("mimes", &format!("The {} must be a file of type: {}.", field, self.types.join(", "))))
                        }
                    } else {
                        Err(ValidationError::new("mimes", &format!("The {} must be a valid file.", field)))
                    }
                } else if let Some(extension) = std::path::Path::new(s).extension() {
                    if let Some(ext_str) = extension.to_str() {
                        let mime_type = match ext_str.to_lowercase().as_str() {
                            "jpg" | "jpeg" => "image/jpeg",
                            "png" => "image/png",
                            "gif" => "image/gif",
                            "pdf" => "application/pdf",
                            "txt" => "text/plain",
                            "csv" => "text/csv",
                            "doc" => "application/msword",
                            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                            _ => ext_str,
                        };

                        if self.types.contains(&mime_type.to_string()) {
                            Ok(())
                        } else {
                            Err(ValidationError::new("mimes", &format!("The {} must be a file of type: {}.", field, self.types.join(", "))))
                        }
                    } else {
                        Err(ValidationError::new("mimes", &format!("The {} must be a valid file.", field)))
                    }
                } else {
                    Err(ValidationError::new("mimes", &format!("The {} must be a valid file.", field)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("mimes", &format!("The {} must be a file.", field))),
        }
    }
}

// Conditional validation rules
pub struct RequiredIfRule {
    pub other_field: String,
    pub value: String,
}

impl RequiredIfRule {
    pub fn new(other_field: &str, value: &str) -> Self {
        Self {
            other_field: other_field.to_string(),
            value: value.to_string(),
        }
    }
}

#[async_trait]
impl Rule for RequiredIfRule {
    async fn validate(&self, field: &str, value: &Value, data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        if let Some(other_value) = data.get(&self.other_field) {
            let other_str = match other_value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => String::new(),
            };

            if other_str == self.value {
                // The other field matches, so this field is required
                match value {
                    Value::Null => Err(ValidationError::new("required_if", &format!("The {} field is required when {} is {}.", field, self.other_field, self.value))),
                    Value::String(s) if s.trim().is_empty() => Err(ValidationError::new("required_if", &format!("The {} field is required when {} is {}.", field, self.other_field, self.value))),
                    Value::Array(arr) if arr.is_empty() => Err(ValidationError::new("required_if", &format!("The {} field is required when {} is {}.", field, self.other_field, self.value))),
                    _ => Ok(()),
                }
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

pub struct RequiredUnlessRule {
    pub other_field: String,
    pub value: String,
}

impl RequiredUnlessRule {
    pub fn new(other_field: &str, value: &str) -> Self {
        Self {
            other_field: other_field.to_string(),
            value: value.to_string(),
        }
    }
}

#[async_trait]
impl Rule for RequiredUnlessRule {
    async fn validate(&self, field: &str, value: &Value, data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        if let Some(other_value) = data.get(&self.other_field) {
            let other_str = match other_value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => String::new(),
            };

            if other_str != self.value {
                // The other field doesn't match, so this field is required
                match value {
                    Value::Null => Err(ValidationError::new("required_unless", &format!("The {} field is required unless {} is {}.", field, self.other_field, self.value))),
                    Value::String(s) if s.trim().is_empty() => Err(ValidationError::new("required_unless", &format!("The {} field is required unless {} is {}.", field, self.other_field, self.value))),
                    Value::Array(arr) if arr.is_empty() => Err(ValidationError::new("required_unless", &format!("The {} field is required unless {} is {}.", field, self.other_field, self.value))),
                    _ => Ok(()),
                }
            } else {
                Ok(())
            }
        } else {
            // Other field doesn't exist, so this field is required
            match value {
                Value::Null => Err(ValidationError::new("required_unless", &format!("The {} field is required unless {} is {}.", field, self.other_field, self.value))),
                Value::String(s) if s.trim().is_empty() => Err(ValidationError::new("required_unless", &format!("The {} field is required unless {} is {}.", field, self.other_field, self.value))),
                Value::Array(arr) if arr.is_empty() => Err(ValidationError::new("required_unless", &format!("The {} field is required unless {} is {}.", field, self.other_field, self.value))),
                _ => Ok(()),
            }
        }
    }
}

pub struct ConfirmedRule;

#[async_trait]
impl Rule for ConfirmedRule {
    async fn validate(&self, field: &str, value: &Value, data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        let confirmation_field = format!("{}_confirmation", field);

        if let Some(confirmation_value) = data.get(&confirmation_field) {
            if value == confirmation_value {
                Ok(())
            } else {
                Err(ValidationError::new("confirmed", &format!("The {} confirmation does not match.", field)))
            }
        } else {
            Err(ValidationError::new("confirmed", &format!("The {} confirmation field is required.", field)))
        }
    }
}

pub struct SameRule {
    pub other_field: String,
}

impl SameRule {
    pub fn new(other_field: &str) -> Self {
        Self {
            other_field: other_field.to_string(),
        }
    }
}

#[async_trait]
impl Rule for SameRule {
    async fn validate(&self, field: &str, value: &Value, data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        if let Some(other_value) = data.get(&self.other_field) {
            if value == other_value {
                Ok(())
            } else {
                Err(ValidationError::new("same", &format!("The {} and {} must match.", field, self.other_field)))
            }
        } else {
            Err(ValidationError::new("same", &format!("The {} field is required for comparison.", self.other_field)))
        }
    }
}

pub struct DifferentRule {
    pub other_field: String,
}

impl DifferentRule {
    pub fn new(other_field: &str) -> Self {
        Self {
            other_field: other_field.to_string(),
        }
    }
}

#[async_trait]
impl Rule for DifferentRule {
    async fn validate(&self, field: &str, value: &Value, data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        if let Some(other_value) = data.get(&self.other_field) {
            if value != other_value {
                Ok(())
            } else {
                Err(ValidationError::new("different", &format!("The {} and {} must be different.", field, self.other_field)))
            }
        } else {
            Ok(()) // If other field doesn't exist, they are different
        }
    }
}

// Additional database validation rules
pub struct ExistsRule {
    pub table: String,
    pub column: Option<String>,
}

impl ExistsRule {
    pub fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            column: None,
        }
    }

    pub fn with_column(table: &str, column: &str) -> Self {
        Self {
            table: table.to_string(),
            column: Some(column.to_string()),
        }
    }
}

#[async_trait]
impl Rule for ExistsRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, db: Option<&DbPool>) -> Result<(), ValidationError> {
        if let Some(db) = db {
            let default_field = field.to_string();
            let column = self.column.as_ref().unwrap_or(&default_field);
            let str_value = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Null => return Ok(()), // Allow null values
                _ => return Err(ValidationError::new("exists", &format!("The {} field must be a valid value for existence check.", field))),
            };

            let query = format!("SELECT COUNT(*) as count FROM {} WHERE {} = $1", self.table, column);

            match sqlx::query_scalar::<_, i64>(&query)
                .bind(&str_value)
                .fetch_one(db)
                .await
            {
                Ok(count) => {
                    if count > 0 {
                        Ok(())
                    } else {
                        Err(ValidationError::new("exists", &format!("The selected {} is invalid.", field)))
                    }
                },
                Err(_) => Err(ValidationError::new("exists", &format!("Unable to validate existence for {}.", field))),
            }
        } else {
            Err(ValidationError::new("exists", "Database connection required for exists validation."))
        }
    }
}

// Additional string rules
pub struct StartsWithRule {
    pub prefix: String,
}

impl StartsWithRule {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }
}

#[async_trait]
impl Rule for StartsWithRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                if s.starts_with(&self.prefix) {
                    Ok(())
                } else {
                    Err(ValidationError::new("starts_with", &format!("The {} must start with {}.", field, self.prefix)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("starts_with", &format!("The {} must be a string.", field))),
        }
    }
}

pub struct EndsWithRule {
    pub suffix: String,
}

impl EndsWithRule {
    pub fn new(suffix: &str) -> Self {
        Self {
            suffix: suffix.to_string(),
        }
    }
}

#[async_trait]
impl Rule for EndsWithRule {
    async fn validate(&self, field: &str, value: &Value, _data: &HashMap<String, Value>, _db: Option<&DbPool>) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                if s.ends_with(&self.suffix) {
                    Ok(())
                } else {
                    Err(ValidationError::new("ends_with", &format!("The {} must end with {}.", field, self.suffix)))
                }
            },
            Value::Null => Ok(()),
            _ => Err(ValidationError::new("ends_with", &format!("The {} must be a string.", field))),
        }
    }
}