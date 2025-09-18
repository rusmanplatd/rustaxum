use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub default: String,
    pub disks: HashMap<String, DiskConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskConfig {
    pub driver: String,
    pub root: Option<String>,
    pub visibility: Option<String>,
    pub throw: Option<bool>,
    pub url: Option<String>,
    pub endpoint: Option<String>,
    pub bucket: Option<String>,
    pub region: Option<String>,
    pub key: Option<String>,
    pub secret: Option<String>,
}

impl StorageConfig {
    pub fn from_env() -> Result<Self> {
        let default = env::var("FILESYSTEM_DISK").unwrap_or_else(|_| "local".to_string());

        let mut disks = HashMap::new();

        // Local disk configuration
        disks.insert("local".to_string(), DiskConfig {
            driver: "local".to_string(),
            root: Some(env::var("STORAGE_LOCAL_ROOT").unwrap_or_else(|_| "storage/app".to_string())),
            visibility: Some("private".to_string()),
            throw: Some(false),
            url: None,
            endpoint: None,
            bucket: None,
            region: None,
            key: None,
            secret: None,
        });

        // Public disk configuration
        disks.insert("public".to_string(), DiskConfig {
            driver: "local".to_string(),
            root: Some(env::var("STORAGE_PUBLIC_ROOT").unwrap_or_else(|_| "storage/app/public".to_string())),
            visibility: Some("public".to_string()),
            throw: Some(false),
            url: Some(env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()) + "/storage"),
            endpoint: None,
            bucket: None,
            region: None,
            key: None,
            secret: None,
        });

        // S3 disk configuration (AWS S3)
        if env::var("AWS_ACCESS_KEY_ID").is_ok() {
            disks.insert("s3".to_string(), DiskConfig {
                driver: "s3".to_string(),
                root: env::var("AWS_ROOT").ok(),
                visibility: Some("private".to_string()),
                throw: Some(false),
                url: env::var("AWS_URL").ok(),
                endpoint: env::var("AWS_ENDPOINT").ok(),
                bucket: env::var("AWS_BUCKET").ok(),
                region: env::var("AWS_DEFAULT_REGION").ok(),
                key: env::var("AWS_ACCESS_KEY_ID").ok(),
                secret: env::var("AWS_SECRET_ACCESS_KEY").ok(),
            });
        }

        // MinIO disk configuration (S3-compatible)
        if env::var("MINIO_ACCESS_KEY").is_ok() || env::var("S3_ACCESS_KEY").is_ok() {
            disks.insert("minio".to_string(), DiskConfig {
                driver: "s3".to_string(),
                root: env::var("MINIO_ROOT").or_else(|_| env::var("S3_ROOT")).ok(),
                visibility: Some("private".to_string()),
                throw: Some(false),
                url: env::var("MINIO_URL").or_else(|_| env::var("S3_URL")).ok(),
                endpoint: env::var("MINIO_ENDPOINT").or_else(|_| env::var("S3_ENDPOINT")).ok(),
                bucket: env::var("MINIO_BUCKET").or_else(|_| env::var("S3_BUCKET")).ok(),
                region: env::var("MINIO_REGION").or_else(|_| env::var("S3_REGION")).ok(),
                key: env::var("MINIO_ACCESS_KEY").or_else(|_| env::var("S3_ACCESS_KEY")).ok(),
                secret: env::var("MINIO_SECRET_KEY").or_else(|_| env::var("S3_SECRET_KEY")).ok(),
            });
        }

        // Cloudflare R2 disk configuration (S3-compatible)
        if env::var("R2_ACCESS_KEY").is_ok() {
            disks.insert("r2".to_string(), DiskConfig {
                driver: "s3".to_string(),
                root: env::var("R2_ROOT").ok(),
                visibility: Some("private".to_string()),
                throw: Some(false),
                url: env::var("R2_URL").ok(),
                endpoint: env::var("R2_ENDPOINT").ok(),
                bucket: env::var("R2_BUCKET").ok(),
                region: env::var("R2_REGION").or_else(|_| Ok::<String, std::env::VarError>("auto".to_string())).ok(),
                key: env::var("R2_ACCESS_KEY").ok(),
                secret: env::var("R2_SECRET_KEY").ok(),
            });
        }

        Ok(StorageConfig {
            default,
            disks,
        })
    }

    pub fn get_disk(&self, name: &str) -> Option<&DiskConfig> {
        self.disks.get(name)
    }

    pub fn get_default_disk(&self) -> Option<&DiskConfig> {
        self.disks.get(&self.default)
    }

    pub fn disk_names(&self) -> Vec<&String> {
        self.disks.keys().collect()
    }
}

impl DiskConfig {
    pub fn is_local(&self) -> bool {
        self.driver == "local"
    }

    pub fn is_s3(&self) -> bool {
        self.driver == "s3"
    }

    pub fn is_public(&self) -> bool {
        self.visibility.as_ref().map_or(false, |v| v == "public")
    }

    pub fn get_root(&self) -> &str {
        self.root.as_deref().unwrap_or("")
    }

    pub fn get_url(&self) -> Option<&str> {
        self.url.as_deref()
    }
}