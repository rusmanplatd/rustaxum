use serde::{Deserialize, Serialize};

/// Sort direction enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Asc
    }
}

impl SortDirection {
    /// Convert to SQL ORDER BY clause
    pub fn to_sql(&self) -> &'static str {
        match self {
            SortDirection::Asc => "ASC",
            SortDirection::Desc => "DESC",
        }
    }
}

/// Sort specification for a field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sort {
    /// Field name to sort by
    pub field: String,
    /// Sort direction (asc/desc)
    pub direction: SortDirection,
}

impl Sort {
    /// Create a new Sort instance
    pub fn new(field: String, direction: SortDirection) -> Self {
        Self { field, direction }
    }

    /// Create ascending sort
    pub fn asc(field: impl Into<String>) -> Self {
        Self::new(field.into(), SortDirection::Asc)
    }

    /// Create descending sort
    pub fn desc(field: impl Into<String>) -> Self {
        Self::new(field.into(), SortDirection::Desc)
    }

    /// Parse sort string into Sort instances
    /// Format: "field1,-field2,field3" where - prefix indicates descending
    pub fn from_string(sort_string: &str) -> Vec<Sort> {
        sort_string
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| {
                if s.starts_with('-') {
                    Sort::desc(&s[1..])
                } else {
                    Sort::asc(s)
                }
            })
            .collect()
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self.direction {
            SortDirection::Asc => self.field.clone(),
            SortDirection::Desc => format!("-{}", self.field),
        }
    }

    /// Convert multiple sorts to string representation
    pub fn vec_to_string(sorts: &[Sort]) -> String {
        sorts
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

impl std::fmt::Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_from_string() {
        let sorts = Sort::from_string("name,-created_at,email");
        assert_eq!(sorts.len(), 3);

        assert_eq!(sorts[0].field, "name");
        assert_eq!(sorts[0].direction, SortDirection::Asc);

        assert_eq!(sorts[1].field, "created_at");
        assert_eq!(sorts[1].direction, SortDirection::Desc);

        assert_eq!(sorts[2].field, "email");
        assert_eq!(sorts[2].direction, SortDirection::Asc);
    }

    #[test]
    fn test_sort_to_string() {
        let sort_asc = Sort::asc("name");
        let sort_desc = Sort::desc("created_at");

        assert_eq!(sort_asc.to_string(), "name");
        assert_eq!(sort_desc.to_string(), "-created_at");
    }

    #[test]
    fn test_sort_vec_to_string() {
        let sorts = vec![
            Sort::asc("name"),
            Sort::desc("created_at"),
            Sort::asc("email"),
        ];

        assert_eq!(Sort::vec_to_string(&sorts), "name,-created_at,email");
    }
}