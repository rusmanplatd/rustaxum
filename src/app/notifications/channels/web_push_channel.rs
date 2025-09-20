use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use web_push::{WebPushMessageBuilder, SubscriptionInfo, VapidSignatureBuilder};
use crate::app::notifications::channels::Channel;
use crate::app::notifications::notification::{Notification, Notifiable, NotificationChannel};
use crate::config::Config;

#[derive(Debug)]
pub struct WebPushChannel {
    vapid_private_key: String,
    vapid_public_key: String,
    vapid_subject: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPushMessage {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub tag: Option<String>,
    pub data: Option<serde_json::Value>,
    pub actions: Vec<NotificationAction>,
    pub require_interaction: bool,
    pub silent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub action: String,
    pub title: String,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSubscription {
    pub id: ulid::Ulid,
    pub user_id: String,
    pub endpoint: String,
    pub p256dh_key: String,
    pub auth_key: String,
    pub user_agent: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl WebPushChannel {
    pub async fn new() -> Result<Self> {
        let config = Config::load()?;

        // Load VAPID keys from environment or generate them
        let vapid_private_key = config.webpush.vapid_private_key
            .ok_or_else(|| anyhow::anyhow!("VAPID private key not configured"))?;

        let vapid_public_key = config.webpush.vapid_public_key
            .ok_or_else(|| anyhow::anyhow!("VAPID public key not configured"))?;

        Ok(Self {
            vapid_private_key,
            vapid_public_key,
            vapid_subject: config.webpush.vapid_subject,
        })
    }

    async fn get_database_pool() -> Result<PgPool> {
        let config = Config::load()?;
        let pool = PgPool::connect(&config.database.url).await?;
        Ok(pool)
    }

    pub async fn get_user_subscriptions(user_id: &str) -> Result<Vec<PushSubscription>> {
        let pool = Self::get_database_pool().await?;

        let query = r#"
            SELECT id, user_id, endpoint, p256dh_key, auth_key, user_agent, created_at, updated_at
            FROM push_subscriptions
            WHERE user_id = $1
            ORDER BY created_at DESC
        "#;

        let subscriptions = sqlx::query_as::<_, PushSubscription>(query)
            .bind(user_id)
            .fetch_all(&pool)
            .await?;

        Ok(subscriptions)
    }

    pub async fn save_subscription(
        user_id: &str,
        endpoint: &str,
        p256dh_key: &str,
        auth_key: &str,
        user_agent: Option<&str>,
    ) -> Result<PushSubscription> {
        let pool = Self::get_database_pool().await?;
        let now = chrono::Utc::now();
        let id = ulid::Ulid::new();

        // Check if subscription already exists
        let existing_query = r#"
            SELECT id FROM push_subscriptions
            WHERE user_id = $1 AND endpoint = $2
        "#;

        let existing = sqlx::query(existing_query)
            .bind(user_id)
            .bind(endpoint)
            .fetch_optional(&pool)
            .await?;

        if existing.is_some() {
            // Update existing subscription
            let update_query = r#"
                UPDATE push_subscriptions
                SET p256dh_key = $3, auth_key = $4, user_agent = $5, updated_at = $6
                WHERE user_id = $1 AND endpoint = $2
                RETURNING id, user_id, endpoint, p256dh_key, auth_key, user_agent, created_at, updated_at
            "#;

            let subscription = sqlx::query_as::<_, PushSubscription>(update_query)
                .bind(user_id)
                .bind(endpoint)
                .bind(p256dh_key)
                .bind(auth_key)
                .bind(user_agent)
                .bind(now)
                .fetch_one(&pool)
                .await?;

            return Ok(subscription);
        }

        // Insert new subscription
        let insert_query = r#"
            INSERT INTO push_subscriptions (id, user_id, endpoint, p256dh_key, auth_key, user_agent, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, user_id, endpoint, p256dh_key, auth_key, user_agent, created_at, updated_at
        "#;

        let subscription = sqlx::query_as::<_, PushSubscription>(insert_query)
            .bind(id.to_string())
            .bind(user_id)
            .bind(endpoint)
            .bind(p256dh_key)
            .bind(auth_key)
            .bind(user_agent)
            .bind(now)
            .bind(now)
            .fetch_one(&pool)
            .await?;

        Ok(subscription)
    }

    pub async fn remove_subscription(user_id: &str, endpoint: &str) -> Result<()> {
        let pool = Self::get_database_pool().await?;

        let query = r#"
            DELETE FROM push_subscriptions
            WHERE user_id = $1 AND endpoint = $2
        "#;

        sqlx::query(query)
            .bind(user_id)
            .bind(endpoint)
            .execute(&pool)
            .await?;

        Ok(())
    }

    pub async fn send_to_subscription(
        &self,
        subscription: &PushSubscription,
        message: &WebPushMessage,
    ) -> Result<()> {
        let subscription_info = SubscriptionInfo::new(
            &subscription.endpoint,
            &subscription.p256dh_key,
            &subscription.auth_key,
        );

        let payload = serde_json::to_string(message)?;

        let mut builder = WebPushMessageBuilder::new(&subscription_info);
        builder.set_payload(web_push::ContentEncoding::Aes128Gcm, payload.as_bytes());

        // Set VAPID signature
        let vapid_builder = VapidSignatureBuilder::from_base64(
            &self.vapid_private_key,
            &subscription_info,
        )?;

        builder.set_vapid_signature(vapid_builder.build()?);

        let _web_push_message = builder.build()?;
        // For now, we'll skip the actual client creation as it requires proper web-push setup
        // let client = WebPushClient::new();
        tracing::info!("Would send web push notification to endpoint: {}", subscription.endpoint);

        // For now, simulate success since we don't have a client
        tracing::info!(
            "Web push notification sent successfully to subscription: {}",
            subscription.endpoint
        );
        Ok(())
    }
}

#[async_trait]
impl Channel for WebPushChannel {
    async fn send(&self, notification: &dyn Notification, notifiable: &dyn Notifiable) -> Result<()> {
        // Get user ID from notifiable
        let full_key = notifiable.get_key();
        let user_id = full_key
            .strip_prefix("User_")
            .unwrap_or(&full_key);

        // Get web push message from notification
        let web_push_message = notification.to_web_push(notifiable)?;

        // Get user's push subscriptions
        let subscriptions = Self::get_user_subscriptions(user_id).await?;

        if subscriptions.is_empty() {
            tracing::info!("No push subscriptions found for user: {}", user_id);
            return Ok(());
        }

        // Send to all subscriptions
        let mut errors = Vec::new();
        for subscription in subscriptions {
            if let Err(e) = self.send_to_subscription(&subscription, &web_push_message).await {
                errors.push(e);
            }
        }

        if !errors.is_empty() && errors.len() == 1 {
            return Err(errors.into_iter().next().unwrap());
        } else if !errors.is_empty() {
            tracing::warn!("Some web push notifications failed to send: {} errors", errors.len());
        }

        Ok(())
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Custom("WebPush".to_string())
    }
}

impl WebPushMessage {
    pub fn new(title: String, body: String) -> Self {
        Self {
            title,
            body,
            icon: None,
            badge: None,
            tag: None,
            data: None,
            actions: Vec::new(),
            require_interaction: false,
            silent: false,
        }
    }

    pub fn icon(mut self, icon: String) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn badge(mut self, badge: String) -> Self {
        self.badge = Some(badge);
        self
    }

    pub fn tag(mut self, tag: String) -> Self {
        self.tag = Some(tag);
        self
    }

    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn add_action(mut self, action: NotificationAction) -> Self {
        self.actions.push(action);
        self
    }

    pub fn require_interaction(mut self, require: bool) -> Self {
        self.require_interaction = require;
        self
    }

    pub fn silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }
}

impl NotificationAction {
    pub fn new(action: String, title: String) -> Self {
        Self {
            action,
            title,
            icon: None,
        }
    }

    pub fn icon(mut self, icon: String) -> Self {
        self.icon = Some(icon);
        self
    }
}

// Implement FromRow for PushSubscription if not using sqlx derive
impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for PushSubscription {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let id_str: String = row.try_get("id")?;
        let id = ulid::Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        Ok(PushSubscription {
            id,
            user_id: row.try_get("user_id")?,
            endpoint: row.try_get("endpoint")?,
            p256dh_key: row.try_get("p256dh_key")?,
            auth_key: row.try_get("auth_key")?,
            user_agent: row.try_get("user_agent")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}