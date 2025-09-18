use serde::{Deserialize, Serialize};
use std::fmt;

/// Sort direction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    /// Parse direction from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "desc" | "descending" | "-" => Self::Desc,
            _ => Self::Asc,
        }
    }

    /// Convert to SQL string
    pub fn to_sql(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

impl fmt::Display for SortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_sql())
    }
}

/// Sort structure
#[derive(Debug, Clone)]
pub struct Sort {
    pub field: String,
    pub direction: SortDirection,
}

impl Sort {
    /// Create a new sort
    pub fn new(field: String, direction: SortDirection) -> Self {
        Self { field, direction }
    }

    /// Parse sort from string
    /// Supports formats:
    /// - "field" (defaults to ASC)
    /// - "field:asc"
    /// - "field:desc"
    /// - "-field" (DESC)
    /// - "+field" (ASC)
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim();

        // Handle prefix notation (-field, +field)
        if let Some(field) = s.strip_prefix('-') {
            return Some(Sort::new(field.to_string(), SortDirection::Desc));
        }

        if let Some(field) = s.strip_prefix('+') {
            return Some(Sort::new(field.to_string(), SortDirection::Asc));
        }

        // Handle colon notation (field:direction)
        if let Some((field, direction)) = s.split_once(':') {
            let direction = SortDirection::from_str(direction);
            return Some(Sort::new(field.to_string(), direction));
        }

        // Default to ASC
        if !s.is_empty() {
            Some(Sort::new(s.to_string(), SortDirection::Asc))
        } else {
            None
        }
    }

    /// Generate SQL ORDER BY clause for this sort
    pub fn to_sql(&self) -> String {
        format!("{} {}", self.field, self.direction.to_sql())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_from_str() {
        assert_eq!(
            Sort::from_str("name").unwrap().field,
            "name"
        );
        assert_eq!(
            Sort::from_str("name").unwrap().direction,
            SortDirection::Asc
        );

        assert_eq!(
            Sort::from_str("-name").unwrap().field,
            "name"
        );
        assert_eq!(
            Sort::from_str("-name").unwrap().direction,
            SortDirection::Desc
        );

        assert_eq!(
            Sort::from_str("name:desc").unwrap().field,
            "name"
        );
        assert_eq!(
            Sort::from_str("name:desc").unwrap().direction,
            SortDirection::Desc
        );
    }
}