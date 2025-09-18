use serde::Deserialize;
use std::collections::HashMap;

use super::{QueryBuilderRequest, PaginationType};

/// Query parameters structure for parsing URL query strings
#[derive(Debug, Deserialize)]
pub struct QueryParams {
    #[serde(flatten)]
    pub filter: HashMap<String, String>,

    #[serde(default)]
    pub sort: Vec<String>,

    #[serde(default)]
    pub fields: Option<String>,

    #[serde(default)]
    pub include: Option<String>,

    #[serde(default)]
    pub page: Option<u64>,

    #[serde(default)]
    pub per_page: Option<u64>,

    #[serde(default)]
    pub pagination_type: Option<PaginationType>,

    #[serde(default)]
    pub cursor: Option<String>,
}

impl Default for QueryParams {
    fn default() -> Self {
        Self {
            filter: HashMap::new(),
            sort: Vec::new(),
            fields: None,
            include: None,
            page: None,
            per_page: Some(15),
            pagination_type: None,
            cursor: None,
        }
    }
}

impl QueryParams {
    /// Parse query parameters into a QueryBuilderRequest
    pub fn parse(&self) -> QueryBuilderRequest {
        // Extract filters (remove reserved query params)
        let reserved_keys = vec!["sort", "fields", "include", "page", "per_page", "pagination_type", "cursor"];
        let filters: HashMap<String, String> = self.filter
            .iter()
            .filter(|(key, _)| !reserved_keys.contains(&key.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        // Parse fields
        let fields = self.fields.as_ref().map(|f| {
            f.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        });

        // Parse includes
        let includes = self.include.as_ref().map(|i| {
            i.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        });

        QueryBuilderRequest {
            filters,
            filter_groups: None, // Filter groups typically come from POST requests
            sorts: self.sort.clone(),
            fields,
            includes,
            page: self.page,
            per_page: self.per_page.or(Some(15)),
            pagination_type: self.pagination_type.clone().unwrap_or_default(),
            cursor: self.cursor.clone(),
        }
    }
}

// Note: QueryParams can be used with axum::extract::Query<QueryParams> directly
// The automatic FromRequestParts implementation is handled by Query extractor