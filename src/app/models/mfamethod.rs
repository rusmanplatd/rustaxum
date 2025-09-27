use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::{HasModelType, DieselUlid};
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::mfa_methods)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaMethod {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub method_type: String,
    pub secret: Option<String>,
    pub is_enabled: bool,
    pub is_verified: bool,
    pub backup_codes: Option<serde_json::Value>,
    pub recovery_codes_used_count: i32,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::mfa_methods)]
pub struct NewMfaMethod {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub method_type: String,
    pub secret: Option<String>,
    pub is_enabled: bool,
    pub is_verified: bool,
    pub backup_codes: Option<serde_json::Value>,
    pub recovery_codes_used_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MfaSetupRequest {
    pub method_type: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MfaVerifyRequest {
    pub code: String,
    pub method_type: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MfaGenerateBackupCodesRequest {
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MfaSetupResponse {
    pub secret: String,
    pub qr_code_url: String,
    pub backup_codes: Option<Vec<String>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MfaMethodResponse {
    pub id: DieselUlid,
    pub method_type: String,
    pub is_enabled: bool,
    pub is_verified: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::mfa_attempts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MfaAttempt {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub method_type: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub attempted_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::mfa_attempts)]
pub struct NewMfaAttempt {
    pub id: DieselUlid,
    pub user_id: DieselUlid,
    pub method_type: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub attempted_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl MfaMethod {
    pub fn new(user_id: DieselUlid, method_type: String) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            user_id,
            method_type,
            secret: None,
            is_enabled: false,
            is_verified: false,
            backup_codes: None,
            recovery_codes_used_count: 0,
            last_used_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    pub fn to_response(&self) -> MfaMethodResponse {
        MfaMethodResponse {
            id: self.id,
            method_type: self.method_type.clone(),
            is_enabled: self.is_enabled,
            is_verified: self.is_verified,
            last_used_at: self.last_used_at,
            created_at: self.created_at,
        }
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn delete(&mut self) {
        self.deleted_at = Some(Utc::now());
    }

    pub fn restore(&mut self) {
        self.deleted_at = None;
    }
}

impl MfaAttempt {
    pub fn new(
        user_id: DieselUlid,
        method_type: String,
        success: bool,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            user_id,
            method_type,
            ip_address,
            user_agent,
            success,
            attempted_at: now,
            created_at: now,
        }
    }
}

impl HasModelType for MfaMethod {
    fn model_type() -> &'static str {
        "MfaMethod"
    }
}

impl crate::app::query_builder::Queryable for MfaMethod {
    fn table_name() -> &'static str {
        "mfa_methods"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "method_type",
            "is_enabled",
            "is_verified",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "method_type",
            "is_enabled",
            "is_verified",
            "last_used_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "method_type",
            "is_enabled",
            "is_verified",
            "last_used_at",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec!["user"]
    }
}

impl HasId for MfaMethod {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

crate::impl_query_builder_service!(MfaMethod);