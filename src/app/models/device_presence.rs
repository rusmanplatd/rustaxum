use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::device_presence)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DevicePresence {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub device_id: DieselUlid,
    pub status: String,
    pub last_seen_at: DateTime<Utc>,
    pub encrypted_status_message: Option<String>,
    pub status_message_algorithm: Option<String>,
    pub auto_away_after_minutes: Option<i32>,
    pub auto_offline_after_minutes: Option<i32>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PresenceStatus {
    Online,
    Away,
    Busy,
    DoNotDisturb,
    Invisible,
    Offline,
}

impl From<String> for PresenceStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "online" => PresenceStatus::Online,
            "away" => PresenceStatus::Away,
            "busy" => PresenceStatus::Busy,
            "do_not_disturb" => PresenceStatus::DoNotDisturb,
            "invisible" => PresenceStatus::Invisible,
            "offline" => PresenceStatus::Offline,
            _ => PresenceStatus::Offline,
        }
    }
}

impl From<PresenceStatus> for String {
    fn from(status: PresenceStatus) -> Self {
        match status {
            PresenceStatus::Online => "online".to_string(),
            PresenceStatus::Away => "away".to_string(),
            PresenceStatus::Busy => "busy".to_string(),
            PresenceStatus::DoNotDisturb => "do_not_disturb".to_string(),
            PresenceStatus::Invisible => "invisible".to_string(),
            PresenceStatus::Offline => "offline".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateDevicePresence {
    pub device_id: DieselUlid,
    pub status: String,
    pub encrypted_status_message: Option<String>,
    pub status_message_algorithm: Option<String>,
    pub auto_away_after_minutes: Option<i32>,
    pub auto_offline_after_minutes: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateDevicePresence {
    pub status: Option<String>,
    pub encrypted_status_message: Option<String>,
    pub status_message_algorithm: Option<String>,
    pub auto_away_after_minutes: Option<i32>,
    pub auto_offline_after_minutes: Option<i32>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::device_presence)]
pub struct NewDevicePresence {
    pub id: DieselUlid,
    pub device_id: DieselUlid,
    pub status: String,
    pub last_seen_at: DateTime<Utc>,
    pub encrypted_status_message: Option<String>,
    pub status_message_algorithm: Option<String>,
    pub auto_away_after_minutes: Option<i32>,
    pub auto_offline_after_minutes: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DevicePresenceResponse {
    pub id: DieselUlid,
    pub device_id: DieselUlid,
    pub status: String,
    pub last_seen_at: DateTime<Utc>,
    pub encrypted_status_message: Option<String>,
    pub status_message_algorithm: Option<String>,
    pub auto_away_after_minutes: Option<i32>,
    pub auto_offline_after_minutes: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DevicePresence {
    pub fn new(
        device_id: DieselUlid,
        status: PresenceStatus,
        encrypted_status_message: Option<String>,
        status_message_algorithm: Option<String>,
        auto_away_after_minutes: Option<i32>,
        auto_offline_after_minutes: Option<i32>,
    ) -> NewDevicePresence {
        let now = Utc::now();
        NewDevicePresence {
            id: DieselUlid::new(),
            device_id,
            status: status.into(),
            last_seen_at: now,
            encrypted_status_message,
            status_message_algorithm,
            auto_away_after_minutes,
            auto_offline_after_minutes,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> DevicePresenceResponse {
        DevicePresenceResponse {
            id: self.id,
            device_id: self.device_id,
            status: self.status.clone(),
            last_seen_at: self.last_seen_at,
            encrypted_status_message: self.encrypted_status_message.clone(),
            status_message_algorithm: self.status_message_algorithm.clone(),
            auto_away_after_minutes: self.auto_away_after_minutes,
            auto_offline_after_minutes: self.auto_offline_after_minutes,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn status_enum(&self) -> PresenceStatus {
        self.status.clone().into()
    }

    pub fn is_online(&self) -> bool {
        matches!(self.status_enum(), PresenceStatus::Online)
    }

    pub fn is_away(&self) -> bool {
        matches!(self.status_enum(), PresenceStatus::Away)
    }

    pub fn is_offline(&self) -> bool {
        matches!(self.status_enum(), PresenceStatus::Offline)
    }

    pub fn is_available(&self) -> bool {
        matches!(self.status_enum(), PresenceStatus::Online | PresenceStatus::Away)
    }

    pub fn is_do_not_disturb(&self) -> bool {
        matches!(self.status_enum(), PresenceStatus::DoNotDisturb)
    }

    pub fn should_auto_away(&self) -> bool {
        if let Some(minutes) = self.auto_away_after_minutes {
            let threshold = Utc::now() - chrono::Duration::minutes(minutes as i64);
            self.last_seen_at < threshold && self.is_online()
        } else {
            false
        }
    }

    pub fn should_auto_offline(&self) -> bool {
        if let Some(minutes) = self.auto_offline_after_minutes {
            let threshold = Utc::now() - chrono::Duration::minutes(minutes as i64);
            self.last_seen_at < threshold && self.is_available()
        } else {
            false
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen_at = Utc::now();
        self.updated_at = Utc::now();
    }

    pub fn set_status(&mut self, status: PresenceStatus) {
        self.status = status.into();
        self.last_seen_at = Utc::now();
        self.updated_at = Utc::now();
    }

    pub fn set_status_message(&mut self, encrypted_message: Option<String>, algorithm: Option<String>) {
        self.encrypted_status_message = encrypted_message;
        self.status_message_algorithm = algorithm;
        self.updated_at = Utc::now();
    }
}

impl HasId for DevicePresence {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for DevicePresence {
    fn table_name() -> &'static str {
        "device_presence"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "status",
            "last_seen_at",
            "auto_away_after_minutes",
            "auto_offline_after_minutes",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "status",
            "last_seen_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "status",
            "last_seen_at",
            "encrypted_status_message",
            "status_message_algorithm",
            "auto_away_after_minutes",
            "auto_offline_after_minutes",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("last_seen_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "device",
        ]
    }
}

crate::impl_query_builder_service!(DevicePresence);