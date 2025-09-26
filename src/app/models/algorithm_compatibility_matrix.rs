use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::algorithm_compatibility_matrix)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AlgorithmCompatibilityMatrix {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub encryption_algorithm_a: String,
    pub encryption_algorithm_b: String,
    pub key_exchange_algorithm_a: String,
    pub key_exchange_algorithm_b: String,
    pub is_compatible: bool,
    pub compatibility_level: String,
    pub negotiation_overhead_ms: Option<i32>,
    pub interop_test_passed: Option<bool>,
    pub tested_at: Option<DateTime<Utc>>,
    pub test_version: String,
    pub notes: Option<String>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompatibilityLevel {
    Full,        // Fully compatible, optimal performance
    Partial,     // Compatible with some limitations
    Fallback,    // Compatible via fallback mechanism
    None,        // Not compatible
}

impl From<String> for CompatibilityLevel {
    fn from(s: String) -> Self {
        match s.as_str() {
            "full" => CompatibilityLevel::Full,
            "partial" => CompatibilityLevel::Partial,
            "fallback" => CompatibilityLevel::Fallback,
            "none" => CompatibilityLevel::None,
            _ => CompatibilityLevel::None,
        }
    }
}

impl From<CompatibilityLevel> for String {
    fn from(level: CompatibilityLevel) -> Self {
        match level {
            CompatibilityLevel::Full => "full".to_string(),
            CompatibilityLevel::Partial => "partial".to_string(),
            CompatibilityLevel::Fallback => "fallback".to_string(),
            CompatibilityLevel::None => "none".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateAlgorithmCompatibilityMatrix {
    pub encryption_algorithm_a: String,
    pub encryption_algorithm_b: String,
    pub key_exchange_algorithm_a: String,
    pub key_exchange_algorithm_b: String,
    pub is_compatible: bool,
    pub compatibility_level: String,
    pub negotiation_overhead_ms: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::algorithm_compatibility_matrix)]
pub struct NewAlgorithmCompatibilityMatrix {
    pub id: DieselUlid,
    pub encryption_algorithm_a: String,
    pub encryption_algorithm_b: String,
    pub key_exchange_algorithm_a: String,
    pub key_exchange_algorithm_b: String,
    pub is_compatible: bool,
    pub compatibility_level: String,
    pub negotiation_overhead_ms: Option<i32>,
    pub interop_test_passed: Option<bool>,
    pub tested_at: Option<DateTime<Utc>>,
    pub test_version: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AlgorithmCompatibilityMatrix {
    pub fn new(
        encryption_algorithm_a: String,
        encryption_algorithm_b: String,
        key_exchange_algorithm_a: String,
        key_exchange_algorithm_b: String,
        is_compatible: bool,
        compatibility_level: CompatibilityLevel,
        negotiation_overhead_ms: Option<i32>,
        notes: Option<String>,
    ) -> NewAlgorithmCompatibilityMatrix {
        let now = Utc::now();
        NewAlgorithmCompatibilityMatrix {
            id: DieselUlid::new(),
            encryption_algorithm_a,
            encryption_algorithm_b,
            key_exchange_algorithm_a,
            key_exchange_algorithm_b,
            is_compatible,
            compatibility_level: compatibility_level.into(),
            negotiation_overhead_ms,
            interop_test_passed: None,
            tested_at: None,
            test_version: "1.0".to_string(),
            notes,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn compatibility_level_enum(&self) -> CompatibilityLevel {
        self.compatibility_level.clone().into()
    }

    pub fn is_fully_compatible(&self) -> bool {
        self.is_compatible && matches!(self.compatibility_level_enum(), CompatibilityLevel::Full)
    }

    pub fn has_acceptable_overhead(&self) -> bool {
        if let Some(overhead) = self.negotiation_overhead_ms {
            overhead < 1000 // Less than 1 second overhead is acceptable
        } else {
            true // Unknown overhead is assumed acceptable
        }
    }

    pub fn is_tested(&self) -> bool {
        self.tested_at.is_some()
    }

    pub fn test_passed(&self) -> bool {
        self.interop_test_passed.unwrap_or(false)
    }

    pub fn mark_tested(&mut self, passed: bool, test_version: String) {
        self.interop_test_passed = Some(passed);
        self.tested_at = Some(Utc::now());
        self.test_version = test_version;
        self.updated_at = Utc::now();
    }

    pub fn get_algorithm_pair_key(&self) -> String {
        format!(
            "{}+{}:{}+{}",
            self.encryption_algorithm_a,
            self.encryption_algorithm_b,
            self.key_exchange_algorithm_a,
            self.key_exchange_algorithm_b
        )
    }

    pub fn is_symmetric_pair(&self) -> bool {
        self.encryption_algorithm_a == self.encryption_algorithm_b &&
        self.key_exchange_algorithm_a == self.key_exchange_algorithm_b
    }

    pub fn get_recommended_fallback(&self) -> Option<&'static str> {
        if self.is_compatible {
            return None;
        }

        // Provide fallback recommendations based on the algorithms
        match (self.encryption_algorithm_a.as_str(), self.encryption_algorithm_b.as_str()) {
            ("aes-256-gcm", _) | (_, "aes-256-gcm") => Some("aes-256-gcm"),
            ("chacha20-poly1305", _) | (_, "chacha20-poly1305") => Some("chacha20-poly1305"),
            ("aes-128-gcm", _) | (_, "aes-128-gcm") => Some("aes-128-gcm"),
            _ => Some("aes-256-gcm"), // Default fallback
        }
    }
}

impl HasId for AlgorithmCompatibilityMatrix {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for AlgorithmCompatibilityMatrix {
    fn table_name() -> &'static str {
        "algorithm_compatibility_matrix"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "encryption_algorithm_a",
            "encryption_algorithm_b",
            "key_exchange_algorithm_a",
            "key_exchange_algorithm_b",
            "is_compatible",
            "compatibility_level",
            "negotiation_overhead_ms",
            "interop_test_passed",
            "tested_at",
            "test_version",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "encryption_algorithm_a",
            "encryption_algorithm_b",
            "is_compatible",
            "compatibility_level",
            "negotiation_overhead_ms",
            "interop_test_passed",
            "tested_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "encryption_algorithm_a",
            "encryption_algorithm_b",
            "key_exchange_algorithm_a",
            "key_exchange_algorithm_b",
            "is_compatible",
            "compatibility_level",
            "negotiation_overhead_ms",
            "interop_test_passed",
            "tested_at",
            "test_version",
            "notes",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("compatibility_level", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![]
    }
}

crate::impl_query_builder_service!(AlgorithmCompatibilityMatrix);