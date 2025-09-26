use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::oauth_ciba_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct OAuthCibaRequest {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub auth_req_id: String,
    pub client_id: DieselUlid,
    pub user_id: Option<DieselUlid>,
    pub scope: Option<String>,
    pub binding_message: Option<String>,
    pub user_code: Option<String>,
    pub login_hint: Option<String>,
    pub login_hint_token: Option<String>,
    pub id_token_hint: Option<String>,
    pub requested_expiry: Option<i32>,
    pub status: String,
    pub notification_endpoint: Option<String>,
    pub notification_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub interval_seconds: i32,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
    pub authorized_at: Option<DateTime<Utc>>,
    pub denied_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CibaStatus {
    Pending,
    Authorized,
    Denied,
    Expired,
    Consumed,
}

impl From<String> for CibaStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "pending" => CibaStatus::Pending,
            "authorized" => CibaStatus::Authorized,
            "denied" => CibaStatus::Denied,
            "expired" => CibaStatus::Expired,
            "consumed" => CibaStatus::Consumed,
            _ => CibaStatus::Pending,
        }
    }
}

impl From<CibaStatus> for String {
    fn from(status: CibaStatus) -> Self {
        match status {
            CibaStatus::Pending => "pending".to_string(),
            CibaStatus::Authorized => "authorized".to_string(),
            CibaStatus::Denied => "denied".to_string(),
            CibaStatus::Expired => "expired".to_string(),
            CibaStatus::Consumed => "consumed".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateOAuthCibaRequest {
    pub client_id: DieselUlid,
    pub scope: Option<String>,
    pub binding_message: Option<String>,
    pub user_code: Option<String>,
    pub login_hint: Option<String>,
    pub login_hint_token: Option<String>,
    pub id_token_hint: Option<String>,
    pub requested_expiry: Option<i32>,
    pub notification_endpoint: Option<String>,
    pub notification_token: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::oauth_ciba_requests)]
pub struct NewOAuthCibaRequest {
    pub id: DieselUlid,
    pub auth_req_id: String,
    pub client_id: DieselUlid,
    pub user_id: Option<DieselUlid>,
    pub scope: Option<String>,
    pub binding_message: Option<String>,
    pub user_code: Option<String>,
    pub login_hint: Option<String>,
    pub login_hint_token: Option<String>,
    pub id_token_hint: Option<String>,
    pub requested_expiry: Option<i32>,
    pub status: String,
    pub notification_endpoint: Option<String>,
    pub notification_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub interval_seconds: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub authorized_at: Option<DateTime<Utc>>,
    pub denied_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OAuthCibaRequestResponse {
    pub id: DieselUlid,
    pub auth_req_id: String,
    pub client_id: DieselUlid,
    pub user_id: Option<DieselUlid>,
    pub scope: Option<String>,
    pub binding_message: Option<String>,
    pub user_code: Option<String>,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub interval_seconds: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub authorized_at: Option<DateTime<Utc>>,
    pub denied_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CibaAuthenticationResponse {
    pub auth_req_id: String,
    pub expires_in: i64,
    pub interval: i32,
}

impl OAuthCibaRequest {
    pub fn new(
        client_id: DieselUlid,
        scope: Option<String>,
        binding_message: Option<String>,
        user_code: Option<String>,
        login_hint: Option<String>,
        login_hint_token: Option<String>,
        id_token_hint: Option<String>,
        requested_expiry: Option<i32>,
        notification_endpoint: Option<String>,
        notification_token: Option<String>,
    ) -> NewOAuthCibaRequest {
        let now = Utc::now();
        let expires_in_seconds = requested_expiry.unwrap_or(600); // Default 10 minutes
        let expires_at = now + chrono::Duration::seconds(expires_in_seconds as i64);
        let auth_req_id = format!("urn:ietf:params:oauth:ciba:auth-req-id:{}", DieselUlid::new());

        NewOAuthCibaRequest {
            id: DieselUlid::new(),
            auth_req_id,
            client_id,
            user_id: None,
            scope,
            binding_message,
            user_code,
            login_hint,
            login_hint_token,
            id_token_hint,
            requested_expiry: Some(expires_in_seconds),
            status: CibaStatus::Pending.into(),
            notification_endpoint,
            notification_token,
            expires_at,
            interval_seconds: 5, // Default polling interval
            created_at: now,
            updated_at: now,
            authorized_at: None,
            denied_at: None,
        }
    }

    pub fn to_response(&self) -> OAuthCibaRequestResponse {
        OAuthCibaRequestResponse {
            id: self.id,
            auth_req_id: self.auth_req_id.clone(),
            client_id: self.client_id,
            user_id: self.user_id,
            scope: self.scope.clone(),
            binding_message: self.binding_message.clone(),
            user_code: self.user_code.clone(),
            status: self.status.clone(),
            expires_at: self.expires_at,
            interval_seconds: self.interval_seconds,
            created_at: self.created_at,
            updated_at: self.updated_at,
            authorized_at: self.authorized_at,
            denied_at: self.denied_at,
        }
    }

    pub fn to_ciba_response(&self) -> CibaAuthenticationResponse {
        let expires_in = (self.expires_at - Utc::now()).num_seconds().max(0);

        CibaAuthenticationResponse {
            auth_req_id: self.auth_req_id.clone(),
            expires_in,
            interval: self.interval_seconds,
        }
    }

    pub fn status_enum(&self) -> CibaStatus {
        self.status.clone().into()
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.status_enum(), CibaStatus::Pending)
    }

    pub fn is_authorized(&self) -> bool {
        matches!(self.status_enum(), CibaStatus::Authorized)
    }

    pub fn is_denied(&self) -> bool {
        matches!(self.status_enum(), CibaStatus::Denied)
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at || matches!(self.status_enum(), CibaStatus::Expired)
    }

    pub fn is_consumed(&self) -> bool {
        matches!(self.status_enum(), CibaStatus::Consumed)
    }

    pub fn can_be_polled(&self) -> bool {
        self.is_pending() && !self.is_expired()
    }

    pub fn authorize(&mut self, user_id: DieselUlid) {
        self.status = CibaStatus::Authorized.into();
        self.user_id = Some(user_id);
        self.authorized_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn deny(&mut self) {
        self.status = CibaStatus::Denied.into();
        self.denied_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn mark_expired(&mut self) {
        self.status = CibaStatus::Expired.into();
        self.updated_at = Utc::now();
    }

    pub fn mark_consumed(&mut self) {
        self.status = CibaStatus::Consumed.into();
        self.updated_at = Utc::now();
    }

    pub fn has_user_code(&self) -> bool {
        self.user_code.is_some()
    }

    pub fn has_binding_message(&self) -> bool {
        self.binding_message.is_some()
    }

    pub fn supports_push_notification(&self) -> bool {
        self.notification_endpoint.is_some()
    }
}

impl HasId for OAuthCibaRequest {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for OAuthCibaRequest {
    fn table_name() -> &'static str {
        "oauth_ciba_requests"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "auth_req_id",
            "client_id",
            "user_id",
            "status",
            "expires_at",
            "created_at",
            "updated_at",
            "authorized_at",
            "denied_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "client_id",
            "status",
            "expires_at",
            "created_at",
            "updated_at",
            "authorized_at",
            "denied_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "auth_req_id",
            "client_id",
            "user_id",
            "scope",
            "binding_message",
            "user_code",
            "status",
            "expires_at",
            "interval_seconds",
            "created_at",
            "updated_at",
            "authorized_at",
            "denied_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "client",
            "user",
        ]
    }
}

crate::impl_query_builder_service!(OAuthCibaRequest);