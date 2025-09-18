use sqlx::{PgPool, Row};
use anyhow::Result;

use super::{
    Queryable, QueryBuilderRequest, PaginatedResponse, PaginationMeta,
    Filter, Sort, FieldSelector
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
        let mut filters = Vec::new();
        let mut params = Vec::new();
        let mut param_index = 0;

        let allowed_filters: std::collections::HashSet<&str> = T::allowed_filters().into_iter().collect();

        for (key, value) in &self.request.filters {
            if let Some(filter) = Filter::from_query_param(key, value) {
                // Check if field is allowed
                if !allowed_filters.contains(filter.field.as_str()) {
                    continue;
                }

                let sql = filter.to_sql(&mut param_index);
                filters.push(sql);

                // Add parameter values
                params.extend(filter.get_sql_values());
            }
        }

        let where_clause = if filters.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", filters.join(" AND "))
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