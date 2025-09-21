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

/// Trait for models that support filtering
/// This is simplified to work with basic Diesel queries
pub trait Filterable {
    /// Apply basic filtering - implementation depends on the specific model
    fn apply_basic_filter(column: &str, operator: &str, value: &serde_json::Value) -> String;
}

/// Trait for models that support sorting
/// This is simplified to work with basic Diesel queries
pub trait Sortable {
    /// Apply basic sorting - implementation depends on the specific model
    fn apply_basic_sort(column: &str, direction: &str) -> String;
}

/// Trait for models that support relationship inclusion
/// This is simplified since Diesel doesn't have automatic eager loading like Laravel
pub trait Includable {
    /// Get the relationship data for includes
    /// Implementation should handle loading related data separately
    fn load_relationships(ids: &[String], includes: &[String], conn: &mut PgConnection) -> Result<()>;
}

/// Trait for models that support field selection
pub trait Selectable {
    /// Get the selected fields as SQL
    fn get_select_sql(fields: &[String]) -> String;
}