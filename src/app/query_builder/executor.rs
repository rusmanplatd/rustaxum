use crate::app::query_builder::{Filter, Sort, QueryBuilder, Queryable, Pagination, PaginationResult};
use crate::database::DbConnection;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Text};
use anyhow::Result;

/// Query executor that builds and executes SQL queries using Diesel
pub struct QueryExecutor;

impl QueryExecutor {
    /// Execute a query builder and return paginated results
    pub fn execute_paginated<T>(
        builder: QueryBuilder<T>,
        conn: &mut DbConnection,
    ) -> Result<PaginationResult<serde_json::Value>>
    where
        T: Queryable + Clone,
    {
        let pagination = builder.get_pagination().cloned().unwrap_or_default();

        // Build the base query
        let mut query_parts = QueryParts::new(T::table_name());

        // Apply field selection
        if let Some(fields) = builder.get_fields() {
            query_parts.select_fields(fields);
        } else {
            query_parts.select_fields(&T::default_fields().iter().map(|s| s.to_string()).collect::<Vec<_>>());
        }

        // Apply filters
        for filter in builder.get_filters() {
            query_parts.add_filter(filter);
        }

        // Apply sorts
        let sorts = if builder.get_sorts().is_empty() {
            if let Some((field, direction)) = T::default_sort() {
                vec![Sort::new(field.to_string(), direction)]
            } else {
                vec![]
            }
        } else {
            builder.get_sorts().to_vec()
        };

        for sort in &sorts {
            query_parts.add_sort(sort);
        }

        // Execute count query for pagination
        let count_sql = query_parts.build_count_query();
        let total: i64 = sql_query(&count_sql)
            .bind::<Text, _>(&count_sql)
            .get_result::<CountResult>(conn)?
            .count;

        // Apply pagination
        query_parts.paginate(&pagination);

        // Execute main query
        let query_sql = query_parts.build_query();
        let results: Vec<QueryResult> = sql_query(&query_sql)
            .bind::<Text, _>(&query_sql)
            .load(conn)?;

        // Convert results to JSON
        let data: Vec<serde_json::Value> = results
            .into_iter()
            .map(|r| r.to_json())
            .collect();

        Ok(pagination.paginate(total as u64, data))
    }

    /// Execute a query builder and return all results (no pagination)
    pub fn execute_all<T>(
        builder: QueryBuilder<T>,
        conn: &mut DbConnection,
    ) -> Result<Vec<serde_json::Value>>
    where
        T: Queryable + Clone,
    {
        let mut query_parts = QueryParts::new(T::table_name());

        // Apply field selection
        if let Some(fields) = builder.get_fields() {
            query_parts.select_fields(fields);
        } else {
            query_parts.select_fields(&T::default_fields().iter().map(|s| s.to_string()).collect::<Vec<_>>());
        }

        // Apply filters
        for filter in builder.get_filters() {
            query_parts.add_filter(filter);
        }

        // Apply sorts
        let sorts = if builder.get_sorts().is_empty() {
            if let Some((field, direction)) = T::default_sort() {
                vec![Sort::new(field.to_string(), direction)]
            } else {
                vec![]
            }
        } else {
            builder.get_sorts().to_vec()
        };

        for sort in &sorts {
            query_parts.add_sort(sort);
        }

        // Execute query
        let query_sql = query_parts.build_query();
        let results: Vec<QueryResult> = sql_query(&query_sql)
            .bind::<Text, _>(&query_sql)
            .load(conn)?;

        Ok(results.into_iter().map(|r| r.to_json()).collect())
    }

    /// Execute a query builder and return the first result
    pub fn execute_first<T>(
        builder: QueryBuilder<T>,
        conn: &mut DbConnection,
    ) -> Result<Option<serde_json::Value>>
    where
        T: Queryable + Clone,
    {
        let limited_builder = builder.per_page(1);
        let results = Self::execute_all(limited_builder, conn)?;
        Ok(results.into_iter().next())
    }

    /// Execute a query builder and return the count of results
    pub fn execute_count<T>(
        builder: QueryBuilder<T>,
        conn: &mut DbConnection,
    ) -> Result<i64>
    where
        T: Queryable + Clone,
    {
        let mut query_parts = QueryParts::new(T::table_name());

        // Apply filters only (no sorting/pagination for count)
        for filter in builder.get_filters() {
            query_parts.add_filter(filter);
        }

        let count_sql = query_parts.build_count_query();
        let result: CountResult = sql_query(&count_sql)
            .bind::<Text, _>(&count_sql)
            .get_result(conn)?;

        Ok(result.count)
    }
}

/// Helper struct for building SQL query parts
#[derive(Debug)]
struct QueryParts {
    table: String,
    select_fields: Vec<String>,
    where_clauses: Vec<String>,
    order_clauses: Vec<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl QueryParts {
    fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            select_fields: vec!["*".to_string()],
            where_clauses: Vec::new(),
            order_clauses: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    fn select_fields(&mut self, fields: &[String]) {
        if !fields.is_empty() {
            self.select_fields = fields.to_vec();
        }
    }

    fn add_filter(&mut self, filter: &Filter) {
        let where_clause = self.build_filter_clause(filter);
        if !where_clause.is_empty() {
            self.where_clauses.push(where_clause);
        }
    }

    fn add_sort(&mut self, sort: &Sort) {
        let order_clause = format!("{} {}", sort.field, sort.direction.to_sql());
        self.order_clauses.push(order_clause);
    }

    fn paginate(&mut self, pagination: &Pagination) {
        self.limit = Some(pagination.limit());
        self.offset = Some(pagination.offset());
    }

    fn build_filter_clause(&self, filter: &Filter) -> String {
        use crate::app::query_builder::FilterOperator;

        match filter.operator {
            FilterOperator::Eq => {
                if let Some(value) = filter.value.as_single() {
                    format!("{} = {}", filter.field, self.escape_value(value))
                } else {
                    String::new()
                }
            }
            FilterOperator::Ne => {
                if let Some(value) = filter.value.as_single() {
                    format!("{} != {}", filter.field, self.escape_value(value))
                } else {
                    String::new()
                }
            }
            FilterOperator::Gt => {
                if let Some(value) = filter.value.as_single() {
                    format!("{} > {}", filter.field, self.escape_value(value))
                } else {
                    String::new()
                }
            }
            FilterOperator::Gte => {
                if let Some(value) = filter.value.as_single() {
                    format!("{} >= {}", filter.field, self.escape_value(value))
                } else {
                    String::new()
                }
            }
            FilterOperator::Lt => {
                if let Some(value) = filter.value.as_single() {
                    format!("{} < {}", filter.field, self.escape_value(value))
                } else {
                    String::new()
                }
            }
            FilterOperator::Lte => {
                if let Some(value) = filter.value.as_single() {
                    format!("{} <= {}", filter.field, self.escape_value(value))
                } else {
                    String::new()
                }
            }
            FilterOperator::Like => {
                if let Some(value) = filter.value.as_single() {
                    format!("{} LIKE {}", filter.field, self.escape_value(value))
                } else {
                    String::new()
                }
            }
            FilterOperator::Ilike => {
                if let Some(value) = filter.value.as_single() {
                    format!("{} ILIKE {}", filter.field, self.escape_value(value))
                } else {
                    String::new()
                }
            }
            FilterOperator::In => {
                if let Some(values) = filter.value.as_multiple() {
                    let escaped_values: Vec<String> = values.iter()
                        .map(|v| self.escape_value(v))
                        .collect();
                    format!("{} IN ({})", filter.field, escaped_values.join(", "))
                } else {
                    String::new()
                }
            }
            FilterOperator::NotIn => {
                if let Some(values) = filter.value.as_multiple() {
                    let escaped_values: Vec<String> = values.iter()
                        .map(|v| self.escape_value(v))
                        .collect();
                    format!("{} NOT IN ({})", filter.field, escaped_values.join(", "))
                } else {
                    String::new()
                }
            }
            FilterOperator::IsNull => {
                format!("{} IS NULL", filter.field)
            }
            FilterOperator::IsNotNull => {
                format!("{} IS NOT NULL", filter.field)
            }
            FilterOperator::Between => {
                if let Some((start, end)) = filter.value.as_range() {
                    format!("{} BETWEEN {} AND {}",
                           filter.field,
                           self.escape_value(start),
                           self.escape_value(end))
                } else {
                    String::new()
                }
            }
            _ => {
                // Other operators can be implemented as needed
                String::new()
            }
        }
    }

    fn escape_value(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "NULL".to_string(),
            _ => format!("'{}'", value.to_string().replace("'", "''")),
        }
    }

    fn build_query(&self) -> String {
        let mut query = format!(
            "SELECT {} FROM {}",
            self.select_fields.join(", "),
            self.table
        );

        if !self.where_clauses.is_empty() {
            query.push_str(&format!(" WHERE {}", self.where_clauses.join(" AND ")));
        }

        if !self.order_clauses.is_empty() {
            query.push_str(&format!(" ORDER BY {}", self.order_clauses.join(", ")));
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        query
    }

    fn build_count_query(&self) -> String {
        let mut query = format!("SELECT COUNT(*) as count FROM {}", self.table);

        if !self.where_clauses.is_empty() {
            query.push_str(&format!(" WHERE {}", self.where_clauses.join(" AND ")));
        }

        query
    }
}

/// Result struct for raw SQL queries returning JSON data
#[derive(QueryableByName, Debug)]
struct QueryResult {
    #[diesel(sql_type = Text)]
    data: String, // Store as string first, then parse to JSON
}

impl QueryResult {
    fn to_json(self) -> serde_json::Value {
        serde_json::from_str(&self.data).unwrap_or(serde_json::Value::String(self.data))
    }
}

/// Result struct for count queries
#[derive(QueryableByName)]
struct CountResult {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_parts_build_query() {
        let mut parts = QueryParts::new("users");
        parts.select_fields(&vec!["id".to_string(), "name".to_string()]);
        parts.where_clauses.push("name = 'John'".to_string());
        parts.order_clauses.push("created_at DESC".to_string());
        parts.limit = Some(10);
        parts.offset = Some(5);

        let query = parts.build_query();
        assert!(query.contains("SELECT id, name FROM users"));
        assert!(query.contains("WHERE name = 'John'"));
        assert!(query.contains("ORDER BY created_at DESC"));
        assert!(query.contains("LIMIT 10"));
        assert!(query.contains("OFFSET 5"));
    }

    #[test]
    fn test_query_parts_build_count_query() {
        let mut parts = QueryParts::new("users");
        parts.where_clauses.push("active = true".to_string());

        let query = parts.build_count_query();
        assert_eq!(query, "SELECT COUNT(*) as count FROM users WHERE active = true");
    }
}