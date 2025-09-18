use rustaxum::query_builder::{QueryBuilder, QueryBuilderRequest, Queryable, SortDirection};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, postgres::PgRow, PgPool};
use std::collections::HashMap;

/// Example model demonstrating QueryBuilder usage
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub author_id: i32,
    pub published: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Queryable for Post {
    fn table_name() -> &'static str {
        "posts"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "title",
            "content",
            "author_id",
            "published",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "title",
            "author_id",
            "published",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "title",
            "content",
            "author_id",
            "published",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 QueryBuilder Usage Examples");
    println!("===============================\n");

    // Note: In a real application, you would connect to an actual database
    // This example shows how to construct QueryBuilder requests

    // Example 1: Basic filtering
    println!("📋 Example 1: Basic Filtering");
    println!("URL: /posts?title=Rust&published=true");

    let mut filters = HashMap::new();
    filters.insert("title".to_string(), "Rust".to_string());
    filters.insert("published".to_string(), "true".to_string());

    let request = QueryBuilderRequest {
        filters,
        sorts: vec![],
        fields: None,
        page: Some(1),
        per_page: Some(10),
    };

    println!("QueryBuilder Request: {:#?}\n", request);

    // Example 2: Advanced filtering with operators
    println!("📋 Example 2: Advanced Filtering with Operators");
    println!("URL: /posts?title[like]=rust&author_id[in]=1,2,3&created_at[gte]=2024-01-01");

    let mut filters = HashMap::new();
    filters.insert("title[like]".to_string(), "rust".to_string());
    filters.insert("author_id[in]".to_string(), "1,2,3".to_string());
    filters.insert("created_at[gte]".to_string(), "2024-01-01".to_string());

    let request = QueryBuilderRequest {
        filters,
        sorts: vec![],
        fields: None,
        page: Some(1),
        per_page: Some(15),
    };

    println!("QueryBuilder Request: {:#?}\n", request);

    // Example 3: Sorting
    println!("📋 Example 3: Sorting");
    println!("URL: /posts?sort=-created_at,title");

    let request = QueryBuilderRequest {
        filters: HashMap::new(),
        sorts: vec!["-created_at".to_string(), "title".to_string()],
        fields: None,
        page: Some(1),
        per_page: Some(15),
    };

    println!("QueryBuilder Request: {:#?}\n", request);

    // Example 4: Field selection
    println!("📋 Example 4: Field Selection");
    println!("URL: /posts?fields=id,title,published,created_at");

    let request = QueryBuilderRequest {
        filters: HashMap::new(),
        sorts: vec![],
        fields: Some(vec![
            "id".to_string(),
            "title".to_string(),
            "published".to_string(),
            "created_at".to_string()
        ]),
        page: Some(1),
        per_page: Some(15),
    };

    println!("QueryBuilder Request: {:#?}\n", request);

    // Example 5: Complex combination
    println!("📋 Example 5: Complex Query");
    println!("URL: /posts?published=true&author_id[in]=1,2&title[like]=rust&sort=-created_at,title&fields=id,title,author_id,created_at&page=2&per_page=5");

    let mut filters = HashMap::new();
    filters.insert("published".to_string(), "true".to_string());
    filters.insert("author_id[in]".to_string(), "1,2".to_string());
    filters.insert("title[like]".to_string(), "rust".to_string());

    let request = QueryBuilderRequest {
        filters,
        sorts: vec!["-created_at".to_string(), "title".to_string()],
        fields: Some(vec![
            "id".to_string(),
            "title".to_string(),
            "author_id".to_string(),
            "created_at".to_string()
        ]),
        page: Some(2),
        per_page: Some(5),
    };

    println!("QueryBuilder Request: {:#?}\n", request);

    // Example of supported filter operators
    println!("🔧 Supported Filter Operators:");
    println!("==============================");
    println!("- eq (default): field=value");
    println!("- ne: field[ne]=value");
    println!("- gt: field[gt]=value");
    println!("- gte: field[gte]=value");
    println!("- lt: field[lt]=value");
    println!("- lte: field[lte]=value");
    println!("- like: field[like]=value (case-insensitive)");
    println!("- not_like: field[not_like]=value");
    println!("- in: field[in]=value1,value2,value3");
    println!("- not_in: field[not_in]=value1,value2");
    println!("- is_null: field[is_null]=true");
    println!("- is_not_null: field[is_not_null]=true\n");

    println!("🔧 Supported Sort Formats:");
    println!("===========================");
    println!("- field (ascending): sort=name");
    println!("- -field (descending): sort=-name");
    println!("- field:asc: sort=name:asc");
    println!("- field:desc: sort=name:desc");
    println!("- Multiple sorts: sort=name,-created_at\n");

    println!("✅ QueryBuilder examples completed!");

    Ok(())
}