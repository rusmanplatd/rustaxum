use std::collections::{HashMap, HashSet};
use serde::Serialize;

use super::filter::FilterOperator;

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
    pub constraints: Vec<RelationshipConstraint>,
}

/// Constraint for relationship queries
#[derive(Debug, Clone)]
pub struct RelationshipConstraint {
    pub field: String,
    pub operator: FilterOperator,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelationshipType {
    HasOne,
    HasMany,
    BelongsTo,
    BelongsToMany {
        pivot_table: String,
        foreign_pivot_key: String,
        related_pivot_key: String
    },
    HasOneThrough {
        through_table: String,
        first_key: String,
        second_key: String,
        local_key: String,
        second_local_key: String,
    },
    HasManyThrough {
        through_table: String,
        first_key: String,
        second_key: String,
        local_key: String,
        second_local_key: String,
    },
    MorphTo {
        morph_type: String,
        morph_id: String,
    },
    MorphOne {
        morph_type: String,
        morph_id: String,
        morph_name: String,
    },
    MorphMany {
        morph_type: String,
        morph_id: String,
        morph_name: String,
    },
    MorphToMany {
        pivot_table: String,
        foreign_pivot_key: String,
        related_pivot_key: String,
        morph_type: String,
        morph_id: String,
        morph_name: String,
    },
}

impl Relationship {
    pub fn has_one(foreign_key: &str, local_key: &str, related_table: &str) -> Self {
        Self {
            foreign_key: foreign_key.to_string(),
            local_key: local_key.to_string(),
            related_table: related_table.to_string(),
            relationship_type: RelationshipType::HasOne,
            constraints: Vec::new(),
        }
    }

    pub fn has_many(foreign_key: &str, local_key: &str, related_table: &str) -> Self {
        Self {
            foreign_key: foreign_key.to_string(),
            local_key: local_key.to_string(),
            related_table: related_table.to_string(),
            relationship_type: RelationshipType::HasMany,
            constraints: Vec::new(),
        }
    }

    pub fn belongs_to(foreign_key: &str, local_key: &str, related_table: &str) -> Self {
        Self {
            foreign_key: foreign_key.to_string(),
            local_key: local_key.to_string(),
            related_table: related_table.to_string(),
            relationship_type: RelationshipType::BelongsTo,
            constraints: Vec::new(),
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
            constraints: Vec::new(),
        }
    }

    pub fn has_one_through(
        foreign_key: &str,
        local_key: &str,
        related_table: &str,
        through_table: &str,
        first_key: &str,
        second_key: &str,
        second_local_key: &str,
    ) -> Self {
        Self {
            foreign_key: foreign_key.to_string(),
            local_key: local_key.to_string(),
            related_table: related_table.to_string(),
            relationship_type: RelationshipType::HasOneThrough {
                through_table: through_table.to_string(),
                first_key: first_key.to_string(),
                second_key: second_key.to_string(),
                local_key: local_key.to_string(),
                second_local_key: second_local_key.to_string(),
            },
            constraints: Vec::new(),
        }
    }

    pub fn has_many_through(
        foreign_key: &str,
        local_key: &str,
        related_table: &str,
        through_table: &str,
        first_key: &str,
        second_key: &str,
        second_local_key: &str,
    ) -> Self {
        Self {
            foreign_key: foreign_key.to_string(),
            local_key: local_key.to_string(),
            related_table: related_table.to_string(),
            relationship_type: RelationshipType::HasManyThrough {
                through_table: through_table.to_string(),
                first_key: first_key.to_string(),
                second_key: second_key.to_string(),
                local_key: local_key.to_string(),
                second_local_key: second_local_key.to_string(),
            },
            constraints: Vec::new(),
        }
    }

    pub fn morph_to(foreign_key: &str, morph_type: &str, morph_id: &str) -> Self {
        Self {
            foreign_key: foreign_key.to_string(),
            local_key: "".to_string(),
            related_table: "".to_string(),
            relationship_type: RelationshipType::MorphTo {
                morph_type: morph_type.to_string(),
                morph_id: morph_id.to_string(),
            },
            constraints: Vec::new(),
        }
    }

    pub fn morph_one(
        foreign_key: &str,
        local_key: &str,
        related_table: &str,
        morph_name: &str,
        morph_type: &str,
        morph_id: &str,
    ) -> Self {
        Self {
            foreign_key: foreign_key.to_string(),
            local_key: local_key.to_string(),
            related_table: related_table.to_string(),
            relationship_type: RelationshipType::MorphOne {
                morph_type: morph_type.to_string(),
                morph_id: morph_id.to_string(),
                morph_name: morph_name.to_string(),
            },
            constraints: Vec::new(),
        }
    }

    pub fn morph_many(
        foreign_key: &str,
        local_key: &str,
        related_table: &str,
        morph_name: &str,
        morph_type: &str,
        morph_id: &str,
    ) -> Self {
        Self {
            foreign_key: foreign_key.to_string(),
            local_key: local_key.to_string(),
            related_table: related_table.to_string(),
            relationship_type: RelationshipType::MorphMany {
                morph_type: morph_type.to_string(),
                morph_id: morph_id.to_string(),
                morph_name: morph_name.to_string(),
            },
            constraints: Vec::new(),
        }
    }

    pub fn morph_to_many(
        foreign_key: &str,
        local_key: &str,
        related_table: &str,
        pivot_table: &str,
        foreign_pivot_key: &str,
        related_pivot_key: &str,
        morph_name: &str,
        morph_type: &str,
        morph_id: &str,
    ) -> Self {
        Self {
            foreign_key: foreign_key.to_string(),
            local_key: local_key.to_string(),
            related_table: related_table.to_string(),
            relationship_type: RelationshipType::MorphToMany {
                pivot_table: pivot_table.to_string(),
                foreign_pivot_key: foreign_pivot_key.to_string(),
                related_pivot_key: related_pivot_key.to_string(),
                morph_type: morph_type.to_string(),
                morph_id: morph_id.to_string(),
                morph_name: morph_name.to_string(),
            },
            constraints: Vec::new(),
        }
    }

    /// Add a constraint to this relationship
    pub fn with_constraint(mut self, field: &str, operator: FilterOperator, value: &str) -> Self {
        self.constraints.push(RelationshipConstraint {
            field: field.to_string(),
            operator,
            value: value.to_string(),
        });
        self
    }

    /// Add multiple constraints to this relationship
    pub fn with_constraints(mut self, constraints: Vec<RelationshipConstraint>) -> Self {
        self.constraints.extend(constraints);
        self
    }

    /// Generate WHERE clause for relationship constraints
    pub fn build_constraint_clause(&self) -> (String, Vec<String>) {
        if self.constraints.is_empty() {
            return (String::new(), Vec::new());
        }

        let mut clauses = Vec::new();
        let mut params = Vec::new();

        for constraint in &self.constraints {
            match constraint.operator {
                FilterOperator::Eq => {
                    clauses.push(format!("{} = ?", constraint.field));
                    params.push(constraint.value.clone());
                },
                FilterOperator::Ne => {
                    clauses.push(format!("{} != ?", constraint.field));
                    params.push(constraint.value.clone());
                },
                FilterOperator::Gt => {
                    clauses.push(format!("{} > ?", constraint.field));
                    params.push(constraint.value.clone());
                },
                FilterOperator::Gte => {
                    clauses.push(format!("{} >= ?", constraint.field));
                    params.push(constraint.value.clone());
                },
                FilterOperator::Lt => {
                    clauses.push(format!("{} < ?", constraint.field));
                    params.push(constraint.value.clone());
                },
                FilterOperator::Lte => {
                    clauses.push(format!("{} <= ?", constraint.field));
                    params.push(constraint.value.clone());
                },
                FilterOperator::Like => {
                    clauses.push(format!("{} LIKE ?", constraint.field));
                    params.push(format!("%{}%", constraint.value));
                },
                FilterOperator::NotLike => {
                    clauses.push(format!("{} NOT LIKE ?", constraint.field));
                    params.push(format!("%{}%", constraint.value));
                },
                FilterOperator::In => {
                    let values: Vec<&str> = constraint.value.split(',').collect();
                    let placeholders = values.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                    clauses.push(format!("{} IN ({})", constraint.field, placeholders));
                    params.extend(values.iter().map(|v| v.trim().to_string()));
                },
                FilterOperator::NotIn => {
                    let values: Vec<&str> = constraint.value.split(',').collect();
                    let placeholders = values.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                    clauses.push(format!("{} NOT IN ({})", constraint.field, placeholders));
                    params.extend(values.iter().map(|v| v.trim().to_string()));
                },
                FilterOperator::IsNull => {
                    clauses.push(format!("{} IS NULL", constraint.field));
                },
                FilterOperator::IsNotNull => {
                    clauses.push(format!("{} IS NOT NULL", constraint.field));
                },
            }
        }

        let clause = if clauses.is_empty() {
            String::new()
        } else {
            format!(" AND {}", clauses.join(" AND "))
        };

        (clause, params)
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
                    // Build constraint clause for this relationship
                    let (constraint_clause, constraint_params) = relationship.build_constraint_clause();

                    let query = match &relationship.relationship_type {
                        RelationshipType::HasOne | RelationshipType::HasMany => {
                            let mut all_params = local_values.clone();
                            all_params.extend(constraint_params.clone());

                            EagerLoadQuery {
                                relationship_name: include.clone(),
                                sql: format!(
                                    "SELECT * FROM {} WHERE {} IN ({}){}",
                                    relationship.related_table,
                                    relationship.foreign_key,
                                    local_values.iter().map(|_| "?").collect::<Vec<_>>().join(", "),
                                    constraint_clause
                                ),
                                parameters: all_params,
                                relationship_type: relationship.relationship_type.clone(),
                                local_key: relationship.local_key.clone(),
                                foreign_key: relationship.foreign_key.clone(),
                            }
                        },
                        RelationshipType::BelongsTo => {
                            let mut all_params = local_values.clone();
                            all_params.extend(constraint_params.clone());

                            EagerLoadQuery {
                                relationship_name: include.clone(),
                                sql: format!(
                                    "SELECT * FROM {} WHERE {} IN ({}){}",
                                    relationship.related_table,
                                    relationship.local_key,
                                    local_values.iter().map(|_| "?").collect::<Vec<_>>().join(", "),
                                    constraint_clause
                                ),
                                parameters: all_params,
                                relationship_type: relationship.relationship_type.clone(),
                                local_key: relationship.local_key.clone(),
                                foreign_key: relationship.foreign_key.clone(),
                            }
                        },
                        RelationshipType::BelongsToMany { pivot_table, foreign_pivot_key, related_pivot_key } => {
                            let mut all_params = local_values.clone();
                            all_params.extend(constraint_params.clone());

                            EagerLoadQuery {
                                relationship_name: include.clone(),
                                sql: format!(
                                    "SELECT r.*, p.{} as pivot_local_key FROM {} r
                                     INNER JOIN {} p ON r.{} = p.{}
                                     WHERE p.{} IN ({}){}",
                                    foreign_pivot_key,
                                    relationship.related_table,
                                    pivot_table,
                                    relationship.local_key,
                                    related_pivot_key,
                                    foreign_pivot_key,
                                    local_values.iter().map(|_| "?").collect::<Vec<_>>().join(", "),
                                    constraint_clause
                                ),
                                parameters: all_params,
                                relationship_type: relationship.relationship_type.clone(),
                                local_key: relationship.local_key.clone(),
                                foreign_key: relationship.foreign_key.clone(),
                            }
                        },
                        RelationshipType::HasOneThrough { through_table, first_key, second_key, local_key: _, second_local_key } => {
                            let mut all_params = local_values.clone();
                            all_params.extend(constraint_params.clone());

                            EagerLoadQuery {
                                relationship_name: include.clone(),
                                sql: format!(
                                    "SELECT r.* FROM {} r
                                     INNER JOIN {} t ON t.{} = r.{}
                                     WHERE t.{} IN ({}){}",
                                    relationship.related_table,
                                    through_table,
                                    second_local_key,
                                    second_key,
                                    first_key,
                                    local_values.iter().map(|_| "?").collect::<Vec<_>>().join(", "),
                                    constraint_clause
                                ),
                                parameters: all_params,
                                relationship_type: relationship.relationship_type.clone(),
                                local_key: relationship.local_key.clone(),
                                foreign_key: relationship.foreign_key.clone(),
                            }
                        },
                        RelationshipType::HasManyThrough { through_table, first_key, second_key, local_key: _, second_local_key } => {
                            let mut all_params = local_values.clone();
                            all_params.extend(constraint_params.clone());

                            EagerLoadQuery {
                                relationship_name: include.clone(),
                                sql: format!(
                                    "SELECT r.* FROM {} r
                                     INNER JOIN {} t ON t.{} = r.{}
                                     WHERE t.{} IN ({}){}",
                                    relationship.related_table,
                                    through_table,
                                    second_local_key,
                                    second_key,
                                    first_key,
                                    local_values.iter().map(|_| "?").collect::<Vec<_>>().join(", "),
                                    constraint_clause
                                ),
                                parameters: all_params,
                                relationship_type: relationship.relationship_type.clone(),
                                local_key: relationship.local_key.clone(),
                                foreign_key: relationship.foreign_key.clone(),
                            }
                        },
                        RelationshipType::MorphTo { morph_type, morph_id: _ } => {
                            // For MorphTo, we need to handle multiple potential tables
                            // This is more complex and would require additional logic
                            EagerLoadQuery {
                                relationship_name: include.clone(),
                                sql: format!(
                                    "-- MorphTo relationship requires dynamic table resolution based on {} field",
                                    morph_type
                                ),
                                parameters: local_values.clone(),
                                relationship_type: relationship.relationship_type.clone(),
                                local_key: relationship.local_key.clone(),
                                foreign_key: relationship.foreign_key.clone(),
                            }
                        },
                        RelationshipType::MorphOne { morph_type, morph_id, morph_name } => {
                            let mut all_params = vec![morph_name.clone()];
                            all_params.extend(local_values.clone());
                            all_params.extend(constraint_params.clone());

                            EagerLoadQuery {
                                relationship_name: include.clone(),
                                sql: format!(
                                    "SELECT * FROM {} WHERE {} = ? AND {} IN ({}){}",
                                    relationship.related_table,
                                    morph_type,
                                    morph_id,
                                    local_values.iter().map(|_| "?").collect::<Vec<_>>().join(", "),
                                    constraint_clause
                                ),
                                parameters: all_params,
                                relationship_type: relationship.relationship_type.clone(),
                                local_key: relationship.local_key.clone(),
                                foreign_key: relationship.foreign_key.clone(),
                            }
                        },
                        RelationshipType::MorphMany { morph_type, morph_id, morph_name } => {
                            let mut all_params = vec![morph_name.clone()];
                            all_params.extend(local_values.clone());
                            all_params.extend(constraint_params.clone());

                            EagerLoadQuery {
                                relationship_name: include.clone(),
                                sql: format!(
                                    "SELECT * FROM {} WHERE {} = ? AND {} IN ({}){}",
                                    relationship.related_table,
                                    morph_type,
                                    morph_id,
                                    local_values.iter().map(|_| "?").collect::<Vec<_>>().join(", "),
                                    constraint_clause
                                ),
                                parameters: all_params,
                                relationship_type: relationship.relationship_type.clone(),
                                local_key: relationship.local_key.clone(),
                                foreign_key: relationship.foreign_key.clone(),
                            }
                        },
                        RelationshipType::MorphToMany { pivot_table, foreign_pivot_key, related_pivot_key, morph_type, morph_id, morph_name } => {
                            let mut all_params = vec![morph_name.clone()];
                            all_params.extend(local_values.clone());
                            all_params.extend(constraint_params.clone());

                            EagerLoadQuery {
                                relationship_name: include.clone(),
                                sql: format!(
                                    "SELECT r.*, p.{} as pivot_local_key FROM {} r
                                     INNER JOIN {} p ON r.{} = p.{}
                                     WHERE p.{} = ? AND p.{} IN ({}){}",
                                    foreign_pivot_key,
                                    relationship.related_table,
                                    pivot_table,
                                    relationship.local_key,
                                    related_pivot_key,
                                    morph_type,
                                    morph_id,
                                    local_values.iter().map(|_| "?").collect::<Vec<_>>().join(", "),
                                    constraint_clause
                                ),
                                parameters: all_params,
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
        assert!(rel.constraints.is_empty());
    }

    #[test]
    fn test_relationship_with_constraints() {
        use super::FilterOperator;

        let rel = Relationship::has_many("user_id", "id", "posts")
            .with_constraint("status", FilterOperator::Eq, "published")
            .with_constraint("created_at", FilterOperator::Gte, "2023-01-01");

        assert_eq!(rel.constraints.len(), 2);
        assert_eq!(rel.constraints[0].field, "status");
        assert_eq!(rel.constraints[0].operator, FilterOperator::Eq);
        assert_eq!(rel.constraints[0].value, "published");
    }

    #[test]
    fn test_has_one_through_relationship() {
        let rel = Relationship::has_one_through(
            "id",
            "id",
            "profiles",
            "posts",
            "user_id",
            "user_id",
            "id"
        );

        match rel.relationship_type {
            RelationshipType::HasOneThrough { through_table, first_key, second_key, .. } => {
                assert_eq!(through_table, "posts");
                assert_eq!(first_key, "user_id");
                assert_eq!(second_key, "user_id");
            },
            _ => panic!("Expected HasOneThrough relationship"),
        }
    }

    #[test]
    fn test_morph_many_relationship() {
        let rel = Relationship::morph_many(
            "id",
            "id",
            "comments",
            "Post",
            "commentable_type",
            "commentable_id"
        );

        match rel.relationship_type {
            RelationshipType::MorphMany { morph_type, morph_id, morph_name } => {
                assert_eq!(morph_type, "commentable_type");
                assert_eq!(morph_id, "commentable_id");
                assert_eq!(morph_name, "Post");
            },
            _ => panic!("Expected MorphMany relationship"),
        }
    }

    #[test]
    fn test_belongs_to_many_relationship() {
        let rel = Relationship::belongs_to_many(
            "id",
            "id",
            "roles",
            "user_roles",
            "user_id",
            "role_id"
        );

        match rel.relationship_type {
            RelationshipType::BelongsToMany { pivot_table, foreign_pivot_key, related_pivot_key } => {
                assert_eq!(pivot_table, "user_roles");
                assert_eq!(foreign_pivot_key, "user_id");
                assert_eq!(related_pivot_key, "role_id");
            },
            _ => panic!("Expected BelongsToMany relationship"),
        }
    }

    #[test]
    fn test_constraint_clause_generation() {
        use super::FilterOperator;

        let rel = Relationship::has_many("user_id", "id", "posts")
            .with_constraint("status", FilterOperator::Eq, "published")
            .with_constraint("views", FilterOperator::Gt, "100");

        let (clause, params) = rel.build_constraint_clause();

        assert!(!clause.is_empty());
        assert!(clause.contains("status = ?"));
        assert!(clause.contains("views > ?"));
        assert_eq!(params.len(), 2);
        assert!(params.contains(&"published".to_string()));
        assert!(params.contains(&"100".to_string()));
    }
}