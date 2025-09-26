use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;
use crate::app::query_builder::SortDirection;
use super::DieselUlid;
use crate::app::models::activity_log::HasId;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::device_capabilities)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeviceCapabilities {
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: DieselUlid,
    pub device_id: DieselUlid,
    pub supports_aes_256_gcm: bool,
    pub supports_chacha20_poly1305: bool,
    pub supports_aes_128_gcm: bool,
    pub supports_curve25519: bool,
    pub supports_p256_ecdh: bool,
    pub supports_rsa_2048: bool,
    pub supports_rsa_4096: bool,
    pub supports_hmac_sha256: bool,
    pub supports_hmac_sha384: bool,
    pub supports_hmac_sha512: bool,
    pub supports_blake3_mac: bool,
    pub max_signal_protocol_version: i32,
    pub min_signal_protocol_version: i32,
    pub supports_multi_device: bool,
    pub supports_group_messaging: bool,
    pub supports_disappearing_messages: bool,
    pub supports_file_encryption: bool,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
    // Post-quantum cryptography support
    pub supports_ed25519_signature: bool,
    pub supports_rsa_pss_signature: bool,
    pub supports_ecdsa_p256_signature: bool,
    pub supports_hkdf_sha256: bool,
    pub supports_hkdf_sha384: bool,
    pub supports_hkdf_sha512: bool,
    pub supports_kyber_768: bool,
    pub supports_dilithium2: bool,
    pub supports_sphincs_plus: bool,
    pub supports_bike_r4: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateDeviceCapabilities {
    pub device_id: DieselUlid,
    pub supports_aes_256_gcm: bool,
    pub supports_chacha20_poly1305: bool,
    pub supports_aes_128_gcm: bool,
    pub supports_curve25519: bool,
    pub max_signal_protocol_version: i32,
    pub min_signal_protocol_version: i32,
    pub supports_multi_device: bool,
    pub supports_group_messaging: bool,
    pub supports_disappearing_messages: bool,
    pub supports_file_encryption: bool,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::device_capabilities)]
pub struct NewDeviceCapabilities {
    pub id: DieselUlid,
    pub device_id: DieselUlid,
    pub supports_aes_256_gcm: bool,
    pub supports_chacha20_poly1305: bool,
    pub supports_aes_128_gcm: bool,
    pub supports_curve25519: bool,
    pub supports_p256_ecdh: bool,
    pub supports_rsa_2048: bool,
    pub supports_rsa_4096: bool,
    pub supports_hmac_sha256: bool,
    pub supports_hmac_sha384: bool,
    pub supports_hmac_sha512: bool,
    pub supports_blake3_mac: bool,
    pub max_signal_protocol_version: i32,
    pub min_signal_protocol_version: i32,
    pub supports_multi_device: bool,
    pub supports_group_messaging: bool,
    pub supports_disappearing_messages: bool,
    pub supports_file_encryption: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub supports_ed25519_signature: bool,
    pub supports_rsa_pss_signature: bool,
    pub supports_ecdsa_p256_signature: bool,
    pub supports_hkdf_sha256: bool,
    pub supports_hkdf_sha384: bool,
    pub supports_hkdf_sha512: bool,
    pub supports_kyber_768: bool,
    pub supports_dilithium2: bool,
    pub supports_sphincs_plus: bool,
    pub supports_bike_r4: bool,
}

impl DeviceCapabilities {
    pub fn new(device_id: DieselUlid) -> Self {
        let now = Utc::now();
        Self {
            id: DieselUlid::new(),
            device_id,
            supports_aes_256_gcm: true,
            supports_chacha20_poly1305: true,
            supports_aes_128_gcm: true,
            supports_curve25519: true,
            supports_p256_ecdh: false,
            supports_rsa_2048: false,
            supports_rsa_4096: false,
            supports_hmac_sha256: true,
            supports_hmac_sha384: false,
            supports_hmac_sha512: false,
            supports_blake3_mac: false,
            max_signal_protocol_version: 3,
            min_signal_protocol_version: 1,
            supports_multi_device: true,
            supports_group_messaging: true,
            supports_disappearing_messages: true,
            supports_file_encryption: true,
            created_at: now,
            updated_at: now,
            supports_ed25519_signature: false,
            supports_rsa_pss_signature: false,
            supports_ecdsa_p256_signature: false,
            supports_hkdf_sha256: true,
            supports_hkdf_sha384: false,
            supports_hkdf_sha512: false,
            supports_kyber_768: false,
            supports_dilithium2: false,
            supports_sphincs_plus: false,
            supports_bike_r4: false,
        }
    }

    pub fn has_post_quantum_support(&self) -> bool {
        self.supports_kyber_768 || self.supports_dilithium2 || self.supports_sphincs_plus || self.supports_bike_r4
    }

    pub fn get_supported_encryption_algorithms(&self) -> Vec<&'static str> {
        let mut algorithms = Vec::new();
        if self.supports_aes_256_gcm { algorithms.push("aes-256-gcm"); }
        if self.supports_chacha20_poly1305 { algorithms.push("chacha20-poly1305"); }
        if self.supports_aes_128_gcm { algorithms.push("aes-128-gcm"); }
        algorithms
    }

    pub fn get_supported_key_exchanges(&self) -> Vec<&'static str> {
        let mut exchanges = Vec::new();
        if self.supports_curve25519 { exchanges.push("curve25519"); }
        if self.supports_p256_ecdh { exchanges.push("p256-ecdh"); }
        if self.supports_kyber_768 { exchanges.push("kyber-768"); }
        exchanges
    }
}

impl HasId for DeviceCapabilities {
    fn id(&self) -> String {
        self.id.to_string()
    }
}

impl crate::app::query_builder::Queryable for DeviceCapabilities {
    fn table_name() -> &'static str {
        "device_capabilities"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "supports_aes_256_gcm",
            "supports_chacha20_poly1305",
            "supports_curve25519",
            "max_signal_protocol_version",
            "min_signal_protocol_version",
            "supports_multi_device",
            "supports_group_messaging",
            "supports_disappearing_messages",
            "supports_file_encryption",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "max_signal_protocol_version",
            "min_signal_protocol_version",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "device_id",
            "supports_aes_256_gcm",
            "supports_chacha20_poly1305",
            "supports_aes_128_gcm",
            "supports_curve25519",
            "max_signal_protocol_version",
            "min_signal_protocol_version",
            "supports_multi_device",
            "supports_group_messaging",
            "supports_disappearing_messages",
            "supports_file_encryption",
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
        ]
    }
}

crate::impl_query_builder_service!(DeviceCapabilities);