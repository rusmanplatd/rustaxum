use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::device_fingerprints)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeviceFingerprint {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub device_id: DieselUlid,
    pub identity_key_fingerprint: String,
    pub fingerprint_algorithm: String,
    pub is_verified: bool,
    pub verified_by_user_id: Option<DieselUlid>,
    pub verified_at: Option<DateTime<Utc>>,
    pub verification_method: Option<String>,
    pub trust_score: i32,
    pub trust_last_updated: Option<DateTime<Utc>>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FingerprintAlgorithm {
    Sha256,
    Sha384,
    Sha512,
    Blake3,
    Sha3_256,
    Sha3_512,
}

impl From<String> for FingerprintAlgorithm {
    fn from(s: String) -> Self {
        match s.as_str() {
            "sha256" => FingerprintAlgorithm::Sha256,
            "sha384" => FingerprintAlgorithm::Sha384,
            "sha512" => FingerprintAlgorithm::Sha512,
            "blake3" => FingerprintAlgorithm::Blake3,
            "sha3-256" => FingerprintAlgorithm::Sha3_256,
            "sha3-512" => FingerprintAlgorithm::Sha3_512,
            _ => FingerprintAlgorithm::Sha256,
        }
    }
}

impl From<FingerprintAlgorithm> for String {
    fn from(algorithm: FingerprintAlgorithm) -> Self {
        match algorithm {
            FingerprintAlgorithm::Sha256 => "sha256".to_string(),
            FingerprintAlgorithm::Sha384 => "sha384".to_string(),
            FingerprintAlgorithm::Sha512 => "sha512".to_string(),
            FingerprintAlgorithm::Blake3 => "blake3".to_string(),
            FingerprintAlgorithm::Sha3_256 => "sha3-256".to_string(),
            FingerprintAlgorithm::Sha3_512 => "sha3-512".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationMethod {
    QrCode,
    ManualComparison,
    OutOfBand,
    TrustedThirdParty,
    PhysicalMeeting,
    VideoCall,
}

impl From<String> for VerificationMethod {
    fn from(s: String) -> Self {
        match s.as_str() {
            "qr_code" => VerificationMethod::QrCode,
            "manual_comparison" => VerificationMethod::ManualComparison,
            "out_of_band" => VerificationMethod::OutOfBand,
            "trusted_third_party" => VerificationMethod::TrustedThirdParty,
            "physical_meeting" => VerificationMethod::PhysicalMeeting,
            "video_call" => VerificationMethod::VideoCall,
            _ => VerificationMethod::ManualComparison,
        }
    }
}

impl From<VerificationMethod> for String {
    fn from(method: VerificationMethod) -> Self {
        match method {
            VerificationMethod::QrCode => "qr_code".to_string(),
            VerificationMethod::ManualComparison => "manual_comparison".to_string(),
            VerificationMethod::OutOfBand => "out_of_band".to_string(),
            VerificationMethod::TrustedThirdParty => "trusted_third_party".to_string(),
            VerificationMethod::PhysicalMeeting => "physical_meeting".to_string(),
            VerificationMethod::VideoCall => "video_call".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateDeviceFingerprint {
    pub device_id: DieselUlid,
    pub identity_key_fingerprint: String,
    pub fingerprint_algorithm: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VerifyDeviceFingerprint {
    pub verification_method: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::device_fingerprints)]
pub struct NewDeviceFingerprint {
    pub id: DieselUlid,
    pub device_id: DieselUlid,
    pub identity_key_fingerprint: String,
    pub fingerprint_algorithm: String,
    pub is_verified: bool,
    pub verified_by_user_id: Option<DieselUlid>,
    pub verified_at: Option<DateTime<Utc>>,
    pub verification_method: Option<String>,
    pub trust_score: i32,
    pub trust_last_updated: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeviceFingerprintResponse {
    pub id: DieselUlid,
    pub device_id: DieselUlid,
    pub identity_key_fingerprint: String,
    pub fingerprint_algorithm: String,
    pub is_verified: bool,
    pub verified_by_user_id: Option<DieselUlid>,
    pub verified_at: Option<DateTime<Utc>>,
    pub verification_method: Option<String>,
    pub trust_score: i32,
    pub trust_last_updated: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DeviceFingerprint {
    pub fn new(
        device_id: DieselUlid,
        identity_key_fingerprint: String,
        fingerprint_algorithm: FingerprintAlgorithm,
    ) -> NewDeviceFingerprint {
        let now = Utc::now();
        NewDeviceFingerprint {
            id: DieselUlid::new(),
            device_id,
            identity_key_fingerprint,
            fingerprint_algorithm: fingerprint_algorithm.into(),
            is_verified: false,
            verified_by_user_id: None,
            verified_at: None,
            verification_method: None,
            trust_score: 0, // Start with 0 trust
            trust_last_updated: Some(now),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> DeviceFingerprintResponse {
        DeviceFingerprintResponse {
            id: self.id,
            device_id: self.device_id,
            identity_key_fingerprint: self.identity_key_fingerprint.clone(),
            fingerprint_algorithm: self.fingerprint_algorithm.clone(),
            is_verified: self.is_verified,
            verified_by_user_id: self.verified_by_user_id,
            verified_at: self.verified_at,
            verification_method: self.verification_method.clone(),
            trust_score: self.trust_score,
            trust_last_updated: self.trust_last_updated,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn fingerprint_algorithm_enum(&self) -> FingerprintAlgorithm {
        self.fingerprint_algorithm.clone().into()
    }

    pub fn verification_method_enum(&self) -> Option<VerificationMethod> {
        self.verification_method.as_ref().map(|m| m.clone().into())
    }

    pub fn verify(&mut self, verified_by_user_id: DieselUlid, method: VerificationMethod) {
        self.is_verified = true;
        self.verified_by_user_id = Some(verified_by_user_id);
        self.verified_at = Some(Utc::now());
        self.trust_score = self.calculate_trust_score_for_method(&method);
        self.verification_method = Some(method.into());
        self.trust_last_updated = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn unverify(&mut self) {
        self.is_verified = false;
        self.verified_by_user_id = None;
        self.verified_at = None;
        self.verification_method = None;
        self.trust_score = 0;
        self.trust_last_updated = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn update_trust_score(&mut self, new_score: i32) {
        self.trust_score = new_score.clamp(0, 100); // Trust score between 0-100
        self.trust_last_updated = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn is_trusted(&self) -> bool {
        self.is_verified && self.trust_score >= 75
    }

    pub fn is_highly_trusted(&self) -> bool {
        self.is_verified && self.trust_score >= 90
    }

    pub fn trust_level(&self) -> &'static str {
        match self.trust_score {
            0..=25 => "untrusted",
            26..=50 => "low_trust",
            51..=75 => "medium_trust",
            76..=90 => "high_trust",
            91..=100 => "very_high_trust",
            _ => "unknown",
        }
    }

    fn calculate_trust_score_for_method(&self, method: &VerificationMethod) -> i32 {
        match method {
            VerificationMethod::PhysicalMeeting => 95,
            VerificationMethod::VideoCall => 85,
            VerificationMethod::QrCode => 80,
            VerificationMethod::OutOfBand => 70,
            VerificationMethod::TrustedThirdParty => 60,
            VerificationMethod::ManualComparison => 50,
        }
    }
}

impl HasId for DeviceFingerprint {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for DeviceFingerprint {
    fn table_name() -> &'static str {
        "device_fingerprints"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "fingerprint_algorithm",
            "is_verified",
            "verified_by_user_id",
            "verified_at",
            "verification_method",
            "trust_score",
            "trust_last_updated",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "is_verified",
            "trust_score",
            "verified_at",
            "trust_last_updated",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "identity_key_fingerprint",
            "fingerprint_algorithm",
            "is_verified",
            "verified_by_user_id",
            "verified_at",
            "verification_method",
            "trust_score",
            "trust_last_updated",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("trust_score", SortDirection::Desc))
    }

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "device",
            "verified_by_user",
        ]
    }
}

crate::impl_query_builder_service!(DeviceFingerprint);