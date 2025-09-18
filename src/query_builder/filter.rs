use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported filter operators
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOperator {
    Eq,      // =
    Ne,      // !=
    Gt,      // >
    Gte,     // >=
    Lt,      // <
    Lte,     // <=
    Like,    // ILIKE
    NotLike, // NOT ILIKE
    In,      // IN
    NotIn,   // NOT IN
    IsNull,  // IS NULL
    IsNotNull, // IS NOT NULL
}

impl FilterOperator {
    /// Parse operator from string (e.g., "eq", "gt", "like")
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "eq" | "=" => Some(Self::Eq),
            "ne" | "!=" | "<>" => Some(Self::Ne),
            "gt" | ">" => Some(Self::Gt),
            "gte" | ">=" => Some(Self::Gte),
            "lt" | "<" => Some(Self::Lt),
            "lte" | "<=" => Some(Self::Lte),
            "like" => Some(Self::Like),
            "not_like" | "notlike" => Some(Self::NotLike),
            "in" => Some(Self::In),
            "not_in" | "notin" => Some(Self::NotIn),
            "is_null" | "isnull" => Some(Self::IsNull),
            "is_not_null" | "isnotnull" => Some(Self::IsNotNull),
            _ => None,
        }
    }

    /// Convert to SQL operator string
    pub fn to_sql(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Ne => "!=",
            Self::Gt => ">",
            Self::Gte => ">=",
            Self::Lt => "<",
            Self::Lte => "<=",
            Self::Like => "ILIKE",
            Self::NotLike => "NOT ILIKE",
            Self::In => "IN",
            Self::NotIn => "NOT IN",
            Self::IsNull => "IS NULL",
            Self::IsNotNull => "IS NOT NULL",
        }
    }

    /// Check if operator needs a value
    pub fn needs_value(&self) -> bool {
        !matches!(self, Self::IsNull | Self::IsNotNull)
    }
}

impl fmt::Display for FilterOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_sql())
    }
}

/// Filter structure
#[derive(Debug, Clone)]
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: Option<String>,
}

impl Filter {
    /// Create a new filter
    pub fn new(field: String, operator: FilterOperator, value: Option<String>) -> Self {
        Self { field, operator, value }
    }

    /// Parse filter from query parameter
    /// Supports formats:
    /// - field=value (defaults to Eq)
    /// - field[eq]=value
    /// - field[gt]=value
    /// - field[like]=value
    pub fn from_query_param(key: &str, value: &str) -> Option<Self> {
        if let Some(captures) = regex::Regex::new(r"^([^[\]]+)(?:\[([^[\]]+)\])?$")
            .unwrap()
            .captures(key)
        {
            let field = captures.get(1)?.as_str().to_string();
            let operator_str = captures.get(2).map(|m| m.as_str()).unwrap_or("eq");

            if let Some(operator) = FilterOperator::from_str(operator_str) {
                let filter_value = if operator.needs_value() {
                    Some(value.to_string())
                } else {
                    None
                };

                return Some(Filter::new(field, operator, filter_value));
            }
        }

        None
    }

    /// Generate SQL WHERE clause for this filter
    pub fn to_sql(&self, param_index: &mut usize) -> String {
        match &self.operator {
            FilterOperator::IsNull | FilterOperator::IsNotNull => {
                format!("{} {}", self.field, self.operator.to_sql())
            }
            FilterOperator::In | FilterOperator::NotIn => {
                if let Some(value) = &self.value {
                    let values: Vec<&str> = value.split(',').collect();
                    let placeholders: Vec<String> = values
                        .iter()
                        .map(|_| {
                            *param_index += 1;
                            format!("${}", param_index)
                        })
                        .collect();
                    format!("{} {} ({})", self.field, self.operator.to_sql(), placeholders.join(", "))
                } else {
                    format!("{} {} ()", self.field, self.operator.to_sql())
                }
            }
            _ => {
                *param_index += 1;
                format!("{} {} ${}", self.field, self.operator.to_sql(), param_index)
            }
        }
    }

    /// Get values for SQL parameters
    pub fn get_sql_values(&self) -> Vec<String> {
        match &self.operator {
            FilterOperator::IsNull | FilterOperator::IsNotNull => vec![],
            FilterOperator::In | FilterOperator::NotIn => {
                if let Some(value) = &self.value {
                    value.split(',').map(|s| s.trim().to_string()).collect()
                } else {
                    vec![]
                }
            }
            _ => {
                if let Some(value) = &self.value {
                    vec![value.clone()]
                } else {
                    vec![]
                }
            }
        }
    }
}