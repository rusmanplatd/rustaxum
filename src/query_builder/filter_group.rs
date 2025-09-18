use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::query_builder::{Filter, FilterOperator};

/// Advanced filtering support with nested AND/OR logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterGroup {
    And(Vec<FilterCondition>),
    Or(Vec<FilterCondition>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterCondition {
    Simple(SimpleFilter),
    Group(FilterGroup),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: Option<String>,
}

impl SimpleFilter {
    pub fn new(field: String, operator: FilterOperator, value: Option<String>) -> Self {
        Self { field, operator, value }
    }

    pub fn to_filter(&self) -> Filter {
        Filter::new(self.field.clone(), self.operator.clone(), self.value.clone())
    }
}

impl FilterGroup {
    /// Create an AND group
    pub fn and(conditions: Vec<FilterCondition>) -> Self {
        Self::And(conditions)
    }

    /// Create an OR group
    pub fn or(conditions: Vec<FilterCondition>) -> Self {
        Self::Or(conditions)
    }

    /// Convert to SQL WHERE clause with parameter binding
    pub fn to_sql(&self, param_index: &mut usize, allowed_filters: &std::collections::HashSet<&str>) -> (String, Vec<String>) {
        self.to_sql_internal(param_index, allowed_filters)
    }

    fn to_sql_internal(&self, param_index: &mut usize, allowed_filters: &std::collections::HashSet<&str>) -> (String, Vec<String>) {
        let mut params = Vec::new();

        let sql = match self {
            FilterGroup::And(conditions) => {
                let mut sql_parts = Vec::new();

                for condition in conditions {
                    let (condition_sql, mut condition_params) = condition.to_sql_internal(param_index, allowed_filters);
                    if !condition_sql.is_empty() {
                        sql_parts.push(condition_sql);
                        params.append(&mut condition_params);
                    }
                }

                if sql_parts.is_empty() {
                    String::new()
                } else if sql_parts.len() == 1 {
                    sql_parts[0].clone()
                } else {
                    format!("({})", sql_parts.join(" AND "))
                }
            },
            FilterGroup::Or(conditions) => {
                let mut sql_parts = Vec::new();

                for condition in conditions {
                    let (condition_sql, mut condition_params) = condition.to_sql_internal(param_index, allowed_filters);
                    if !condition_sql.is_empty() {
                        sql_parts.push(condition_sql);
                        params.append(&mut condition_params);
                    }
                }

                if sql_parts.is_empty() {
                    String::new()
                } else if sql_parts.len() == 1 {
                    sql_parts[0].clone()
                } else {
                    format!("({})", sql_parts.join(" OR "))
                }
            },
        };

        (sql, params)
    }

    /// Parse from JSON string (for advanced filtering via POST requests)
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Parse from query parameters with nested syntax
    /// Example: filter[and][0][field]=name&filter[and][0][op]=eq&filter[and][0][value]=John
    pub fn from_query_params(params: &HashMap<String, String>) -> Option<Self> {
        // Look for filter[and] or filter[or] patterns
        let mut and_conditions = Vec::new();
        let mut or_conditions = Vec::new();

        // Parse AND conditions
        let mut and_index = 0;
        loop {
            let field_key = format!("filter[and][{}][field]", and_index);
            let op_key = format!("filter[and][{}][op]", and_index);
            let value_key = format!("filter[and][{}][value]", and_index);

            if let (Some(field), Some(op_str)) = (params.get(&field_key), params.get(&op_key)) {
                if let Some(operator) = FilterOperator::from_str(op_str) {
                    let value = if operator.needs_value() {
                        params.get(&value_key).cloned()
                    } else {
                        None
                    };

                    let simple_filter = SimpleFilter::new(field.clone(), operator, value);
                    and_conditions.push(FilterCondition::Simple(simple_filter));
                }
                and_index += 1;
            } else {
                break;
            }
        }

        // Parse OR conditions
        let mut or_index = 0;
        loop {
            let field_key = format!("filter[or][{}][field]", or_index);
            let op_key = format!("filter[or][{}][op]", or_index);
            let value_key = format!("filter[or][{}][value]", or_index);

            if let (Some(field), Some(op_str)) = (params.get(&field_key), params.get(&op_key)) {
                if let Some(operator) = FilterOperator::from_str(op_str) {
                    let value = if operator.needs_value() {
                        params.get(&value_key).cloned()
                    } else {
                        None
                    };

                    let simple_filter = SimpleFilter::new(field.clone(), operator, value);
                    or_conditions.push(FilterCondition::Simple(simple_filter));
                }
                or_index += 1;
            } else {
                break;
            }
        }

        // Return the appropriate group
        if !and_conditions.is_empty() && or_conditions.is_empty() {
            Some(FilterGroup::And(and_conditions))
        } else if and_conditions.is_empty() && !or_conditions.is_empty() {
            Some(FilterGroup::Or(or_conditions))
        } else if !and_conditions.is_empty() && !or_conditions.is_empty() {
            // If both exist, wrap them in an AND group
            Some(FilterGroup::And(vec![
                FilterCondition::Group(FilterGroup::And(and_conditions)),
                FilterCondition::Group(FilterGroup::Or(or_conditions)),
            ]))
        } else {
            None
        }
    }
}

impl FilterCondition {
    pub fn simple(field: String, operator: FilterOperator, value: Option<String>) -> Self {
        Self::Simple(SimpleFilter::new(field, operator, value))
    }

    pub fn group(group: FilterGroup) -> Self {
        Self::Group(group)
    }

    fn to_sql_internal(&self, param_index: &mut usize, allowed_filters: &std::collections::HashSet<&str>) -> (String, Vec<String>) {
        match self {
            FilterCondition::Simple(filter) => {
                // Check if field is allowed
                if !allowed_filters.contains(filter.field.as_str()) {
                    return (String::new(), vec![]);
                }

                let db_filter = filter.to_filter();
                let sql = db_filter.to_sql(param_index);
                let params = db_filter.get_sql_values();
                (sql, params)
            },
            FilterCondition::Group(group) => {
                group.to_sql_internal(param_index, allowed_filters)
            },
        }
    }
}

/// Builder for creating complex filter groups programmatically
pub struct FilterGroupBuilder {
    conditions: Vec<FilterCondition>,
}

impl FilterGroupBuilder {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    pub fn add_condition(mut self, field: &str, operator: FilterOperator, value: Option<String>) -> Self {
        let filter = SimpleFilter::new(field.to_string(), operator, value);
        self.conditions.push(FilterCondition::Simple(filter));
        self
    }

    pub fn add_group(mut self, group: FilterGroup) -> Self {
        self.conditions.push(FilterCondition::Group(group));
        self
    }

    pub fn build_and(self) -> FilterGroup {
        FilterGroup::And(self.conditions)
    }

    pub fn build_or(self) -> FilterGroup {
        FilterGroup::Or(self.conditions)
    }
}

impl Default for FilterGroupBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_simple_and_group() {
        let group = FilterGroup::And(vec![
            FilterCondition::simple("name".to_string(), FilterOperator::Eq, Some("John".to_string())),
            FilterCondition::simple("age".to_string(), FilterOperator::Gt, Some("25".to_string())),
        ]);

        let mut allowed_filters = HashSet::new();
        allowed_filters.insert("name");
        allowed_filters.insert("age");

        let mut param_index = 0;
        let (sql, params) = group.to_sql(&mut param_index, &allowed_filters);

        assert!(sql.contains("name = $1"));
        assert!(sql.contains("age > $2"));
        assert!(sql.contains("AND"));
        assert_eq!(params, vec!["John".to_string(), "25".to_string()]);
    }

    #[test]
    fn test_simple_or_group() {
        let group = FilterGroup::Or(vec![
            FilterCondition::simple("status".to_string(), FilterOperator::Eq, Some("active".to_string())),
            FilterCondition::simple("status".to_string(), FilterOperator::Eq, Some("pending".to_string())),
        ]);

        let mut allowed_filters = HashSet::new();
        allowed_filters.insert("status");

        let mut param_index = 0;
        let (sql, params) = group.to_sql(&mut param_index, &allowed_filters);

        assert!(sql.contains("status = $1"));
        assert!(sql.contains("status = $2"));
        assert!(sql.contains("OR"));
        assert_eq!(params, vec!["active".to_string(), "pending".to_string()]);
    }

    #[test]
    fn test_builder() {
        let group = FilterGroupBuilder::new()
            .add_condition("name", FilterOperator::Like, Some("%john%".to_string()))
            .add_condition("active", FilterOperator::Eq, Some("true".to_string()))
            .build_and();

        let mut allowed_filters = HashSet::new();
        allowed_filters.insert("name");
        allowed_filters.insert("active");

        let mut param_index = 0;
        let (sql, _params) = group.to_sql(&mut param_index, &allowed_filters);

        assert!(sql.contains("name ILIKE"));
        assert!(sql.contains("active ="));
        assert!(sql.contains("AND"));
    }
}