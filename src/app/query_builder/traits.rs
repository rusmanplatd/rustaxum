use crate::app::query_builder::SortDirection;
use diesel::pg::PgConnection;
use anyhow::Result;

/// Trait for models that can be queried using the query builder
pub trait Queryable: Sized {
    /// The table name for this model
    fn table_name() -> &'static str;

    /// List of fields that can be filtered on
    fn allowed_filters() -> Vec<&'static str>;

    /// List of fields that can be sorted on
    fn allowed_sorts() -> Vec<&'static str>;

    /// List of fields that can be selected
    fn allowed_fields() -> Vec<&'static str>;

    /// List of relationships that can be included
    fn allowed_includes() -> Vec<&'static str> {
        vec![]
    }

    /// Default sort field and direction
    fn default_sort() -> Option<(&'static str, SortDirection)> {
        None
    }

    /// Default fields to select when no fields are specified
    fn default_fields() -> Vec<&'static str> {
        Self::allowed_fields()
    }

    /// Check if a filter field is allowed
    fn is_filter_allowed(field: &str) -> bool {
        Self::allowed_filters().contains(&field)
    }

    /// Check if a sort field is allowed
    fn is_sort_allowed(field: &str) -> bool {
        Self::allowed_sorts().contains(&field)
    }

    /// Check if a field can be selected
    fn is_field_allowed(field: &str) -> bool {
        Self::allowed_fields().contains(&field)
    }

    /// Check if an include relationship is allowed
    fn is_include_allowed(include: &str) -> bool {
        Self::allowed_includes().contains(&include)
    }
}

/// Trait for models that support filtering with complex operations
pub trait Filterable {
    /// Apply basic filtering - implementation depends on the specific model
    fn apply_basic_filter(column: &str, operator: &str, value: &serde_json::Value) -> String;

    /// Apply range filtering (e.g. BETWEEN, >, <, >=, <=)
    fn apply_range_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
        match operator {
            "between" => {
                if let Some(values) = value.as_array() {
                    if values.len() == 2 {
                        format!("{} BETWEEN {} AND {}", column,
                               Self::format_filter_value(&values[0]),
                               Self::format_filter_value(&values[1]))
                    } else {
                        format!("{} = {}", column, Self::format_filter_value(value))
                    }
                } else {
                    format!("{} = {}", column, Self::format_filter_value(value))
                }
            },
            "gt" | ">" => format!("{} > {}", column, Self::format_filter_value(value)),
            "gte" | ">=" => format!("{} >= {}", column, Self::format_filter_value(value)),
            "lt" | "<" => format!("{} < {}", column, Self::format_filter_value(value)),
            "lte" | "<=" => format!("{} <= {}", column, Self::format_filter_value(value)),
            _ => Self::apply_basic_filter(column, operator, value)
        }
    }

    /// Apply IN filtering for multiple values
    fn apply_in_filter(column: &str, values: &[serde_json::Value]) -> String {
        if values.is_empty() {
            return "1=0".to_string(); // No matches
        }
        let formatted_values: Vec<String> = values.iter()
            .map(|v| Self::format_filter_value(v))
            .collect();
        format!("{} IN ({})", column, formatted_values.join(", "))
    }

    /// Apply LIKE filtering for text search
    fn apply_like_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
        let search_value = if let Some(text) = value.as_str() {
            match operator {
                "like" => format!("'{}'", text.replace("'", "''")),
                "ilike" => format!("'{}'", text.replace("'", "''")),
                "starts_with" => format!("'{}%'", text.replace("'", "''")),
                "ends_with" => format!("'%{}'", text.replace("'", "''")),
                "contains" => format!("'%{}%'", text.replace("'", "''")),
                _ => format!("'{}'", text.replace("'", "''"))
            }
        } else {
            "'%'".to_string()
        };

        match operator {
            "ilike" | "contains" | "starts_with" | "ends_with" => {
                format!("{} ILIKE {}", column, search_value)
            },
            _ => format!("{} LIKE {}", column, search_value)
        }
    }

    /// Apply NULL/NOT NULL filtering
    fn apply_null_filter(column: &str, is_null: bool) -> String {
        if is_null {
            format!("{} IS NULL", column)
        } else {
            format!("{} IS NOT NULL", column)
        }
    }

    /// Format a single filter value for SQL
    fn format_filter_value(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "NULL".to_string(),
            _ => format!("'{}'", value.to_string().replace("'", "''"))
        }
    }

    /// Apply complex filtering based on operator type
    fn apply_filter(column: &str, operator: &str, value: &serde_json::Value) -> String {
        match operator {
            "eq" | "=" => Self::apply_basic_filter(column, "=", value),
            "ne" | "!=" => Self::apply_basic_filter(column, "!=", value),
            "gt" | ">" | "gte" | ">=" | "lt" | "<" | "lte" | "<=" | "between" => {
                Self::apply_range_filter(column, operator, value)
            },
            "in" => {
                if let Some(values) = value.as_array() {
                    Self::apply_in_filter(column, values)
                } else {
                    Self::apply_basic_filter(column, "=", value)
                }
            },
            "like" | "ilike" | "contains" | "starts_with" | "ends_with" => {
                Self::apply_like_filter(column, operator, value)
            },
            "is_null" => Self::apply_null_filter(column, true),
            "is_not_null" => Self::apply_null_filter(column, false),
            _ => Self::apply_basic_filter(column, operator, value)
        }
    }
}

/// Trait for models that support sorting with multi-column capabilities
pub trait Sortable {
    /// Apply basic sorting - implementation depends on the specific model
    fn apply_basic_sort(column: &str, direction: &str) -> String;

    /// Apply multi-column sorting
    fn apply_multi_sort(sorts: &[(String, SortDirection)]) -> String {
        if sorts.is_empty() {
            return String::new();
        }

        let sort_clauses: Vec<String> = sorts.iter()
            .map(|(column, direction)| {
                let dir_str = match direction {
                    SortDirection::Asc => "ASC",
                    SortDirection::Desc => "DESC",
                };
                Self::apply_basic_sort(column, dir_str)
            })
            .collect();

        sort_clauses.join(", ")
    }

    /// Validate and apply sorting with fallback to default sort
    fn apply_validated_sort(sorts: &[(String, SortDirection)], allowed_sorts: &[&str]) -> String {
        let valid_sorts: Vec<(String, SortDirection)> = sorts.iter()
            .filter(|(column, _)| allowed_sorts.contains(&column.as_str()))
            .cloned()
            .collect();

        if valid_sorts.is_empty() {
            // Return empty string if no valid sorts, let caller handle default
            String::new()
        } else {
            Self::apply_multi_sort(&valid_sorts)
        }
    }

    /// Parse sort string into structured format
    /// Expected format: "column1:asc,column2:desc" or "column1,-column2"
    fn parse_sort_string(sort_str: &str) -> Vec<(String, SortDirection)> {
        if sort_str.is_empty() {
            return vec![];
        }

        sort_str.split(',')
            .filter_map(|s| {
                let s = s.trim();
                if s.is_empty() {
                    return None;
                }

                if s.starts_with('-') {
                    // Format: "-column" means DESC
                    Some((s[1..].to_string(), SortDirection::Desc))
                } else if s.contains(':') {
                    // Format: "column:direction"
                    let parts: Vec<&str> = s.split(':').collect();
                    if parts.len() == 2 {
                        let column = parts[0].trim().to_string();
                        let direction = match parts[1].trim().to_lowercase().as_str() {
                            "desc" | "descending" | "down" => SortDirection::Desc,
                            _ => SortDirection::Asc,
                        };
                        Some((column, direction))
                    } else {
                        Some((s.to_string(), SortDirection::Asc))
                    }
                } else {
                    // Format: "column" means ASC
                    Some((s.to_string(), SortDirection::Asc))
                }
            })
            .collect()
    }

    /// Build ORDER BY clause from sort parameters
    fn build_order_by_clause(sorts: &[(String, SortDirection)]) -> String {
        if sorts.is_empty() {
            return String::new();
        }

        let order_clause = Self::apply_multi_sort(sorts);
        if order_clause.is_empty() {
            String::new()
        } else {
            format!("ORDER BY {}", order_clause)
        }
    }
}

/// Trait for models that support relationship inclusion with data loading
pub trait Includable {
    /// Get the relationship data for includes
    /// Implementation should handle loading related data separately
    fn load_relationships(ids: &[String], includes: &[String], conn: &mut PgConnection) -> Result<()>;

    /// Load a single relationship by name
    fn load_relationship(ids: &[String], relationship: &str, conn: &mut PgConnection) -> Result<serde_json::Value> {
        // Default implementation returns empty object
        // Individual models should override this for specific relationships
        match relationship {
            _ => Ok(serde_json::json!({}))
        }
    }

    /// Load multiple relationships efficiently in batch
    fn load_multiple_relationships(ids: &[String], includes: &[String], conn: &mut PgConnection) -> Result<std::collections::HashMap<String, serde_json::Value>> {
        let mut relationships = std::collections::HashMap::new();

        for include in includes {
            let relationship_data = Self::load_relationship(ids, include, conn)?;
            relationships.insert(include.to_string(), relationship_data);
        }

        Ok(relationships)
    }

    /// Build relationship join clause for SQL queries
    fn build_join_clause(relationship: &str, main_table: &str) -> Option<String> {
        // Default implementation - models should override for specific relationships
        // Returns None if relationship is not supported
        None
    }

    /// Get the foreign key column for a relationship
    fn get_foreign_key(relationship: &str) -> Option<String> {
        // Common patterns for foreign keys
        match relationship {
            rel if rel.ends_with("s") => {
                // Plural relationship (hasMany) - foreign key is usually singular_id
                let singular = &rel[..rel.len()-1];
                Some(format!("{}_id", singular))
            },
            rel => {
                // Singular relationship (belongsTo, hasOne) - foreign key is usually rel_id
                Some(format!("{}_id", rel))
            }
        }
    }

    /// Check if relationship should be loaded eagerly or lazily
    fn should_eager_load(relationship: &str) -> bool {
        // Default to lazy loading for performance
        // Models can override for specific relationships that should always be eager loaded
        false
    }

    /// Validate that relationships exist and are allowed
    fn validate_includes(includes: &[String], allowed_includes: &[&str]) -> Vec<String> {
        includes.iter()
            .filter(|include| allowed_includes.contains(&include.as_str()))
            .cloned()
            .collect()
    }

    /// Parse nested includes (e.g., "user.profile.avatar")
    fn parse_nested_includes(include_str: &str) -> Vec<Vec<String>> {
        include_str.split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.split('.').map(|part| part.trim().to_string()).collect())
            .collect()
    }

    /// Build relationship query for specific model type
    fn build_relationship_query(relationship: &str, parent_ids: &[String]) -> Option<String> {
        if parent_ids.is_empty() {
            return None;
        }

        let foreign_key = Self::get_foreign_key(relationship)?;
        let id_list = parent_ids.iter()
            .map(|id| format!("'{}'", id.replace("'", "''")))
            .collect::<Vec<_>>()
            .join(", ");

        Some(format!("SELECT * FROM {} WHERE {} IN ({})", relationship, foreign_key, id_list))
    }

    /// Group related data by parent ID for efficient association
    fn group_by_parent_id(data: &serde_json::Value, foreign_key: &str) -> std::collections::HashMap<String, Vec<serde_json::Value>> {
        let mut grouped = std::collections::HashMap::new();

        if let Some(items) = data.as_array() {
            for item in items {
                if let Some(parent_id) = item.get(foreign_key).and_then(|v| v.as_str()) {
                    grouped.entry(parent_id.to_string())
                        .or_insert_with(Vec::new)
                        .push(item.clone());
                }
            }
        }

        grouped
    }
}

/// Trait for models that support field selection
pub trait Selectable {
    /// Get the selected fields as SQL
    fn get_select_sql(fields: &[String]) -> String;
}