use serde::Deserialize;
use std::collections::HashMap;

use super::QueryBuilderRequest;

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
    pub page: Option<u64>,

    #[serde(default)]
    pub per_page: Option<u64>,
}

impl Default for QueryParams {
    fn default() -> Self {
        Self {
            filter: HashMap::new(),
            sort: Vec::new(),
            fields: None,
            page: None,
            per_page: Some(15),
        }
    }
}

impl QueryParams {
    /// Parse query parameters into a QueryBuilderRequest
    pub fn parse(&self) -> QueryBuilderRequest {
        // Extract filters (remove reserved query params)
        let reserved_keys = vec!["sort", "fields", "page", "per_page"];
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

        QueryBuilderRequest {
            filters,
            sorts: self.sort.clone(),
            fields,
            page: self.page,
            per_page: self.per_page.or(Some(15)),
        }
    }
}

// Note: QueryParams can be used with axum::extract::Query<QueryParams> directly
// The automatic FromRequestParts implementation is handled by Query extractor