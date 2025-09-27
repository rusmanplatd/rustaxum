use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT claims for secure cursor data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorClaims {
    /// Cursor data
    pub cursor: CursorData,
    /// Issued at timestamp
    pub iat: u64,
    /// Expiration timestamp (issued + 1 hour)
    pub exp: u64,
    /// JWT issuer
    pub iss: String,
}

impl CursorClaims {
    /// Create new cursor claims with 1 hour expiration
    pub fn new(cursor: CursorData) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            cursor,
            iat: now,
            exp: now + 3600, // 1 hour expiration
            iss: "rustaxum-pagination".to_string(),
        }
    }

    /// Check if the cursor claims are expired
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= self.exp
    }
}

/// Cursor data structure for cursor-based pagination
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CursorData {
    /// Timestamp for ordering consistency
    pub timestamp: i64,
    /// Position within the current page
    pub position: u32,
    /// Page size for consistency checks
    pub per_page: u32,
}

impl CursorData {
    /// Create new cursor data
    pub fn new(timestamp: i64, position: u32, per_page: u32) -> Self {
        Self {
            timestamp,
            position,
            per_page,
        }
    }

    /// Create cursor data for current time
    pub fn now(position: u32, per_page: u32) -> Self {
        Self::new(chrono::Utc::now().timestamp_millis(), position, per_page)
    }

    /// Get the timestamp as a DateTime
    pub fn datetime(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp_millis(self.timestamp)
            .unwrap_or_else(chrono::Utc::now)
    }

    /// Check if this cursor is expired (older than a certain duration)
    pub fn is_expired(&self, max_age_seconds: i64) -> bool {
        let now = chrono::Utc::now().timestamp();
        let cursor_time = self.timestamp / 1000; // Convert millis to seconds
        (now - cursor_time) > max_age_seconds
    }

    /// Validate cursor data consistency
    pub fn is_valid(&self) -> bool {
        self.per_page > 0 && self.per_page <= 100 && self.timestamp > 0
    }
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

impl PaginationType {
    /// Create pagination type from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "cursor" => Ok(PaginationType::Cursor),
            "offset" => Ok(PaginationType::Offset),
            _ => Err(format!("Invalid pagination type: {}. Must be 'cursor' or 'offset'", s)),
        }
    }

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            PaginationType::Cursor => "cursor",
            PaginationType::Offset => "offset",
        }
    }

    /// Check if this is cursor-based pagination
    pub fn is_cursor(&self) -> bool {
        matches!(self, PaginationType::Cursor)
    }

    /// Check if this is offset-based pagination
    pub fn is_offset(&self) -> bool {
        matches!(self, PaginationType::Offset)
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
    /// Get JWT secret for cursor signing (from environment or default)
    fn get_jwt_secret() -> String {
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_cursor_secret_change_in_production".to_string())
    }

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

        let prev_cursor = self.generate_prev_cursor(&actual_data);

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

    /// Generate a cursor from a specific position
    pub fn generate_cursor_from_position(&self, position: u32) -> Option<String> {
        let timestamp = chrono::Utc::now().timestamp_millis();
        let cursor_data = CursorData {
            timestamp,
            position,
            per_page: self.per_page,
        };
        self.encode_cursor(&cursor_data)
    }

    /// Get the SQL WHERE clause for cursor-based pagination
    /// Supports custom sort fields for more flexible cursor pagination
    pub fn cursor_where_clause(&self) -> Option<(String, Vec<i64>)> {
        self.cursor_where_clause_with_sort("created_at", "id")
    }

    /// Get the SQL WHERE clause for cursor-based pagination with custom sort fields
    pub fn cursor_where_clause_with_sort(&self, sort_field: &str, id_field: &str) -> Option<(String, Vec<i64>)> {
        if let Some(ref cursor_str) = self.cursor {
            if let Some(cursor_data) = self.decode_cursor(cursor_str) {
                // Support flexible sort fields for cursor pagination
                return Some((
                    format!("{} > $1 OR ({} = $1 AND {} > $2)", sort_field, sort_field, id_field),
                    vec![cursor_data.timestamp, cursor_data.position as i64],
                ));
            }
        }
        None
    }

    /// Get WHERE clause for multi-column cursor pagination with proper tuple comparison
    pub fn multi_column_cursor_where_clause(&self, sort_columns: &[(&str, bool)]) -> Option<(String, Vec<String>)> {
        if let Some(ref cursor_str) = self.cursor {
            if let Some(cursor_data) = self.decode_cursor(cursor_str) {
                if sort_columns.is_empty() {
                    return None;
                }

                // For multi-column sorting, we need to build a proper tuple comparison
                // that ensures correct lexicographic ordering across all columns
                let mut conditions = Vec::new();
                let mut values = Vec::new();

                // Build progressive conditions for each column combination
                for i in 0..sort_columns.len() {
                    let mut equality_conditions = Vec::new();

                    // Add equality conditions for all previous columns
                    for j in 0..i {
                        let (column, _) = sort_columns[j];
                        equality_conditions.push(format!("{} = ${}", column, values.len() + 1));
                        // Use cursor timestamp as a fallback value for previous columns
                        values.push(cursor_data.timestamp.to_string());
                    }

                    // Add the comparison condition for the current column
                    let (current_column, is_desc) = sort_columns[i];
                    let operator = if is_desc { "<" } else { ">" };
                    let final_condition = format!("{} {} ${}", current_column, operator, values.len() + 1);

                    // Use position for the final column comparison
                    if i == sort_columns.len() - 1 {
                        values.push(cursor_data.position.to_string());
                    } else {
                        values.push(cursor_data.timestamp.to_string());
                    }

                    // Combine equality conditions with final comparison
                    if equality_conditions.is_empty() {
                        conditions.push(final_condition);
                    } else {
                        conditions.push(format!("({} AND {})",
                            equality_conditions.join(" AND "),
                            final_condition));
                    }
                }

                let where_clause = format!("({})", conditions.join(" OR "));
                return Some((where_clause, values));
            }
        }
        None
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

    /// Decode a JWT cursor string into CursorData
    pub fn decode_cursor(&self, cursor: &str) -> Option<CursorData> {
        let secret = Self::get_jwt_secret();
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&["rustaxum-pagination"]);

        match decode::<CursorClaims>(cursor, &DecodingKey::from_secret(secret.as_ref()), &validation) {
            Ok(token_data) => {
                let claims = token_data.claims;
                // Check if the cursor is expired
                if claims.is_expired() {
                    return None;
                }
                Some(claims.cursor)
            }
            Err(_) => None,
        }
    }

    /// Encode cursor data into a JWT token
    fn encode_cursor(&self, cursor_data: &CursorData) -> Option<String> {
        let claims = CursorClaims::new(cursor_data.clone());
        let header = Header::new(Algorithm::HS256);
        let secret = Self::get_jwt_secret();

        match encode(&header, &claims, &EncodingKey::from_secret(secret.as_ref())) {
            Ok(token) => Some(token),
            Err(_) => None,
        }
    }

    /// Check if a JWT cursor is valid and not expired
    pub fn is_valid_cursor(&self, cursor: &str) -> bool {
        self.decode_cursor(cursor).is_some()
    }

    /// Validate JWT cursor signature and expiration
    pub fn validate_cursor_jwt(&self, cursor: &str) -> Result<CursorClaims, String> {
        let secret = Self::get_jwt_secret();
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&["rustaxum-pagination"]);

        match decode::<CursorClaims>(cursor, &DecodingKey::from_secret(secret.as_ref()), &validation) {
            Ok(token_data) => {
                let claims = token_data.claims;
                if claims.is_expired() {
                    return Err("Cursor has expired".to_string());
                }
                Ok(claims)
            }
            Err(e) => Err(format!("Invalid cursor JWT: {}", e)),
        }
    }

    /// Get cursor metadata for debugging (includes JWT claims)
    pub fn cursor_info(&self, cursor: &str) -> Option<CursorClaims> {
        let secret = Self::get_jwt_secret();
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&["rustaxum-pagination"]);

        decode::<CursorClaims>(cursor, &DecodingKey::from_secret(secret.as_ref()), &validation)
            .map(|token_data| token_data.claims)
            .ok()
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
        assert_eq!(result.pagination.current_page, Some(2));
        assert_eq!(result.pagination.per_page, 10);
        assert_eq!(result.pagination.total, Some(25));
        assert_eq!(result.pagination.total_pages, Some(3));
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

    #[test]
    fn test_pagination_type_from_str() {
        assert_eq!(PaginationType::from_str("cursor").unwrap(), PaginationType::Cursor);
        assert_eq!(PaginationType::from_str("offset").unwrap(), PaginationType::Offset);
        assert_eq!(PaginationType::from_str("CURSOR").unwrap(), PaginationType::Cursor);
        assert_eq!(PaginationType::from_str("OFFSET").unwrap(), PaginationType::Offset);
        assert!(PaginationType::from_str("invalid").is_err());
    }

    #[test]
    fn test_pagination_type_as_str() {
        assert_eq!(PaginationType::Cursor.as_str(), "cursor");
        assert_eq!(PaginationType::Offset.as_str(), "offset");
    }

    #[test]
    fn test_pagination_type_checks() {
        assert!(PaginationType::Cursor.is_cursor());
        assert!(!PaginationType::Cursor.is_offset());
        assert!(!PaginationType::Offset.is_cursor());
        assert!(PaginationType::Offset.is_offset());
    }

    #[test]
    fn test_cursor_data_creation() {
        let cursor_data = CursorData::new(1000, 5, 20);
        assert_eq!(cursor_data.timestamp, 1000);
        assert_eq!(cursor_data.position, 5);
        assert_eq!(cursor_data.per_page, 20);
    }

    #[test]
    fn test_cursor_data_now() {
        let cursor_data = CursorData::now(10, 15);
        assert_eq!(cursor_data.position, 10);
        assert_eq!(cursor_data.per_page, 15);
        assert!(cursor_data.timestamp > 0);
    }

    #[test]
    fn test_cursor_data_validation() {
        let valid_cursor = CursorData::new(1000, 5, 20);
        assert!(valid_cursor.is_valid());

        let invalid_cursor_per_page = CursorData::new(1000, 5, 0);
        assert!(!invalid_cursor_per_page.is_valid());

        let invalid_cursor_timestamp = CursorData::new(0, 5, 20);
        assert!(!invalid_cursor_timestamp.is_valid());

        let invalid_cursor_per_page_too_large = CursorData::new(1000, 5, 200);
        assert!(!invalid_cursor_per_page_too_large.is_valid());
    }

    #[test]
    fn test_cursor_data_expiration() {
        let old_timestamp = chrono::Utc::now().timestamp_millis() - 7200000; // 2 hours ago
        let old_cursor = CursorData::new(old_timestamp, 5, 20);
        assert!(old_cursor.is_expired(3600)); // 1 hour max age

        let recent_cursor = CursorData::now(5, 20);
        assert!(!recent_cursor.is_expired(3600)); // 1 hour max age
    }

    #[test]
    fn test_cursor_based_pagination() {
        let pagination = Pagination::cursor(20, Some("test_cursor".to_string()));
        assert_eq!(pagination.pagination_type, PaginationType::Cursor);
        assert_eq!(pagination.per_page, 20);
        assert_eq!(pagination.cursor, Some("test_cursor".to_string()));
        assert!(pagination.is_cursor());
        assert!(!pagination.is_offset());
    }

    #[test]
    fn test_offset_based_pagination() {
        let pagination = Pagination::page_based(3, 25);
        assert_eq!(pagination.pagination_type, PaginationType::Offset);
        assert_eq!(pagination.page, 3);
        assert_eq!(pagination.per_page, 25);
        assert_eq!(pagination.offset(), 50); // (3-1) * 25
        assert_eq!(pagination.limit(), 25);
        assert!(!pagination.is_cursor());
        assert!(pagination.is_offset());
    }

    #[test]
    fn test_jwt_cursor_encode_decode() {
        let pagination = Pagination::cursor(20, None);
        let cursor_data = CursorData::now(5, 20);

        // Test JWT encoding
        let encoded = pagination.encode_cursor(&cursor_data);
        assert!(encoded.is_some());

        // Test JWT decoding
        if let Some(encoded_cursor) = encoded {
            let decoded = pagination.decode_cursor(&encoded_cursor);
            assert!(decoded.is_some());

            let decoded_data = decoded.unwrap();
            assert_eq!(decoded_data.position, cursor_data.position);
            assert_eq!(decoded_data.per_page, cursor_data.per_page);

            // Test JWT validation
            let validation_result = pagination.validate_cursor_jwt(&encoded_cursor);
            assert!(validation_result.is_ok());

            let claims = validation_result.unwrap();
            assert_eq!(claims.iss, "rustaxum-pagination");
            assert!(claims.exp > claims.iat);
        }
    }

    #[test]
    fn test_jwt_cursor_validation() {
        let pagination = Pagination::cursor(20, None);
        let cursor_data = CursorData::now(5, 20);
        let encoded = pagination.encode_cursor(&cursor_data).unwrap();

        // Valid JWT cursor should pass validation
        assert!(pagination.is_valid_cursor(&encoded));

        // Invalid/tampered cursor should fail validation
        assert!(!pagination.is_valid_cursor("invalid_cursor"));
        assert!(!pagination.is_valid_cursor("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJjdXJzb3IiOnsidGltZXN0YW1wIjoxNjQwOTk1MjAwMDAwLCJwb3NpdGlvbiI6MTAsInBlcl9wYWdlIjoyMH0sImlhdCI6MTY0MDk5NTIwMCwiZXhwIjoxNjQwOTk4ODAwLCJpc3MiOiJydXN0YXh1bS1wYWdpbmF0aW9uIn0.TAMPERED_SIGNATURE"));
    }

    #[test]
    fn test_jwt_cursor_expiration() {
        let pagination = Pagination::cursor(20, None);

        // Create an expired cursor (set timestamp to past)
        let old_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - 7200; // 2 hours ago

        let expired_claims = CursorClaims {
            cursor: CursorData::new(1640995200000, 10, 20),
            iat: old_timestamp,
            exp: old_timestamp + 3600, // Expired 1 hour ago
            iss: "rustaxum-pagination".to_string(),
        };

        // Manually create JWT for expired cursor
        let header = Header::new(Algorithm::HS256);
        let secret = Pagination::get_jwt_secret();
        let expired_jwt = encode(&header, &expired_claims, &EncodingKey::from_secret(secret.as_ref())).unwrap();

        // Expired cursor should fail validation
        assert!(!pagination.is_valid_cursor(&expired_jwt));
        assert!(pagination.validate_cursor_jwt(&expired_jwt).is_err());
    }

    #[test]
    fn test_multi_column_cursor_where_clause_jwt() {
        let cursor_data = CursorData::new(1640995200000, 10, 20); // 2022-01-01 timestamp
        let pagination = Pagination::cursor(20, None);
        let encoded_cursor = pagination.encode_cursor(&cursor_data).unwrap();

        let pagination_with_cursor = Pagination::cursor(20, Some(encoded_cursor));

        // Test single column
        let sort_columns = vec![("created_at", false)];
        let result = pagination_with_cursor.multi_column_cursor_where_clause(&sort_columns);
        assert!(result.is_some());
        let (where_clause, values) = result.unwrap();
        assert!(where_clause.contains("created_at >"));
        assert_eq!(values.len(), 1);

        // Test multiple columns
        let sort_columns = vec![("created_at", false), ("id", true)];
        let result = pagination_with_cursor.multi_column_cursor_where_clause(&sort_columns);
        assert!(result.is_some());
        let (where_clause, values) = result.unwrap();
        assert!(where_clause.contains("created_at >"));
        assert!(where_clause.contains("id <"));
        assert!(where_clause.contains(" OR "));
        assert_eq!(values.len(), 2);

        // Test empty columns
        let sort_columns = vec![];
        let result = pagination_with_cursor.multi_column_cursor_where_clause(&sort_columns);
        assert!(result.is_none());
    }
}