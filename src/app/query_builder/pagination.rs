use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Cursor data structure for cursor-based pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorData {
    /// Timestamp for ordering consistency
    pub timestamp: i64,
    /// Position within the current page
    pub position: u32,
    /// Page size for consistency checks
    pub per_page: u32,
}

/// Pagination type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum PaginationType {
    /// Offset-based pagination (traditional page/per_page)
    Offset,
    /// Cursor-based pagination (for better performance with large datasets)
    Cursor,
}

impl Default for PaginationType {
    fn default() -> Self {
        PaginationType::Cursor
    }
}

/// Pagination configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Pagination {
    /// Current page number (1-based) - used for offset pagination
    #[schema(example = 1, minimum = 1)]
    pub page: u32,
    /// Number of items per page
    #[schema(example = 15, minimum = 1, maximum = 100)]
    pub per_page: u32,
    /// Type of pagination to use
    #[schema(example = "cursor")]
    pub pagination_type: PaginationType,
    /// Cursor for cursor-based pagination (optional)
    pub cursor: Option<String>,
}

impl Pagination {
    /// Create new pagination with default cursor type
    pub fn new(page: u32, per_page: u32) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.clamp(1, 100),
            pagination_type: PaginationType::default(),
            cursor: None,
        }
    }

    /// Create new pagination with specific type
    pub fn new_with_type(page: u32, per_page: u32, pagination_type: PaginationType) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.clamp(1, 100),
            pagination_type,
            cursor: None,
        }
    }

    /// Create cursor-based pagination
    pub fn cursor(per_page: u32, cursor: Option<String>) -> Self {
        Self {
            page: 1, // Not used for cursor pagination
            per_page: per_page.clamp(1, 100),
            pagination_type: PaginationType::Cursor,
            cursor,
        }
    }

    /// Create offset-based pagination
    pub fn page_based(page: u32, per_page: u32) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.clamp(1, 100),
            pagination_type: PaginationType::Offset,
            cursor: None,
        }
    }

    /// Check if this is cursor-based pagination
    pub fn is_cursor(&self) -> bool {
        self.pagination_type == PaginationType::Cursor
    }

    /// Check if this is offset-based pagination
    pub fn is_offset(&self) -> bool {
        self.pagination_type == PaginationType::Offset
    }

    /// Get offset for SQL OFFSET clause
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.per_page
    }

    /// Get limit for SQL LIMIT clause
    pub fn limit(&self) -> u32 {
        self.per_page
    }

    /// Calculate pagination info from total count
    pub fn paginate<T>(&self, total: u64, data: Vec<T>) -> PaginationResult<T> {
        match self.pagination_type {
            PaginationType::Offset => self.paginate_offset(total, data),
            PaginationType::Cursor => self.paginate_cursor(data),
        }
    }

    /// Calculate offset-based pagination info
    fn paginate_offset<T>(&self, total: u64, data: Vec<T>) -> PaginationResult<T> {
        let total_pages = ((total as f64) / (self.per_page as f64)).ceil() as u32;
        let data_len = data.len() as u64;

        PaginationResult {
            data,
            pagination: PaginationInfo {
                pagination_type: PaginationType::Offset,
                current_page: Some(self.page),
                per_page: self.per_page,
                total: Some(total),
                total_pages: Some(total_pages),
                from: if total > 0 { Some(self.offset() as u64 + 1) } else { None },
                to: if total > 0 {
                    Some((self.offset() as u64 + data_len).min(total))
                } else {
                    None
                },
                has_more_pages: self.page < total_pages,
                prev_page: if self.page > 1 { Some(self.page - 1) } else { None },
                next_page: if self.page < total_pages { Some(self.page + 1) } else { None },
                prev_cursor: None,
                next_cursor: None,
                first_page_url: Some(self.build_page_url(1)),
                last_page_url: Some(self.build_page_url(total_pages)),
                prev_page_url: if self.page > 1 {
                    Some(self.build_page_url(self.page - 1))
                } else {
                    None
                },
                next_page_url: if self.page < total_pages {
                    Some(self.build_page_url(self.page + 1))
                } else {
                    None
                },
                path: "/".to_string(), // This should be set by the controller
            },
        }
    }

    /// Calculate cursor-based pagination info
    fn paginate_cursor<T>(&self, data: Vec<T>) -> PaginationResult<T> {
        // For cursor pagination, we don't know the total count
        // We determine if there are more pages by checking if we got more data than requested
        let has_more = data.len() > self.per_page as usize;
        let actual_data = if has_more {
            data.into_iter().take(self.per_page as usize).collect()
        } else {
            data
        };

        // Generate cursors based on item position and timestamp
        let next_cursor = if has_more {
            self.generate_next_cursor(&actual_data)
        } else {
            None
        };

        let prev_cursor = self.cursor.clone(); // Previous cursor from request

        PaginationResult {
            data: actual_data,
            pagination: PaginationInfo {
                pagination_type: PaginationType::Cursor,
                current_page: None,
                per_page: self.per_page,
                total: None, // Not available in cursor pagination
                total_pages: None, // Not available in cursor pagination
                from: None, // Not available in cursor pagination
                to: None, // Not available in cursor pagination
                has_more_pages: has_more,
                prev_page: None,
                next_page: None,
                prev_cursor,
                next_cursor,
                first_page_url: None, // Not applicable for cursor pagination
                last_page_url: None, // Not applicable for cursor pagination
                prev_page_url: None,
                next_page_url: None,
                path: "/".to_string(),
            },
        }
    }

    fn build_page_url(&self, page: u32) -> String {
        format!("?page={}&per_page={}", page, self.per_page)
    }

    /// Generate a cursor for the next page based on the last item in the current dataset
    fn generate_next_cursor<T>(&self, data: &[T]) -> Option<String> {
        if data.is_empty() {
            return None;
        }

        // Use timestamp-based cursor for better consistency
        let timestamp = chrono::Utc::now().timestamp_millis();
        let position = data.len();

        // Create a base64-encoded cursor containing timestamp and position
        let cursor_data = CursorData {
            timestamp,
            position: position as u32,
            per_page: self.per_page,
        };

        self.encode_cursor(&cursor_data)
    }

    /// Generate a cursor for the previous page
    fn generate_prev_cursor<T>(&self, data: &[T]) -> Option<String> {
        if data.is_empty() {
            return None;
        }

        // For previous cursor, we need to go backwards
        let timestamp = chrono::Utc::now().timestamp_millis();

        let cursor_data = CursorData {
            timestamp: timestamp - (self.per_page as i64 * 1000), // Go back in time
            position: 0,
            per_page: self.per_page,
        };

        self.encode_cursor(&cursor_data)
    }

    /// Decode a cursor string into CursorData
    pub fn decode_cursor(&self, cursor: &str) -> Option<CursorData> {
        use base64::{Engine as _, engine::general_purpose};

        // Remove any URL-safe padding characters and decode
        let cleaned_cursor = cursor.replace('-', "+").replace('_', "/");
        let decoded = general_purpose::STANDARD.decode(cleaned_cursor).ok()?;
        let cursor_str = String::from_utf8(decoded).ok()?;

        // Parse JSON cursor data
        serde_json::from_str(&cursor_str).ok()
    }

    /// Encode cursor data into a base64 string
    fn encode_cursor(&self, cursor_data: &CursorData) -> Option<String> {
        use base64::{Engine as _, engine::general_purpose};

        let json = serde_json::to_string(cursor_data).ok()?;
        let encoded = general_purpose::STANDARD.encode(json.as_bytes());

        // Make it URL-safe
        let url_safe = encoded.replace('+', "-").replace('/', "_").trim_end_matches('=').to_string();
        Some(url_safe)
    }

    /// Check if a cursor is valid
    pub fn is_valid_cursor(&self, cursor: &str) -> bool {
        self.decode_cursor(cursor).is_some()
    }

    /// Get cursor metadata for debugging
    pub fn cursor_info(&self, cursor: &str) -> Option<CursorData> {
        self.decode_cursor(cursor)
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            per_page: 15,
            pagination_type: PaginationType::default(),
            cursor: None,
        }
    }
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginationInfo {
    /// Pagination type being used
    #[schema(example = "cursor")]
    pub pagination_type: PaginationType,
    /// Current page number (offset pagination only)
    #[schema(example = 1)]
    pub current_page: Option<u32>,
    /// Items per page
    #[schema(example = 15)]
    pub per_page: u32,
    /// Total number of items (offset pagination only)
    #[schema(example = 150)]
    pub total: Option<u64>,
    /// Total number of pages (offset pagination only)
    #[schema(example = 10)]
    pub total_pages: Option<u32>,
    /// First item number on current page (offset pagination only)
    #[schema(example = 1)]
    pub from: Option<u64>,
    /// Last item number on current page (offset pagination only)
    #[schema(example = 15)]
    pub to: Option<u64>,
    /// Whether there are more pages
    #[schema(example = true)]
    pub has_more_pages: bool,
    /// Previous page number (offset pagination only)
    #[schema(example = 1)]
    pub prev_page: Option<u32>,
    /// Next page number (offset pagination only)
    #[schema(example = 3)]
    pub next_page: Option<u32>,
    /// Previous cursor (cursor pagination only)
    #[schema(example = "prev_cursor_value")]
    pub prev_cursor: Option<String>,
    /// Next cursor (cursor pagination only)
    #[schema(example = "next_cursor_value")]
    pub next_cursor: Option<String>,
    /// First page URL
    #[schema(example = "?page=1&per_page=15")]
    pub first_page_url: Option<String>,
    /// Last page URL
    #[schema(example = "?page=10&per_page=15")]
    pub last_page_url: Option<String>,
    /// Previous page URL
    #[schema(example = "?page=1&per_page=15")]
    pub prev_page_url: Option<String>,
    /// Next page URL
    #[schema(example = "?page=3&per_page=15")]
    pub next_page_url: Option<String>,
    /// Base path for URLs
    #[schema(example = "/api/users")]
    pub path: String,
}

/// Paginated result wrapper
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PaginationResult<T> {
    /// The data items for this page
    pub data: Vec<T>,
    /// Pagination metadata
    pub pagination: PaginationInfo,
}

impl<T> PaginationResult<T> {
    /// Create a new paginated result
    pub fn new(data: Vec<T>, pagination: PaginationInfo) -> Self {
        Self { data, pagination }
    }

    /// Set the base path for pagination URLs
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        let path = path.into();
        self.pagination.path = path.clone();

        // Update URLs with the correct path
        let base_url = format!("{}?", path);

        self.pagination.first_page_url = Some(format!("{}page=1&per_page={}",
            base_url, self.pagination.per_page));
        self.pagination.last_page_url = Some(format!("{}page={}&per_page={}",
            base_url, self.pagination.total_pages.unwrap_or(1), self.pagination.per_page));

        if let Some(prev_page) = self.pagination.prev_page {
            self.pagination.prev_page_url = Some(format!("{}page={}&per_page={}",
                base_url, prev_page, self.pagination.per_page));
        }

        if let Some(next_page) = self.pagination.next_page {
            self.pagination.next_page_url = Some(format!("{}page={}&per_page={}",
                base_url, next_page, self.pagination.per_page));
        }

        self
    }

    /// Transform the data while keeping pagination info
    pub fn map<U, F>(self, f: F) -> PaginationResult<U>
    where
        F: FnOnce(Vec<T>) -> Vec<U>,
    {
        PaginationResult {
            data: f(self.data),
            pagination: self.pagination,
        }
    }

    /// Get the length of data
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if data is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_new() {
        let pagination = Pagination::new(2, 10);
        assert_eq!(pagination.page, 2);
        assert_eq!(pagination.per_page, 10);
        assert_eq!(pagination.offset(), 10);
        assert_eq!(pagination.limit(), 10);
    }

    #[test]
    fn test_pagination_bounds() {
        let pagination = Pagination::new(0, 0);
        assert_eq!(pagination.page, 1);
        assert_eq!(pagination.per_page, 1);

        let pagination = Pagination::new(5, 200);
        assert_eq!(pagination.page, 5);
        assert_eq!(pagination.per_page, 100);
    }

    #[test]
    fn test_pagination_paginate() {
        let pagination = Pagination::new(2, 10);
        let data = vec![1, 2, 3, 4, 5];
        let result = pagination.paginate(25, data);

        assert_eq!(result.data, vec![1, 2, 3, 4, 5]);
        assert_eq!(result.pagination.current_page, 2);
        assert_eq!(result.pagination.per_page, 10);
        assert_eq!(result.pagination.total, 25);
        assert_eq!(result.pagination.total_pages, 3);
        assert_eq!(result.pagination.from, Some(11));
        assert_eq!(result.pagination.to, Some(15));
        assert!(result.pagination.has_more_pages);
        assert_eq!(result.pagination.prev_page, Some(1));
        assert_eq!(result.pagination.next_page, Some(3));
    }

    #[test]
    fn test_pagination_result_with_path() {
        let pagination = Pagination::new(1, 10);
        let data = vec![1, 2, 3];
        let result = pagination.paginate(25, data).with_path("/api/users");

        assert_eq!(result.pagination.path, "/api/users");
        assert!(result.pagination.first_page_url.unwrap().starts_with("/api/users?"));
        assert!(result.pagination.next_page_url.unwrap().starts_with("/api/users?"));
    }
}