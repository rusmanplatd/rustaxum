use std::collections::HashSet;

/// Field selector for controlling which fields to include in the response
#[derive(Debug, Clone)]
pub struct FieldSelector {
    pub fields: HashSet<String>,
    pub allowed_fields: HashSet<String>,
}

impl FieldSelector {
    /// Create a new field selector
    pub fn new(fields: Vec<String>, allowed_fields: Vec<String>) -> Self {
        let fields: HashSet<String> = fields.into_iter().collect();
        let allowed_fields: HashSet<String> = allowed_fields.into_iter().map(|s| s.to_string()).collect();

        Self {
            fields,
            allowed_fields,
        }
    }

    /// Get the validated fields to select
    pub fn get_validated_fields(&self) -> Vec<String> {
        if self.fields.is_empty() {
            // If no fields specified, return all allowed fields
            self.allowed_fields.iter().cloned().collect()
        } else {
            // Return only the intersection of requested and allowed fields
            self.fields
                .intersection(&self.allowed_fields)
                .cloned()
                .collect()
        }
    }

    /// Generate SQL SELECT clause
    pub fn to_sql(&self) -> String {
        let fields = self.get_validated_fields();

        if fields.is_empty() {
            "*".to_string()
        } else {
            fields.join(", ")
        }
    }

    /// Check if a field is selected
    pub fn is_selected(&self, field: &str) -> bool {
        if self.fields.is_empty() {
            // If no specific fields requested, all allowed fields are selected
            self.allowed_fields.contains(field)
        } else {
            self.fields.contains(field) && self.allowed_fields.contains(field)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_selector() {
        let allowed = vec!["id".to_string(), "name".to_string(), "email".to_string()];

        // Test with specific fields
        let selector = FieldSelector::new(vec!["name".to_string(), "email".to_string()], allowed.clone());
        let fields = selector.get_validated_fields();
        assert!(fields.contains(&"name".to_string()));
        assert!(fields.contains(&"email".to_string()));
        assert!(!fields.contains(&"id".to_string()));

        // Test with empty fields (should return all allowed)
        let selector = FieldSelector::new(vec![], allowed.clone());
        let fields = selector.get_validated_fields();
        assert_eq!(fields.len(), 3);

        // Test with invalid field
        let selector = FieldSelector::new(vec!["invalid".to_string()], allowed);
        let fields = selector.get_validated_fields();
        assert!(fields.is_empty());
    }
}