use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Include specification for eager loading relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Include {
    /// Relationship name to include
    pub relation: String,
    /// Nested includes for the relationship
    pub nested: Vec<Include>,
    /// Fields to select from the included relationship
    pub fields: Option<Vec<String>>,
    /// Filters to apply to the included relationship
    pub filters: Option<HashMap<String, serde_json::Value>>,
    /// Sort to apply to the included relationship
    pub sort: Option<String>,
}

impl Include {
    /// Create a new include
    pub fn new(relation: impl Into<String>) -> Self {
        Self {
            relation: relation.into(),
            nested: Vec::new(),
            fields: None,
            filters: None,
            sort: None,
        }
    }

    /// Add nested include
    pub fn with_nested(mut self, nested: Include) -> Self {
        self.nested.push(nested);
        self
    }

    /// Set fields to select
    pub fn with_fields(mut self, fields: Vec<impl Into<String>>) -> Self {
        self.fields = Some(fields.into_iter().map(|f| f.into()).collect());
        self
    }

    /// Set filters for the relationship
    pub fn with_filters(mut self, filters: HashMap<String, serde_json::Value>) -> Self {
        self.filters = Some(filters);
        self
    }

    /// Set sort for the relationship
    pub fn with_sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    /// Parse includes from string format
    /// Supports nested includes: "user,organization.positions,organization.positions.level"
    pub fn from_string(include_string: &str) -> Vec<Include> {
        let mut includes = Vec::new();
        let mut include_map: HashMap<String, Include> = HashMap::new();

        for include_part in include_string.split(',') {
            let include_part = include_part.trim();
            if include_part.is_empty() {
                continue;
            }

            let parts: Vec<&str> = include_part.split('.').collect();
            Self::add_nested_include(&mut include_map, &parts, 0);
        }

        includes.extend(include_map.into_values());
        includes
    }

    /// Recursively add nested includes
    fn add_nested_include(
        include_map: &mut HashMap<String, Include>,
        parts: &[&str],
        index: usize,
    ) {
        if index >= parts.len() {
            return;
        }

        let current_part = parts[index];
        let include = include_map
            .entry(current_part.to_string())
            .or_insert_with(|| Include::new(current_part));

        if index + 1 < parts.len() {
            let mut nested_map = HashMap::new();
            Self::add_nested_include(&mut nested_map, parts, index + 1);

            for (_, nested_include) in nested_map {
                // Check if this nested include already exists
                if !include.nested.iter().any(|n| n.relation == nested_include.relation) {
                    include.nested.push(nested_include);
                }
            }
        }
    }

    /// Convert includes back to string format
    pub fn to_string(&self) -> String {
        let mut result = self.relation.clone();

        for nested in &self.nested {
            let nested_str = nested.to_string();
            result.push('.');
            result.push_str(&nested_str);
        }

        result
    }

    /// Convert multiple includes to string format
    pub fn vec_to_string(includes: &[Include]) -> String {
        includes
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Get all relation paths (flattened)
    pub fn get_relation_paths(&self) -> Vec<String> {
        let mut paths = vec![self.relation.clone()];

        for nested in &self.nested {
            let nested_paths = nested.get_relation_paths();
            for nested_path in nested_paths {
                paths.push(format!("{}.{}", self.relation, nested_path));
            }
        }

        paths
    }

    /// Get all relation paths for multiple includes
    pub fn get_all_relation_paths(includes: &[Include]) -> Vec<String> {
        includes
            .iter()
            .flat_map(|i| i.get_relation_paths())
            .collect()
    }

    /// Check if this include contains a specific relation path
    pub fn contains_path(&self, path: &str) -> bool {
        let path_parts: Vec<&str> = path.split('.').collect();
        self.contains_path_parts(&path_parts, 0)
    }

    fn contains_path_parts(&self, parts: &[&str], index: usize) -> bool {
        if index >= parts.len() {
            return true;
        }

        if parts[index] != self.relation {
            return false;
        }

        if index + 1 >= parts.len() {
            return true;
        }

        // Check nested includes
        for nested in &self.nested {
            if nested.contains_path_parts(parts, index + 1) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include_from_string_simple() {
        let includes = Include::from_string("user,organization");
        assert_eq!(includes.len(), 2);

        let relations: Vec<&str> = includes.iter().map(|i| i.relation.as_str()).collect();
        assert!(relations.contains(&"user"));
        assert!(relations.contains(&"organization"));
    }

    #[test]
    fn test_include_from_string_nested() {
        let includes = Include::from_string("user,organization.positions,organization.positions.level");

        let org_include = includes.iter().find(|i| i.relation == "organization").unwrap();
        assert_eq!(org_include.nested.len(), 1);
        assert_eq!(org_include.nested[0].relation, "positions");
        assert_eq!(org_include.nested[0].nested.len(), 1);
        assert_eq!(org_include.nested[0].nested[0].relation, "level");
    }

    #[test]
    fn test_include_to_string() {
        let include = Include::new("organization")
            .with_nested(
                Include::new("positions")
                    .with_nested(Include::new("level"))
            );

        assert_eq!(include.to_string(), "organization.positions.level");
    }

    #[test]
    fn test_include_get_relation_paths() {
        let include = Include::new("organization")
            .with_nested(
                Include::new("positions")
                    .with_nested(Include::new("level"))
            );

        let paths = include.get_relation_paths();
        assert!(paths.contains(&"organization".to_string()));
        assert!(paths.contains(&"organization.positions".to_string()));
        assert!(paths.contains(&"organization.positions.level".to_string()));
    }

    #[test]
    fn test_include_contains_path() {
        let include = Include::new("organization")
            .with_nested(
                Include::new("positions")
                    .with_nested(Include::new("level"))
            );

        assert!(include.contains_path("organization"));
        assert!(include.contains_path("organization.positions"));
        assert!(include.contains_path("organization.positions.level"));
        assert!(!include.contains_path("user"));
        assert!(!include.contains_path("organization.users"));
    }
}