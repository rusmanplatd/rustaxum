use std::collections::HashMap;

use super::{FilterGroup, FilterCondition, FilterOperator, SimpleFilter};

/// Trait for models that support query scopes
pub trait Scopeable {
    /// Get available scopes for the model
    fn scopes() -> HashMap<&'static str, Box<dyn Fn() -> FilterGroup + Send + Sync>>;

    /// Apply a scope by name
    fn apply_scope(name: &str) -> Option<FilterGroup> {
        Self::scopes().get(name).map(|scope_fn| scope_fn())
    }
}

/// Macro for defining scopes easily
#[macro_export]
macro_rules! scope {
    ($name:ident, $filter:expr) => {
        pub fn $name() -> FilterGroup {
            $filter
        }
    };
}

/// Built-in common scopes
pub struct CommonScopes;

impl CommonScopes {
    /// Active records scope (status = 'active')
    pub fn active() -> FilterGroup {
        FilterGroup::And(vec![
            FilterCondition::Simple(SimpleFilter::new(
                "status".to_string(),
                FilterOperator::Eq,
                Some("active".to_string())
            ))
        ])
    }

    /// Recently created scope (created within last 30 days)
    pub fn recent() -> FilterGroup {
        FilterGroup::And(vec![
            FilterCondition::Simple(SimpleFilter::new(
                "created_at".to_string(),
                FilterOperator::Gte,
                Some(
                    chrono::Utc::now()
                        .checked_sub_days(chrono::Days::new(30))
                        .unwrap_or(chrono::Utc::now())
                        .format("%Y-%m-%d")
                        .to_string()
                )
            ))
        ])
    }

    /// Published content scope
    pub fn published() -> FilterGroup {
        FilterGroup::And(vec![
            FilterCondition::Simple(SimpleFilter::new(
                "published_at".to_string(),
                FilterOperator::IsNotNull,
                None
            )),
            FilterCondition::Simple(SimpleFilter::new(
                "published_at".to_string(),
                FilterOperator::Lte,
                Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string())
            ))
        ])
    }

    /// Archived records scope
    pub fn archived() -> FilterGroup {
        FilterGroup::And(vec![
            FilterCondition::Simple(SimpleFilter::new(
                "archived_at".to_string(),
                FilterOperator::IsNotNull,
                None
            ))
        ])
    }

    /// Non-deleted records scope (soft deletes)
    pub fn not_deleted() -> FilterGroup {
        FilterGroup::And(vec![
            FilterCondition::Simple(SimpleFilter::new(
                "deleted_at".to_string(),
                FilterOperator::IsNull,
                None
            ))
        ])
    }

    /// Records owned by a specific user
    pub fn owned_by(user_id: &str) -> FilterGroup {
        FilterGroup::And(vec![
            FilterCondition::Simple(SimpleFilter::new(
                "user_id".to_string(),
                FilterOperator::Eq,
                Some(user_id.to_string())
            ))
        ])
    }

    /// Records created in a date range
    pub fn created_between(start_date: &str, end_date: &str) -> FilterGroup {
        FilterGroup::And(vec![
            FilterCondition::Simple(SimpleFilter::new(
                "created_at".to_string(),
                FilterOperator::Gte,
                Some(start_date.to_string())
            )),
            FilterCondition::Simple(SimpleFilter::new(
                "created_at".to_string(),
                FilterOperator::Lte,
                Some(end_date.to_string())
            ))
        ])
    }

    /// Records with specific status values
    pub fn with_status(statuses: Vec<&str>) -> FilterGroup {
        FilterGroup::And(vec![
            FilterCondition::Simple(SimpleFilter::new(
                "status".to_string(),
                FilterOperator::In,
                Some(statuses.join(","))
            ))
        ])
    }
}

/// Dynamic scope resolver that can combine multiple scopes
pub struct ScopeResolver {
    scopes: HashMap<String, FilterGroup>,
}

impl ScopeResolver {
    pub fn new() -> Self {
        Self {
            scopes: HashMap::new(),
        }
    }

    /// Add a predefined scope
    pub fn add_scope(mut self, name: String, filter_group: FilterGroup) -> Self {
        self.scopes.insert(name, filter_group);
        self
    }

    /// Add multiple scopes at once
    pub fn add_scopes(mut self, scopes: HashMap<String, FilterGroup>) -> Self {
        self.scopes.extend(scopes);
        self
    }

    /// Resolve scope names to a combined filter group
    pub fn resolve(&self, scope_names: &[String]) -> Option<FilterGroup> {
        let mut resolved_scopes = Vec::new();

        for scope_name in scope_names {
            if let Some(scope) = self.scopes.get(scope_name) {
                resolved_scopes.push(FilterCondition::Group(scope.clone()));
            }
        }

        if resolved_scopes.is_empty() {
            None
        } else if resolved_scopes.len() == 1 {
            if let FilterCondition::Group(group) = &resolved_scopes[0] {
                Some(group.clone())
            } else {
                None
            }
        } else {
            Some(FilterGroup::And(resolved_scopes))
        }
    }

    /// Get all available scope names
    pub fn available_scopes(&self) -> Vec<&String> {
        self.scopes.keys().collect()
    }

    /// Check if a scope exists
    pub fn has_scope(&self, name: &str) -> bool {
        self.scopes.contains_key(name)
    }
}

impl Default for ScopeResolver {
    fn default() -> Self {
        let resolver = Self::new();

        // Add common scopes
        let mut common_scopes = HashMap::new();
        common_scopes.insert("active".to_string(), CommonScopes::active());
        common_scopes.insert("recent".to_string(), CommonScopes::recent());
        common_scopes.insert("published".to_string(), CommonScopes::published());
        common_scopes.insert("archived".to_string(), CommonScopes::archived());
        common_scopes.insert("not_deleted".to_string(), CommonScopes::not_deleted());

        resolver.add_scopes(common_scopes)
    }
}

/// Builder pattern for creating complex scoped queries
pub struct ScopedQueryBuilder {
    resolver: ScopeResolver,
    applied_scopes: Vec<String>,
    custom_filters: Option<FilterGroup>,
}

impl ScopedQueryBuilder {
    pub fn new() -> Self {
        Self {
            resolver: ScopeResolver::default(),
            applied_scopes: Vec::new(),
            custom_filters: None,
        }
    }

    pub fn with_resolver(resolver: ScopeResolver) -> Self {
        Self {
            resolver,
            applied_scopes: Vec::new(),
            custom_filters: None,
        }
    }

    /// Apply a scope
    pub fn scope(mut self, scope_name: String) -> Self {
        self.applied_scopes.push(scope_name);
        self
    }

    /// Apply multiple scopes
    pub fn scopes(mut self, scope_names: Vec<String>) -> Self {
        self.applied_scopes.extend(scope_names);
        self
    }

    /// Add custom filters alongside scopes
    pub fn with_filters(mut self, filters: FilterGroup) -> Self {
        self.custom_filters = Some(filters);
        self
    }

    /// Build the final filter group
    pub fn build(self) -> Option<FilterGroup> {
        let mut conditions = Vec::new();

        // Add resolved scopes
        if let Some(scopes_filter) = self.resolver.resolve(&self.applied_scopes) {
            conditions.push(FilterCondition::Group(scopes_filter));
        }

        // Add custom filters
        if let Some(custom) = self.custom_filters {
            conditions.push(FilterCondition::Group(custom));
        }

        if conditions.is_empty() {
            None
        } else if conditions.len() == 1 {
            if let FilterCondition::Group(group) = &conditions[0] {
                Some(group.clone())
            } else {
                None
            }
        } else {
            Some(FilterGroup::And(conditions))
        }
    }
}

impl Default for ScopedQueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_scopes() {
        let active_scope = CommonScopes::active();

        // Should create proper filter group
        match active_scope {
            FilterGroup::And(conditions) => {
                assert_eq!(conditions.len(), 1);
                if let FilterCondition::Simple(filter) = &conditions[0] {
                    assert_eq!(filter.field, "status");
                    assert_eq!(filter.operator, FilterOperator::Eq);
                    assert_eq!(filter.value, Some("active".to_string()));
                }
            },
            _ => panic!("Expected And group"),
        }
    }

    #[test]
    fn test_scope_resolver() {
        let resolver = ScopeResolver::default();

        assert!(resolver.has_scope("active"));
        assert!(resolver.has_scope("recent"));
        assert!(!resolver.has_scope("nonexistent"));

        let resolved = resolver.resolve(&["active".to_string()]);
        assert!(resolved.is_some());
    }

    #[test]
    fn test_scoped_query_builder() {
        let builder = ScopedQueryBuilder::new()
            .scope("active".to_string())
            .scope("recent".to_string());

        let result = builder.build();
        assert!(result.is_some());

        if let Some(FilterGroup::And(conditions)) = result {
            // Should have combined both scopes
            assert!(!conditions.is_empty());
        }
    }

    #[test]
    fn test_owned_by_scope() {
        let scope = CommonScopes::owned_by("user123");

        match scope {
            FilterGroup::And(conditions) => {
                assert_eq!(conditions.len(), 1);
                if let FilterCondition::Simple(filter) = &conditions[0] {
                    assert_eq!(filter.field, "user_id");
                    assert_eq!(filter.value, Some("user123".to_string()));
                }
            },
            _ => panic!("Expected And group"),
        }
    }
}