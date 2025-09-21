use sqlx::{PgPool, Row, Column};
use anyhow::Result;

use super::{
    Queryable, QueryBuilderRequest, PaginatedResponse, PaginationMeta,
    Filter, Sort, FieldSelector, Relatable, IncludeSelector, WithRelationships
};

/// Main QueryBuilder implementation
pub struct QueryBuilder<T>
where
    T: Queryable,
{
    pool: PgPool,
    request: QueryBuilderRequest,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> QueryBuilder<T>
where
    T: Queryable + Send + 'static,
{
    /// Create a new QueryBuilder instance
    pub fn new(pool: PgPool, request: QueryBuilderRequest) -> Self {
        Self {
            pool,
            request,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Execute the query and return paginated results
    pub async fn paginate(&self) -> Result<PaginatedResponse<T>> {
        let page = self.request.page.unwrap_or(1);
        let per_page = self.request.per_page.unwrap_or(15);

        // Build the base query
        let (select_clause, where_clause, order_clause, params) = self.build_query_parts()?;

        // Count total records
        let count_query = format!(
            "SELECT COUNT(*) as total FROM {} {}",
            T::table_name(),
            where_clause
        );

        let total: i64 = sqlx::query(&count_query)
            .bind_all(params.clone())
            .fetch_one(&self.pool)
            .await?
            .try_get("total")?;

        // Build paginated query
        let offset = (page - 1) * per_page;
        let query = format!(
            "SELECT {} FROM {} {} {} LIMIT $? OFFSET $?",
            select_clause,
            T::table_name(),
            where_clause,
            order_clause
        );

        let mut query_builder = sqlx::query_as::<_, T>(&query);

        // Bind parameters
        for param in params {
            query_builder = query_builder.bind(param);
        }

        // Add limit and offset
        query_builder = query_builder.bind(per_page as i64).bind(offset as i64);

        let data = query_builder.fetch_all(&self.pool).await?;

        let meta = PaginationMeta::new(page, per_page, total as u64);

        Ok(PaginatedResponse { data, meta })
    }

    /// Execute the query and return paginated results with relationships
    pub async fn paginate_with_relationships(&self) -> Result<PaginatedResponse<WithRelationships<T>>>
    where
        T: Relatable,
    {
        let page = self.request.page.unwrap_or(1);
        let per_page = self.request.per_page.unwrap_or(15);

        // Build the base query
        let (select_clause, where_clause, order_clause, params) = self.build_query_parts()?;

        // Count total records
        let count_query = format!(
            "SELECT COUNT(*) as total FROM {} {}",
            T::table_name(),
            where_clause
        );

        let total: i64 = sqlx::query(&count_query)
            .bind_all(params.clone())
            .fetch_one(&self.pool)
            .await?
            .try_get("total")?;

        // Build paginated query
        let offset = (page - 1) * per_page;
        let query = format!(
            "SELECT {} FROM {} {} {} LIMIT $? OFFSET $?",
            select_clause,
            T::table_name(),
            where_clause,
            order_clause
        );

        let mut query_builder = sqlx::query_as::<_, T>(&query);

        // Bind parameters
        for param in params {
            query_builder = query_builder.bind(param);
        }

        // Add limit and offset
        query_builder = query_builder.bind(per_page as i64).bind(offset as i64);

        let data = query_builder.fetch_all(&self.pool).await?;

        // Load relationships if requested
        let data_with_relationships = self.load_relationships(data).await?;

        let meta = PaginationMeta::new(page, per_page, total as u64);

        Ok(PaginatedResponse { data: data_with_relationships, meta })
    }

    /// Execute the query and return all results (without pagination)
    pub async fn get(&self) -> Result<Vec<T>> {
        let (select_clause, where_clause, order_clause, params) = self.build_query_parts()?;

        let query = format!(
            "SELECT {} FROM {} {} {}",
            select_clause,
            T::table_name(),
            where_clause,
            order_clause
        );

        let mut query_builder = sqlx::query_as::<_, T>(&query);

        // Bind parameters
        for param in params {
            query_builder = query_builder.bind(param);
        }

        Ok(query_builder.fetch_all(&self.pool).await?)
    }

    /// Build query parts (SELECT, WHERE, ORDER BY clauses and parameters)
    fn build_query_parts(&self) -> Result<(String, String, String, Vec<String>)> {
        // Build SELECT clause
        let select_clause = self.build_select_clause();

        // Build WHERE clause and collect parameters
        let (where_clause, params) = self.build_where_clause()?;

        // Build ORDER BY clause
        let order_clause = self.build_order_clause();

        Ok((select_clause, where_clause, order_clause, params))
    }

    /// Build SELECT clause
    fn build_select_clause(&self) -> String {
        if let Some(fields) = &self.request.fields {
            let allowed_fields: Vec<String> = T::allowed_fields().iter().map(|s| s.to_string()).collect();
            let selector = FieldSelector::new(fields.clone(), allowed_fields);
            selector.to_sql()
        } else {
            "*".to_string()
        }
    }

    /// Build WHERE clause and return parameters
    fn build_where_clause(&self) -> Result<(String, Vec<String>)> {
        let mut filter_clauses = Vec::new();
        let mut params = Vec::new();
        let mut param_index = 0;

        let allowed_filters: std::collections::HashSet<&str> = T::allowed_filters().into_iter().collect();

        // Handle simple filters
        for (key, value) in &self.request.filters {
            if let Some(filter) = Filter::from_query_param(key, value) {
                // Check if field is allowed
                if !allowed_filters.contains(filter.field.as_str()) {
                    continue;
                }

                let sql = filter.to_sql(&mut param_index);
                filter_clauses.push(sql);

                // Add parameter values
                params.extend(filter.get_sql_values());
            }
        }

        // Handle filter groups
        if let Some(filter_group) = &self.request.filter_groups {
            let (group_sql, mut group_params) = filter_group.to_sql(&mut param_index, &allowed_filters);
            if !group_sql.is_empty() {
                filter_clauses.push(group_sql);
                params.append(&mut group_params);
            }
        }

        let where_clause = if filter_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", filter_clauses.join(" AND "))
        };

        Ok((where_clause, params))
    }

    /// Build ORDER BY clause
    fn build_order_clause(&self) -> String {
        let mut sorts = Vec::new();
        let allowed_sorts: std::collections::HashSet<&str> = T::allowed_sorts().into_iter().collect();

        // Parse requested sorts
        for sort_str in &self.request.sorts {
            if let Some(sort) = Sort::from_str(sort_str) {
                if allowed_sorts.contains(sort.field.as_str()) {
                    sorts.push(sort);
                }
            }
        }

        // Add default sort if no valid sorts specified
        if sorts.is_empty() {
            if let Some((field, direction)) = T::default_sort() {
                sorts.push(Sort::new(field.to_string(), direction));
            }
        }

        if sorts.is_empty() {
            String::new()
        } else {
            let sort_clauses: Vec<String> = sorts.iter().map(|s| s.to_sql()).collect();
            format!("ORDER BY {}", sort_clauses.join(", "))
        }
    }

    /// Load relationships for the given data
    async fn load_relationships(&self, data: Vec<T>) -> Result<Vec<WithRelationships<T>>>
    where
        T: Relatable,
    {
        if data.is_empty() {
            return Ok(vec![]);
        }

        let mut result = Vec::new();

        // Convert includes to IncludeSelector if provided
        if let Some(includes) = &self.request.includes {
            let allowed_includes: Vec<String> = T::allowed_includes().iter().map(|s| s.to_string()).collect();
            let include_selector = IncludeSelector::new(includes.clone(), allowed_includes);
            let relationships = T::relationships();

            // Build eager load queries
            let eager_queries = include_selector.build_eager_load_queries(&data, &relationships);

            // Execute eager load queries and collect results
            let mut relationship_data = std::collections::HashMap::new();

            for query in eager_queries {
                let rows = sqlx::query(&query.sql)
                    .bind_all(query.parameters)
                    .fetch_all(&self.pool)
                    .await?;

                // Convert rows to JSON values for storage
                let mut json_rows = Vec::new();
                for row in rows {
                    let mut json_obj = serde_json::Map::new();

                    // Extract all columns from the row
                    for (i, column) in row.columns().iter().enumerate() {
                        let name = column.name();
                        match row.try_get::<String, _>(i) { Ok(value) => {
                            json_obj.insert(name.to_string(), serde_json::Value::String(value));
                        } _ => { match row.try_get::<i64, _>(i) { Ok(value) => {
                            json_obj.insert(name.to_string(), serde_json::Value::Number(serde_json::Number::from(value)));
                        } _ => { match row.try_get::<bool, _>(i) { Ok(value) => {
                            json_obj.insert(name.to_string(), serde_json::Value::Bool(value));
                        } _ => {}}}}}}
                        // Add more type conversions as needed
                    }
                    json_rows.push(serde_json::Value::Object(json_obj));
                }

                relationship_data.insert(query.relationship_name, json_rows);
            }

            // Build response with relationships
            for model in data {
                let mut with_rels = WithRelationships::new(model);

                // Add loaded relationships
                for (rel_name, rel_data) in &relationship_data {
                    if include_selector.should_include(rel_name) {
                        with_rels = with_rels.with_relationship(rel_name.clone(), serde_json::Value::Array(rel_data.clone()));
                    }
                }

                result.push(with_rels);
            }
        } else {
            // No includes requested, just wrap models
            for model in data {
                result.push(WithRelationships::new(model));
            }
        }

        Ok(result)
    }
}

/// Helper trait for binding all parameters to a query
trait BindAll {
    fn bind_all(self, params: Vec<String>) -> Self;
}

impl<'q> BindAll for sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    fn bind_all(mut self, params: Vec<String>) -> Self {
        for param in params {
            self = self.bind(param);
        }
        self
    }
}

impl<'q, T> BindAll for sqlx::query::QueryAs<'q, sqlx::Postgres, T, sqlx::postgres::PgArguments> {
    fn bind_all(mut self, params: Vec<String>) -> Self {
        for param in params {
            self = self.bind(param);
        }
        self
    }
}