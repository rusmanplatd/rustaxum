use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Filter operators for query conditions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilterOperator {
    /// Equals (=)
    Eq,
    /// Not equals (!=)
    Ne,
    /// Greater than (>)
    Gt,
    /// Greater than or equal (>=)
    Gte,
    /// Less than (<)
    Lt,
    /// Less than or equal (<=)
    Lte,
    /// LIKE pattern matching
    Like,
    /// ILIKE case-insensitive pattern matching
    Ilike,
    /// NOT LIKE
    NotLike,
    /// NOT ILIKE
    NotIlike,
    /// IN (value1, value2, ...)
    In,
    /// NOT IN (value1, value2, ...)
    NotIn,
    /// IS NULL
    IsNull,
    /// IS NOT NULL
    IsNotNull,
    /// BETWEEN value1 AND value2
    Between,
    /// NOT BETWEEN value1 AND value2
    NotBetween,
    /// JSON contains (@>)
    JsonContains,
    /// JSON contained by (<@)
    JsonContainedBy,
    /// JSON has key (?)
    JsonHasKey,
    /// JSON has any key (?|)
    JsonHasAnyKey,
    /// JSON has all keys (?&)
    JsonHasAllKeys,
    /// Full text search
    FullText,
}

impl FilterOperator {
    /// Get SQL operator string
    pub fn to_sql(&self) -> &'static str {
        match self {
            FilterOperator::Eq => "=",
            FilterOperator::Ne => "!=",
            FilterOperator::Gt => ">",
            FilterOperator::Gte => ">=",
            FilterOperator::Lt => "<",
            FilterOperator::Lte => "<=",
            FilterOperator::Like => "LIKE",
            FilterOperator::Ilike => "ILIKE",
            FilterOperator::NotLike => "NOT LIKE",
            FilterOperator::NotIlike => "NOT ILIKE",
            FilterOperator::In => "IN",
            FilterOperator::NotIn => "NOT IN",
            FilterOperator::IsNull => "IS NULL",
            FilterOperator::IsNotNull => "IS NOT NULL",
            FilterOperator::Between => "BETWEEN",
            FilterOperator::NotBetween => "NOT BETWEEN",
            FilterOperator::JsonContains => "@>",
            FilterOperator::JsonContainedBy => "<@",
            FilterOperator::JsonHasKey => "?",
            FilterOperator::JsonHasAnyKey => "?|",
            FilterOperator::JsonHasAllKeys => "?&",
            FilterOperator::FullText => "@@",
        }
    }

    /// Parse operator from string
    pub fn from_string(op: &str) -> Option<Self> {
        match op.to_lowercase().as_str() {
            "eq" | "=" => Some(FilterOperator::Eq),
            "ne" | "!=" | "neq" => Some(FilterOperator::Ne),
            "gt" | ">" => Some(FilterOperator::Gt),
            "gte" | ">=" => Some(FilterOperator::Gte),
            "lt" | "<" => Some(FilterOperator::Lt),
            "lte" | "<=" => Some(FilterOperator::Lte),
            "like" => Some(FilterOperator::Like),
            "ilike" => Some(FilterOperator::Ilike),
            "not_like" | "notlike" => Some(FilterOperator::NotLike),
            "not_ilike" | "notilike" => Some(FilterOperator::NotIlike),
            "in" => Some(FilterOperator::In),
            "not_in" | "notin" => Some(FilterOperator::NotIn),
            "is_null" | "isnull" | "null" => Some(FilterOperator::IsNull),
            "is_not_null" | "isnotnull" | "not_null" | "notnull" => Some(FilterOperator::IsNotNull),
            "between" => Some(FilterOperator::Between),
            "not_between" | "notbetween" => Some(FilterOperator::NotBetween),
            "json_contains" | "jsoncontains" => Some(FilterOperator::JsonContains),
            "json_contained_by" | "jsoncontainedby" => Some(FilterOperator::JsonContainedBy),
            "json_has_key" | "jsonhaskey" => Some(FilterOperator::JsonHasKey),
            "json_has_any_key" | "jsonhasanykey" => Some(FilterOperator::JsonHasAnyKey),
            "json_has_all_keys" | "jsonhasallkeys" => Some(FilterOperator::JsonHasAllKeys),
            "full_text" | "fulltext" => Some(FilterOperator::FullText),
            _ => None,
        }
    }
}

/// Filter value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
    /// Single value
    Single(Value),
    /// Multiple values (for IN, NOT IN operations)
    Multiple(Vec<Value>),
    /// Range values (for BETWEEN operations)
    Range(Value, Value),
}

impl FilterValue {
    /// Create a single value filter
    pub fn single(value: impl Into<Value>) -> Self {
        FilterValue::Single(value.into())
    }

    /// Create a multiple values filter
    pub fn multiple(values: Vec<impl Into<Value>>) -> Self {
        FilterValue::Multiple(values.into_iter().map(|v| v.into()).collect())
    }

    /// Create a range filter
    pub fn range(start: impl Into<Value>, end: impl Into<Value>) -> Self {
        FilterValue::Range(start.into(), end.into())
    }

    /// Get single value
    pub fn as_single(&self) -> Option<&Value> {
        match self {
            FilterValue::Single(value) => Some(value),
            _ => None,
        }
    }

    /// Get multiple values
    pub fn as_multiple(&self) -> Option<&Vec<Value>> {
        match self {
            FilterValue::Multiple(values) => Some(values),
            _ => None,
        }
    }

    /// Get range values
    pub fn as_range(&self) -> Option<(&Value, &Value)> {
        match self {
            FilterValue::Range(start, end) => Some((start, end)),
            _ => None,
        }
    }
}

/// Filter specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
    /// Field name to filter on
    pub field: String,
    /// Filter operator
    pub operator: FilterOperator,
    /// Filter value(s)
    pub value: FilterValue,
    /// Whether this filter should be combined with AND (true) or OR (false)
    #[serde(default = "default_and")]
    pub and: bool,
}

fn default_and() -> bool {
    true
}

impl Filter {
    /// Create a new filter
    pub fn new(field: impl Into<String>, operator: FilterOperator, value: FilterValue) -> Self {
        Self {
            field: field.into(),
            operator,
            value,
            and: true,
        }
    }

    /// Create an equals filter
    pub fn eq(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::new(field, FilterOperator::Eq, FilterValue::single(value))
    }

    /// Create a not equals filter
    pub fn ne(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::new(field, FilterOperator::Ne, FilterValue::single(value))
    }

    /// Create a greater than filter
    pub fn gt(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::new(field, FilterOperator::Gt, FilterValue::single(value))
    }

    /// Create a greater than or equal filter
    pub fn gte(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::new(field, FilterOperator::Gte, FilterValue::single(value))
    }

    /// Create a less than filter
    pub fn lt(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::new(field, FilterOperator::Lt, FilterValue::single(value))
    }

    /// Create a less than or equal filter
    pub fn lte(field: impl Into<String>, value: impl Into<Value>) -> Self {
        Self::new(field, FilterOperator::Lte, FilterValue::single(value))
    }

    /// Create a LIKE filter
    pub fn like(field: impl Into<String>, pattern: impl Into<Value>) -> Self {
        Self::new(field, FilterOperator::Like, FilterValue::single(pattern))
    }

    /// Create an ILIKE filter
    pub fn ilike(field: impl Into<String>, pattern: impl Into<Value>) -> Self {
        Self::new(field, FilterOperator::Ilike, FilterValue::single(pattern))
    }

    /// Create an IN filter
    pub fn in_values(field: impl Into<String>, values: Vec<impl Into<Value>>) -> Self {
        Self::new(field, FilterOperator::In, FilterValue::multiple(values))
    }

    /// Create a NOT IN filter
    pub fn not_in(field: impl Into<String>, values: Vec<impl Into<Value>>) -> Self {
        Self::new(field, FilterOperator::NotIn, FilterValue::multiple(values))
    }

    /// Create an IS NULL filter
    pub fn is_null(field: impl Into<String>) -> Self {
        Self::new(field, FilterOperator::IsNull, FilterValue::single(Value::Null))
    }

    /// Create an IS NOT NULL filter
    pub fn is_not_null(field: impl Into<String>) -> Self {
        Self::new(field, FilterOperator::IsNotNull, FilterValue::single(Value::Null))
    }

    /// Create a BETWEEN filter
    pub fn between(field: impl Into<String>, start: impl Into<Value>, end: impl Into<Value>) -> Self {
        Self::new(field, FilterOperator::Between, FilterValue::range(start, end))
    }

    /// Set this filter to use OR instead of AND
    pub fn or(mut self) -> Self {
        self.and = false;
        self
    }

    /// Parse filters from query parameters
    /// Supports Laravel-style nested filter syntax: filter[field][operator]=value
    pub fn from_params(params: &HashMap<String, Value>) -> Vec<Filter> {
        let mut filters = Vec::new();

        for (key, value) in params {
            if let Some(filter) = Self::parse_filter_param(key, value) {
                filters.push(filter);
            }
        }

        filters
    }

    /// Parse a single filter parameter
    fn parse_filter_param(key: &str, value: &Value) -> Option<Filter> {
        // Handle simple format: filter[field]=value (defaults to eq)
        if !key.contains("][") && !key.contains("[") {
            return Some(Filter::eq(key, value.clone()));
        }

        // Handle nested format: filter[field][operator]=value
        if let Some(captures) = regex::Regex::new(r"^(.+?)\[(.+?)\]$")
            .ok()?
            .captures(key)
        {
            let field = captures.get(1)?.as_str();
            let operator_str = captures.get(2)?.as_str();

            if let Some(operator) = FilterOperator::from_string(operator_str) {
                let filter_value = match operator {
                    FilterOperator::In | FilterOperator::NotIn => {
                        if let Value::Array(arr) = value {
                            FilterValue::Multiple(arr.clone())
                        } else if let Value::String(s) = value {
                            // Handle comma-separated values
                            let values: Vec<Value> = s
                                .split(',')
                                .map(|v| Value::String(v.trim().to_string()))
                                .collect();
                            FilterValue::Multiple(values)
                        } else {
                            FilterValue::single(value.clone())
                        }
                    }
                    FilterOperator::Between | FilterOperator::NotBetween => {
                        if let Value::Array(arr) = value {
                            if arr.len() >= 2 {
                                FilterValue::Range(arr[0].clone(), arr[1].clone())
                            } else {
                                FilterValue::single(value.clone())
                            }
                        } else {
                            FilterValue::single(value.clone())
                        }
                    }
                    _ => FilterValue::single(value.clone()),
                };

                return Some(Filter::new(field, operator, filter_value));
            }
        }

        // Fallback: treat as simple equals
        Some(Filter::eq(key, value.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_filter_operator_from_string() {
        assert_eq!(FilterOperator::from_string("eq"), Some(FilterOperator::Eq));
        assert_eq!(FilterOperator::from_string("="), Some(FilterOperator::Eq));
        assert_eq!(FilterOperator::from_string("gt"), Some(FilterOperator::Gt));
        assert_eq!(FilterOperator::from_string("like"), Some(FilterOperator::Like));
        assert_eq!(FilterOperator::from_string("invalid"), None);
    }

    #[test]
    fn test_filter_creation() {
        let filter = Filter::eq("name", "John");
        assert_eq!(filter.field, "name");
        assert_eq!(filter.operator, FilterOperator::Eq);
        assert_eq!(filter.value.as_single(), Some(&json!("John")));

        let filter = Filter::in_values("status", vec!["active", "pending"]);
        assert_eq!(filter.field, "status");
        assert_eq!(filter.operator, FilterOperator::In);
        assert!(filter.value.as_multiple().is_some());
    }

    #[test]
    fn test_filter_from_params() {
        let mut params = HashMap::new();
        params.insert("name".to_string(), json!("John"));
        params.insert("age[gte]".to_string(), json!(18));
        params.insert("status[in]".to_string(), json!(["active", "pending"]));

        let filters = Filter::from_params(&params);
        assert_eq!(filters.len(), 3);

        // Find the name filter
        let name_filter = filters.iter().find(|f| f.field == "name").unwrap();
        assert_eq!(name_filter.operator, FilterOperator::Eq);
        assert_eq!(name_filter.value.as_single(), Some(&json!("John")));

        // Find the age filter
        let age_filter = filters.iter().find(|f| f.field == "age").unwrap();
        assert_eq!(age_filter.operator, FilterOperator::Gte);
        assert_eq!(age_filter.value.as_single(), Some(&json!(18)));
    }
}