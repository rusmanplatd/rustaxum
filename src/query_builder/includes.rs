use std::collections::{HashMap, HashSet};
use serde::Serialize;

/// Trait for models that support relationship loading
pub trait Relatable {
    /// Get the allowed relationship names that can be included
    fn allowed_includes() -> Vec<&'static str>;

    /// Get the relationship configurations
    fn relationships() -> HashMap<&'static str, Relationship>;
}

/// Relationship configuration
#[derive(Debug, Clone)]
pub struct Relationship {
    pub foreign_key: String,
    pub local_key: String,
    pub related_table: String,
    pub relationship_type: RelationshipType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipType {
    HasOne,
    HasMany,
    BelongsTo,
    BelongsToMany { pivot_table: String, foreign_pivot_key: String, related_pivot_key: String },
}

impl Relationship {
    pub fn has_one(foreign_key: &str, local_key: &str, related_table: &str) -> Self {
        Self {
            foreign_key: foreign_key.to_string(),
            local_key: local_key.to_string(),
            related_table: related_table.to_string(),
            relationship_type: RelationshipType::HasOne,
        }
    }

    pub fn has_many(foreign_key: &str, local_key: &str, related_table: &str) -> Self {
        Self {
            foreign_key: foreign_key.to_string(),
            local_key: local_key.to_string(),
            related_table: related_table.to_string(),
            relationship_type: RelationshipType::HasMany,
        }
    }

    pub fn belongs_to(foreign_key: &str, local_key: &str, related_table: &str) -> Self {
        Self {
            foreign_key: foreign_key.to_string(),
            local_key: local_key.to_string(),
            related_table: related_table.to_string(),
            relationship_type: RelationshipType::BelongsTo,
        }
    }

    pub fn belongs_to_many(
        foreign_key: &str,
        local_key: &str,
        related_table: &str,
        pivot_table: &str,
        foreign_pivot_key: &str,
        related_pivot_key: &str,
    ) -> Self {
        Self {
            foreign_key: foreign_key.to_string(),
            local_key: local_key.to_string(),
            related_table: related_table.to_string(),
            relationship_type: RelationshipType::BelongsToMany {
                pivot_table: pivot_table.to_string(),
                foreign_pivot_key: foreign_pivot_key.to_string(),
                related_pivot_key: related_pivot_key.to_string(),
            },
        }
    }
}

/// Include selector for controlling which relationships to load
#[derive(Debug, Clone)]
pub struct IncludeSelector {
    pub includes: HashSet<String>,
    pub allowed_includes: HashSet<String>,
}

impl IncludeSelector {
    /// Create a new include selector
    pub fn new(includes: Vec<String>, allowed_includes: Vec<String>) -> Self {
        let includes: HashSet<String> = includes.into_iter().collect();
        let allowed_includes: HashSet<String> = allowed_includes.into_iter().collect();

        Self {
            includes,
            allowed_includes,
        }
    }

    /// Get the validated includes
    pub fn get_validated_includes(&self) -> Vec<String> {
        self.includes
            .intersection(&self.allowed_includes)
            .cloned()
            .collect()
    }

    /// Check if a relationship should be included
    pub fn should_include(&self, relationship: &str) -> bool {
        self.includes.contains(relationship) && self.allowed_includes.contains(relationship)
    }

    /// Generate SQL for eager loading relationships
    pub fn build_eager_load_queries<T>(&self, main_results: &[T], relationships: &HashMap<&str, Relationship>) -> Vec<EagerLoadQuery>
    where
        T: serde::Serialize,
    {
        let mut queries = Vec::new();

        for include in self.get_validated_includes() {
            if let Some(relationship) = relationships.get(include.as_str()) {
                // Extract the local key values from main results
                let local_values = self.extract_values_from_results(main_results, &relationship.local_key);

                if !local_values.is_empty() {
                    let query = match &relationship.relationship_type {
                        RelationshipType::HasOne | RelationshipType::HasMany => {
                            EagerLoadQuery {
                                relationship_name: include.clone(),
                                sql: format!(
                                    "SELECT * FROM {} WHERE {} IN ({})",
                                    relationship.related_table,
                                    relationship.foreign_key,
                                    local_values.iter().map(|_| "?").collect::<Vec<_>>().join(", ")
                                ),
                                parameters: local_values.clone(),
                                relationship_type: relationship.relationship_type.clone(),
                                local_key: relationship.local_key.clone(),
                                foreign_key: relationship.foreign_key.clone(),
                            }
                        },
                        RelationshipType::BelongsTo => {
                            EagerLoadQuery {
                                relationship_name: include.clone(),
                                sql: format!(
                                    "SELECT * FROM {} WHERE {} IN ({})",
                                    relationship.related_table,
                                    relationship.local_key,
                                    local_values.iter().map(|_| "?").collect::<Vec<_>>().join(", ")
                                ),
                                parameters: local_values.clone(),
                                relationship_type: relationship.relationship_type.clone(),
                                local_key: relationship.local_key.clone(),
                                foreign_key: relationship.foreign_key.clone(),
                            }
                        },
                        RelationshipType::BelongsToMany { pivot_table, foreign_pivot_key, related_pivot_key } => {
                            EagerLoadQuery {
                                relationship_name: include.clone(),
                                sql: format!(
                                    "SELECT r.*, p.{} as pivot_local_key FROM {} r
                                     INNER JOIN {} p ON r.{} = p.{}
                                     WHERE p.{} IN ({})",
                                    foreign_pivot_key,
                                    relationship.related_table,
                                    pivot_table,
                                    relationship.local_key,
                                    related_pivot_key,
                                    foreign_pivot_key,
                                    local_values.iter().map(|_| "?").collect::<Vec<_>>().join(", ")
                                ),
                                parameters: local_values.clone(),
                                relationship_type: relationship.relationship_type.clone(),
                                local_key: relationship.local_key.clone(),
                                foreign_key: relationship.foreign_key.clone(),
                            }
                        },
                    };
                    queries.push(query);
                }
            }
        }

        queries
    }

    /// Extract values from serialized results for a given key
    fn extract_values_from_results<T>(&self, results: &[T], key: &str) -> Vec<String>
    where
        T: serde::Serialize,
    {
        let mut values = Vec::new();

        for result in results {
            if let Ok(json_value) = serde_json::to_value(result) {
                if let Some(value) = json_value.get(key) {
                    if let Some(string_val) = value.as_str() {
                        values.push(string_val.to_string());
                    } else if let Some(num_val) = value.as_i64() {
                        values.push(num_val.to_string());
                    }
                }
            }
        }

        values.sort();
        values.dedup();
        values
    }
}

/// Eager load query structure
#[derive(Debug, Clone)]
pub struct EagerLoadQuery {
    pub relationship_name: String,
    pub sql: String,
    pub parameters: Vec<String>,
    pub relationship_type: RelationshipType,
    pub local_key: String,
    pub foreign_key: String,
}

/// Response structure that includes relationships
#[derive(Debug, Serialize)]
pub struct WithRelationships<T> {
    #[serde(flatten)]
    pub model: T,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub relationships: HashMap<String, serde_json::Value>,
}

impl<T> WithRelationships<T> {
    pub fn new(model: T) -> Self {
        Self {
            model,
            relationships: HashMap::new(),
        }
    }

    pub fn with_relationship(mut self, name: String, data: serde_json::Value) -> Self {
        self.relationships.insert(name, data);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include_selector() {
        let allowed = vec!["posts".to_string(), "comments".to_string(), "tags".to_string()];

        let selector = IncludeSelector::new(vec!["posts".to_string(), "invalid".to_string()], allowed);
        let includes = selector.get_validated_includes();

        assert!(includes.contains(&"posts".to_string()));
        assert!(!includes.contains(&"invalid".to_string()));
    }

    #[test]
    fn test_relationship_creation() {
        let rel = Relationship::has_many("user_id", "id", "posts");
        assert_eq!(rel.foreign_key, "user_id");
        assert_eq!(rel.local_key, "id");
        assert_eq!(rel.related_table, "posts");
        assert_eq!(rel.relationship_type, RelationshipType::HasMany);
    }
}