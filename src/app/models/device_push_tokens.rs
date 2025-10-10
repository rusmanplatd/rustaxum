use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::device_push_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DevicePushToken {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub device_id: DieselUlid,
    pub platform: String,
    pub token: String,
    pub endpoint: Option<String>,
    pub encrypted_notification_settings: Option<String>,
    pub settings_algorithm: Option<String>,
    pub is_active: bool,
    pub last_used_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PushPlatform {
    Fcm,        // Firebase Cloud Messaging (Android)
    Apns,       // Apple Push Notification Service (iOS)
    WebPush,    // Web Push Protocol
    WindowsWns, // Windows Notification Service
    HuaweiHms,  // Huawei Mobile Services
    Custom,     // Custom push service
}

impl From<String> for PushPlatform {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "fcm" | "firebase" => PushPlatform::Fcm,
            "apns" | "apple" => PushPlatform::Apns,
            "webpush" | "web" => PushPlatform::WebPush,
            "wns" | "windows" => PushPlatform::WindowsWns,
            "hms" | "huawei" => PushPlatform::HuaweiHms,
            "custom" => PushPlatform::Custom,
            _ => PushPlatform::Custom,
        }
    }
}

impl From<PushPlatform> for String {
    fn from(platform: PushPlatform) -> Self {
        match platform {
            PushPlatform::Fcm => "fcm".to_string(),
            PushPlatform::Apns => "apns".to_string(),
            PushPlatform::WebPush => "webpush".to_string(),
            PushPlatform::WindowsWns => "wns".to_string(),
            PushPlatform::HuaweiHms => "hms".to_string(),
            PushPlatform::Custom => "custom".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateDevicePushToken {
    pub device_id: DieselUlid,
    pub platform: String,
    pub token: String,
    pub endpoint: Option<String>,
    pub encrypted_notification_settings: Option<String>,
    pub settings_algorithm: Option<String>,
    pub expires_in_days: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateDevicePushToken {
    pub token: Option<String>,
    pub endpoint: Option<String>,
    pub encrypted_notification_settings: Option<String>,
    pub settings_algorithm: Option<String>,
    pub is_active: Option<bool>,
    pub expires_in_days: Option<i32>,
}
#[derive(Debug, Serialize, ToSchema)]
pub struct DevicePushTokenResponse {
    pub id: DieselUlid,
    pub device_id: DieselUlid,
    pub platform: String,
    pub endpoint: Option<String>,
    pub is_active: bool,
    pub last_used_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DevicePushToken {
    pub fn new(
        device_id: DieselUlid,
        platform: PushPlatform,
        token: String,
        endpoint: Option<String>,
        encrypted_notification_settings: Option<String>,
        settings_algorithm: Option<String>,
        expires_in_days: Option<i32>,
    ) -> Self {
        let now = Utc::now();
        let expires_at = expires_in_days.map(|days| now + chrono::Duration::days(days as i64));

        DevicePushToken {
            id: DieselUlid::new(),
            device_id,
            platform: platform.into(),
            token,
            endpoint,
            encrypted_notification_settings,
            settings_algorithm,
            is_active: true,
            last_used_at: now,
            expires_at,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> DevicePushTokenResponse {
        DevicePushTokenResponse {
            id: self.id,
            device_id: self.device_id,
            platform: self.platform.clone(),
            endpoint: self.endpoint.clone(),
            is_active: self.is_active,
            last_used_at: self.last_used_at,
            expires_at: self.expires_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn platform_enum(&self) -> PushPlatform {
        self.platform.clone().into()
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() >= expires_at
        } else {
            false
        }
    }

    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    pub fn update_last_used(&mut self) {
        self.last_used_at = Utc::now();
        self.updated_at = Utc::now();
    }

    pub fn extend_expiry(&mut self, additional_days: i32) {
        if let Some(current_expiry) = self.expires_at {
            self.expires_at = Some(current_expiry + chrono::Duration::days(additional_days as i64));
        } else {
            self.expires_at = Some(Utc::now() + chrono::Duration::days(additional_days as i64));
        }
        self.updated_at = Utc::now();
    }

    pub fn supports_rich_notifications(&self) -> bool {
        matches!(self.platform_enum(), PushPlatform::Fcm | PushPlatform::Apns)
    }

    pub fn supports_web_push(&self) -> bool {
        matches!(self.platform_enum(), PushPlatform::WebPush)
    }

    pub fn get_platform_display_name(&self) -> &'static str {
        match self.platform_enum() {
            PushPlatform::Fcm => "Firebase (Android)",
            PushPlatform::Apns => "Apple Push (iOS)",
            PushPlatform::WebPush => "Web Push",
            PushPlatform::WindowsWns => "Windows",
            PushPlatform::HuaweiHms => "Huawei",
            PushPlatform::Custom => "Custom",
        }
    }
}

impl HasId for DevicePushToken {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for DevicePushToken {
    fn table_name() -> &'static str {
        "device_push_tokens"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "platform",
            "is_active",
            "last_used_at",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "platform",
            "is_active",
            "last_used_at",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "platform",
            "endpoint",
            "is_active",
            "last_used_at",
            "expires_at",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("last_used_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "device",
        ]
    }
}

crate::impl_query_builder_service!(DevicePushToken);