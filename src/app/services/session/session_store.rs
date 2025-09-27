use axum::http::{HeaderMap, HeaderValue};
use axum::response::Response;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::session::SessionConfig;
use super::{Session, SessionManager};

#[derive(Clone)]
pub struct SessionStore {
    session: Arc<RwLock<Option<Session>>>,
    manager: Arc<SessionManager>,
    config: SessionConfig,
}

impl SessionStore {
    pub fn new(manager: SessionManager, config: SessionConfig) -> Self {
        Self {
            session: Arc::new(RwLock::new(None)),
            manager: Arc::new(manager),
            config,
        }
    }

    pub async fn start(&self, session_id: Option<String>) -> anyhow::Result<()> {
        let session = self.manager.start(session_id).await?;
        let mut session_lock = self.session.write().await;
        *session_lock = Some(session);
        Ok(())
    }

    pub async fn get_session_id(&self) -> Option<String> {
        let session = self.session.read().await;
        session.as_ref().map(|s| s.id.clone())
    }

    pub async fn put(&self, key: &str, value: serde_json::Value) {
        if let Some(session) = self.session.write().await.as_mut() {
            session.put(key, value);
        }
    }

    pub async fn get(&self, key: &str) -> Option<serde_json::Value> {
        let session = self.session.read().await;
        session.as_ref()?.get(key).cloned()
    }

    pub async fn get_string(&self, key: &str) -> Option<String> {
        let session = self.session.read().await;
        session.as_ref()?.get_string(key)
    }

    pub async fn get_i64(&self, key: &str) -> Option<i64> {
        let session = self.session.read().await;
        session.as_ref()?.get_i64(key)
    }

    pub async fn get_bool(&self, key: &str) -> Option<bool> {
        let session = self.session.read().await;
        session.as_ref()?.get_bool(key)
    }

    pub async fn has(&self, key: &str) -> bool {
        let session = self.session.read().await;
        session.as_ref().map_or(false, |s| s.has(key))
    }

    pub async fn forget(&self, key: &str) -> Option<serde_json::Value> {
        let mut session = self.session.write().await;
        session.as_mut()?.forget(key)
    }

    pub async fn flush(&self) {
        if let Some(session) = self.session.write().await.as_mut() {
            session.flush();
        }
    }

    pub async fn flash(&self, key: &str, value: serde_json::Value) {
        if let Some(session) = self.session.write().await.as_mut() {
            session.flash(key, value);
        }
    }

    pub async fn now(&self, key: &str, value: serde_json::Value) {
        if let Some(session) = self.session.write().await.as_mut() {
            session.now(key, value);
        }
    }

    pub async fn reflash(&self) {
        if let Some(session) = self.session.write().await.as_mut() {
            session.reflash();
        }
    }

    pub async fn keep(&self, keys: Vec<&str>) {
        if let Some(session) = self.session.write().await.as_mut() {
            session.keep(keys);
        }
    }

    pub async fn token(&self) -> Option<String> {
        let mut session = self.session.write().await;
        session.as_mut().map(|s| s.token())
    }

    pub async fn regenerate_token(&self) -> Option<String> {
        let mut session = self.session.write().await;
        session.as_mut().map(|s| s.regenerate_token())
    }

    pub async fn regenerate(&self) -> anyhow::Result<()> {
        let mut session_lock = self.session.write().await;
        if let Some(session) = session_lock.as_mut() {
            self.manager.regenerate(session).await?;
        }
        Ok(())
    }

    pub async fn migrate(&self) -> anyhow::Result<()> {
        let mut session_lock = self.session.write().await;
        if let Some(session) = session_lock.as_mut() {
            self.manager.migrate(session).await?;
        }
        Ok(())
    }

    pub async fn invalidate(&self) -> anyhow::Result<()> {
        let mut session_lock = self.session.write().await;
        if let Some(session) = session_lock.as_mut() {
            self.manager.invalidate(session).await?;
        }
        Ok(())
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        let mut session_lock = self.session.write().await;
        if let Some(session) = session_lock.as_mut() {
            self.manager.save(session).await?;
        }
        Ok(())
    }

    pub fn extract_session_id_from_cookies(&self, cookies: &HeaderMap) -> Option<String> {
        let cookie_header = cookies.get("cookie")?.to_str().ok()?;

        for cookie_str in cookie_header.split(';') {
            let cookie_str = cookie_str.trim();
            if let Some((name, value)) = cookie_str.split_once('=') {
                if name == self.config.cookie {
                    return Some(value.to_string());
                }
            }
        }

        None
    }

    pub fn create_session_cookie_header(&self, session_id: &str) -> String {
        let mut cookie_parts = vec![
            format!("{}={}", self.config.cookie, session_id),
            format!("Path={}", self.config.path),
        ];

        if self.config.http_only {
            cookie_parts.push("HttpOnly".to_string());
        }

        if let Some(domain) = &self.config.domain {
            cookie_parts.push(format!("Domain={}", domain));
        }

        if let Some(secure) = self.config.secure {
            if secure {
                cookie_parts.push("Secure".to_string());
            }
        }

        if let Some(same_site) = &self.config.same_site {
            cookie_parts.push(format!("SameSite={}", same_site));
        }

        if !self.config.expire_on_close {
            // Use both Max-Age and a properly formatted Expires date for compatibility
            let expiry = chrono::Utc::now()
                .checked_add_signed(chrono::Duration::seconds(self.config.lifetime_in_seconds() as i64))
                .unwrap_or_else(|| chrono::Utc::now());

            // RFC 1123 format, e.g. "Wed, 21 Oct 2015 07:28:00 GMT"
            let expires_str = expiry.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
            cookie_parts.push(format!("Max-Age={}", self.config.lifetime_in_seconds()));
            cookie_parts.push(format!("Expires={}", expires_str));
        }

        cookie_parts.join("; ")
    }

    pub fn add_session_cookie_to_response(&self, mut response: Response, session_id: &str) -> Response {
        let cookie_header = self.create_session_cookie_header(session_id);
        let header_value = HeaderValue::from_str(&cookie_header).unwrap();

        response.headers_mut().append("set-cookie", header_value);
        response
    }
}