use anyhow::Result;
use sha2::{Sha256, Digest};
use uuid::Uuid;

pub struct TokenUtils;

impl TokenUtils {
    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn generate_reset_token() -> String {
        Uuid::new_v4().to_string().replace('-', "")
    }

    pub fn extract_token_from_header(auth_header: Option<&str>) -> Result<&str> {
        let header = auth_header.ok_or_else(|| anyhow::anyhow!("Authorization header missing"))?;

        if !header.starts_with("Bearer ") {
            anyhow::bail!("Invalid authorization header format");
        }

        Ok(&header[7..])
    }
}