use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::security_incidents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SecurityIncident {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub device_id: Option<DieselUlid>,
    pub user_id: Option<DieselUlid>,
    pub conversation_id: Option<DieselUlid>,
    pub incident_type: String,
    pub severity: String,
    pub encrypted_incident_data: Option<String>,
    pub incident_algorithm: Option<String>,
    pub is_resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_notes: Option<String>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncidentType {
    UnauthorizedAccess,
    KeyCompromise,
    SessionHijack,
    MessageIntercept,
    DeviceCompromise,
    ProtocolViolation,
    CryptographicFailure,
    TamperingDetected,
    ReplayAttack,
    ManInTheMiddle,
}

impl From<String> for IncidentType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "unauthorized_access" => IncidentType::UnauthorizedAccess,
            "key_compromise" => IncidentType::KeyCompromise,
            "session_hijack" => IncidentType::SessionHijack,
            "message_intercept" => IncidentType::MessageIntercept,
            "device_compromise" => IncidentType::DeviceCompromise,
            "protocol_violation" => IncidentType::ProtocolViolation,
            "cryptographic_failure" => IncidentType::CryptographicFailure,
            "tampering_detected" => IncidentType::TamperingDetected,
            "replay_attack" => IncidentType::ReplayAttack,
            "man_in_the_middle" => IncidentType::ManInTheMiddle,
            _ => IncidentType::UnauthorizedAccess,
        }
    }
}

impl From<IncidentType> for String {
    fn from(incident_type: IncidentType) -> Self {
        match incident_type {
            IncidentType::UnauthorizedAccess => "unauthorized_access".to_string(),
            IncidentType::KeyCompromise => "key_compromise".to_string(),
            IncidentType::SessionHijack => "session_hijack".to_string(),
            IncidentType::MessageIntercept => "message_intercept".to_string(),
            IncidentType::DeviceCompromise => "device_compromise".to_string(),
            IncidentType::ProtocolViolation => "protocol_violation".to_string(),
            IncidentType::CryptographicFailure => "cryptographic_failure".to_string(),
            IncidentType::TamperingDetected => "tampering_detected".to_string(),
            IncidentType::ReplayAttack => "replay_attack".to_string(),
            IncidentType::ManInTheMiddle => "man_in_the_middle".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncidentSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl From<String> for IncidentSeverity {
    fn from(s: String) -> Self {
        match s.as_str() {
            "low" => IncidentSeverity::Low,
            "medium" => IncidentSeverity::Medium,
            "high" => IncidentSeverity::High,
            "critical" => IncidentSeverity::Critical,
            _ => IncidentSeverity::Medium,
        }
    }
}

impl From<IncidentSeverity> for String {
    fn from(severity: IncidentSeverity) -> Self {
        match severity {
            IncidentSeverity::Low => "low".to_string(),
            IncidentSeverity::Medium => "medium".to_string(),
            IncidentSeverity::High => "high".to_string(),
            IncidentSeverity::Critical => "critical".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSecurityIncident {
    pub device_id: Option<DieselUlid>,
    pub user_id: Option<DieselUlid>,
    pub conversation_id: Option<DieselUlid>,
    pub incident_type: String,
    pub severity: String,
    pub encrypted_incident_data: Option<String>,
    pub incident_algorithm: Option<String>,
}
impl SecurityIncident {
    pub fn new(
        device_id: Option<DieselUlid>,
        user_id: Option<DieselUlid>,
        conversation_id: Option<DieselUlid>,
        incident_type: IncidentType,
        severity: IncidentSeverity,
        encrypted_incident_data: Option<String>,
        incident_algorithm: Option<String>,
    ) -> Self {
        let now = Utc::now();
        SecurityIncident {
            id: DieselUlid::new(),
            device_id,
            user_id,
            conversation_id,
            incident_type: incident_type.into(),
            severity: severity.into(),
            encrypted_incident_data,
            incident_algorithm,
            is_resolved: false,
            resolved_at: None,
            resolution_notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn incident_type_enum(&self) -> IncidentType {
        self.incident_type.clone().into()
    }

    pub fn severity_enum(&self) -> IncidentSeverity {
        self.severity.clone().into()
    }

    pub fn is_critical(&self) -> bool {
        matches!(self.severity_enum(), IncidentSeverity::Critical)
    }

    pub fn is_high_priority(&self) -> bool {
        matches!(self.severity_enum(), IncidentSeverity::High | IncidentSeverity::Critical)
    }

    pub fn resolve(&mut self, resolution_notes: String) {
        self.is_resolved = true;
        self.resolved_at = Some(Utc::now());
        self.resolution_notes = Some(resolution_notes);
        self.updated_at = Utc::now();
    }
}

impl HasId for SecurityIncident {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for SecurityIncident {
    fn table_name() -> &'static str {
        "security_incidents"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "user_id",
            "conversation_id",
            "incident_type",
            "severity",
            "is_resolved",
            "resolved_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "incident_type",
            "severity",
            "is_resolved",
            "resolved_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "user_id",
            "conversation_id",
            "incident_type",
            "severity",
            "is_resolved",
            "resolved_at",
            "resolution_notes",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "device",
            "user",
            "conversation",
        ]
    }
}

crate::impl_query_builder_service!(SecurityIncident);