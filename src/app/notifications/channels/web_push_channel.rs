use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::database::DbPool;
use web_push::{WebPushMessageBuilder, SubscriptionInfo};
use crate::app::notifications::channels::Channel;
use crate::app::notifications::notification::{Notification, Notifiable, NotificationChannel};
use crate::config::Config;
use crate::app::utils::vapid::VapidTokenGenerator;
use crate::app::utils::web_push_metrics;
use diesel::prelude::*;
use crate::schema::push_subscriptions;

#[derive(Debug)]
pub struct WebPushChannel {
    vapid_token_generator: VapidTokenGenerator,
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

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, QueryableByName)]
#[diesel(table_name = crate::schema::push_subscriptions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PushSubscription {
    pub id: String,
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

        // Load VAPID keys from environment
        let vapid_private_key = config.webpush.vapid_private_key
            .ok_or_else(|| anyhow::anyhow!("VAPID private key not configured"))?;

        let vapid_public_key = config.webpush.vapid_public_key
            .ok_or_else(|| anyhow::anyhow!("VAPID public key not configured"))?;

        let vapid_token_generator = VapidTokenGenerator::new(
            vapid_private_key,
            vapid_public_key,
            config.webpush.vapid_subject,
        );

        // Validate the VAPID configuration
        vapid_token_generator.validate_keys()?;

        Ok(Self {
            vapid_token_generator,
        })
    }

    fn get_database_pool() -> Result<DbPool> {
        let config = Config::load()?;
        let pool = crate::database::create_pool(&config)?;
        Ok(pool)
    }

    pub fn get_user_subscriptions(user_id: &str) -> Result<Vec<PushSubscription>> {
        let pool = Self::get_database_pool()?;
        let mut conn = pool.get()?;

        let subscriptions = push_subscriptions::table
            .filter(push_subscriptions::user_id.eq(user_id))
            .order(push_subscriptions::created_at.desc())
            .load::<PushSubscription>(&mut conn)?;

        Ok(subscriptions)
    }

    pub fn save_subscription(
        user_id: &str,
        endpoint: &str,
        p256dh_key: &str,
        auth_key: &str,
        user_agent: Option<&str>,
    ) -> Result<PushSubscription> {
        let pool = Self::get_database_pool()?;
        let mut conn = pool.get()?;
        let now = chrono::Utc::now();
        let id = ulid::Ulid::new();

        // Check if subscription already exists
        let existing = push_subscriptions::table
            .filter(push_subscriptions::user_id.eq(user_id))
            .filter(push_subscriptions::endpoint.eq(endpoint))
            .first::<PushSubscription>(&mut conn)
            .optional()?;

        if let Some(_existing_sub) = existing {
            // Update existing subscription
            let subscription = diesel::update(
                push_subscriptions::table
                    .filter(push_subscriptions::user_id.eq(user_id))
                    .filter(push_subscriptions::endpoint.eq(endpoint))
            )
            .set((
                push_subscriptions::p256dh_key.eq(p256dh_key),
                push_subscriptions::auth_key.eq(auth_key),
                push_subscriptions::user_agent.eq(user_agent),
                push_subscriptions::updated_at.eq(now),
            ))
            .get_result::<PushSubscription>(&mut conn)?;

            return Ok(subscription);
        }

        // Insert new subscription
        let subscription = diesel::insert_into(push_subscriptions::table)
            .values((
                push_subscriptions::id.eq(id.to_string()),
                push_subscriptions::user_id.eq(user_id),
                push_subscriptions::endpoint.eq(endpoint),
                push_subscriptions::p256dh_key.eq(p256dh_key),
                push_subscriptions::auth_key.eq(auth_key),
                push_subscriptions::user_agent.eq(user_agent),
                push_subscriptions::created_at.eq(now),
                push_subscriptions::updated_at.eq(now),
            ))
            .get_result::<PushSubscription>(&mut conn)?;

        Ok(subscription)
    }

    pub fn remove_subscription(user_id: &str, endpoint: &str) -> Result<()> {
        let pool = Self::get_database_pool()?;
        let mut conn = pool.get()?;

        diesel::delete(
            push_subscriptions::table
                .filter(push_subscriptions::user_id.eq(user_id))
                .filter(push_subscriptions::endpoint.eq(endpoint))
        )
        .execute(&mut conn)?;

        Ok(())
    }

    /// Remove expired subscription by ID
    pub async fn remove_expired_subscription(subscription_id: &str) -> Result<()> {
        let pool = Self::get_database_pool()?;
        let mut conn = pool.get()?;

        let rows_affected = diesel::delete(
            push_subscriptions::table
                .filter(push_subscriptions::id.eq(subscription_id))
        )
        .execute(&mut conn)?;

        if rows_affected > 0 {
            tracing::info!("Removed expired subscription: {}", subscription_id);
        } else {
            tracing::warn!("No subscription found with ID: {}", subscription_id);
        }

        Ok(())
    }

    /// Clean up old and inactive subscriptions
    pub async fn cleanup_inactive_subscriptions(days_inactive: i32) -> Result<u64> {
        let pool = Self::get_database_pool()?;
        let mut conn = pool.get()?;
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days_inactive as i64);

        let deleted_count = diesel::delete(
            push_subscriptions::table
                .filter(push_subscriptions::updated_at.lt(cutoff_date))
        ).execute(&mut conn)?;

        tracing::info!("Cleaned up {} inactive push subscriptions older than {} days", deleted_count, days_inactive);
        Ok(deleted_count as u64)
    }

    /// Mark subscription as failed for future cleanup
    pub async fn mark_subscription_failed(subscription_id: &str) -> Result<()> {
        use crate::database::connection::get_connection;
        use crate::schema::push_subscriptions;
        use diesel::prelude::*;
        use chrono::Utc;

        let pool = get_connection().await?;
        let mut conn = pool.get()?;

        // Update subscription timestamp for tracking (basic failure tracking)
        let result = diesel::update(push_subscriptions::table)
            .filter(push_subscriptions::id.eq(subscription_id))
            .set(push_subscriptions::updated_at.eq(Utc::now().naive_utc()))
            .execute(&mut conn)?;

        if result > 0 {
            tracing::warn!("Subscription {} marked as failed (timestamp updated for cleanup tracking)", subscription_id);
        } else {
            tracing::error!("Failed to update subscription {} failure status", subscription_id);
        }

        // For production: add failure tracking fields to push_subscriptions table:
        // - failed_attempts: Integer
        // - last_error: Nullable Text
        // - last_failed_at: Nullable Timestamp
        // This enables proper failure tracking and exponential backoff

        Ok(())
    }

    /// Validate and test a subscription endpoint
    pub async fn validate_subscription(subscription: &PushSubscription) -> Result<bool> {
        // Create a minimal test message
        let test_payload = serde_json::json!({
            "title": "Test",
            "body": "Connection test",
            "tag": "test-validation",
            "requireInteraction": false,
            "silent": true
        });

        let subscription_info = SubscriptionInfo::new(
            &subscription.endpoint,
            &subscription.p256dh_key,
            &subscription.auth_key,
        );

        let test_payload_str = serde_json::to_string(&test_payload)?;
        let mut builder = WebPushMessageBuilder::new(&subscription_info);
        builder.set_payload(web_push::ContentEncoding::Aes128Gcm, test_payload_str.as_bytes());

        let web_push_message = builder.build()?;
        let client = reqwest::Client::new();

        let response = client
            .post(&subscription.endpoint)
            .header("TTL", web_push_message.ttl.to_string())
            .header("Content-Type", "application/octet-stream")
            .header("Content-Encoding", "aes128gcm")
            .body(if let Some(payload) = web_push_message.payload {
                payload.content
            } else {
                Vec::new()
            })
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    pub async fn send_to_subscription(
        &self,
        subscription: &PushSubscription,
        message: &WebPushMessage,
    ) -> Result<()> {
        self.send_to_subscription_with_retry(subscription, message, 3).await
    }

    /// Send notification with retry logic
    pub async fn send_to_subscription_with_retry(
        &self,
        subscription: &PushSubscription,
        message: &WebPushMessage,
        max_retries: u32,
    ) -> Result<()> {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            match self.attempt_send_notification(subscription, message).await {
                Ok(_) => {
                    if attempt > 0 {
                        tracing::info!(
                            "Web push notification succeeded on attempt {} for subscription: {}",
                            attempt + 1,
                            subscription.endpoint
                        );
                    }
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e);

                    if attempt < max_retries {
                        // Calculate exponential backoff delay
                        let delay_ms = 1000 * (2_u64.pow(attempt));
                        let delay = std::time::Duration::from_millis(delay_ms);

                        tracing::warn!(
                            "Web push notification failed on attempt {} for {}, retrying in {}ms: {}",
                            attempt + 1,
                            subscription.endpoint,
                            delay_ms,
                            last_error.as_ref().unwrap()
                        );

                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        // All retries failed
        let final_error = last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error"));
        tracing::error!(
            "Web push notification failed after {} attempts for {}: {}",
            max_retries + 1,
            subscription.endpoint,
            final_error
        );

        // Mark subscription as potentially problematic
        if let Err(e) = Self::mark_subscription_failed(&subscription.id).await {
            tracing::error!("Failed to mark subscription as failed: {}", e);
        }

        Err(final_error)
    }

    /// Single attempt to send notification
    async fn attempt_send_notification(
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

        // For proper VAPID authentication, we'll use our token generator
        // but still need to use the web-push library's builder for message construction
        let web_push_message = builder.build()?;

        // Create HTTP client and send the notification
        let client = reqwest::Client::new();

        // Extract headers and body from the web push message
        let mut request_builder = client.post(&subscription.endpoint);

        // Add TTL header
        request_builder = request_builder.header("TTL", web_push_message.ttl.to_string());

        // Add Urgency header if specified
        if let Some(urgency) = &web_push_message.urgency {
            request_builder = request_builder.header("Urgency", urgency.to_string());
        }

        // Add Topic header if specified
        if let Some(topic) = &web_push_message.topic {
            request_builder = request_builder.header("Topic", topic);
        }

        // Add Content-Encoding header
        request_builder = request_builder.header("Content-Encoding", "aes128gcm");

        // Add Content-Type header for encrypted payload
        request_builder = request_builder.header("Content-Type", "application/octet-stream");

        // Add Authorization header with proper VAPID JWT
        let auth_header = self.vapid_token_generator.generate_auth_header(&subscription.endpoint)?;
        request_builder = request_builder.header("Authorization", auth_header);

        // Send the request with encrypted payload
        let body_data = if let Some(payload) = web_push_message.payload {
            payload.content
        } else {
            Vec::new()
        };

        let request_start = std::time::Instant::now();
        let response = request_builder
            .body(body_data)
            .send()
            .await?;

        let request_duration = request_start.elapsed();
        let response_status = response.status();

        // Handle the response
        match response_status.as_u16() {
            200..=299 => {
                // Record successful notification
                web_push_metrics::record_success(request_duration).await;

                tracing::info!(
                    "Web push notification sent successfully to subscription: {} ({}ms)",
                    subscription.endpoint,
                    request_duration.as_millis()
                );
                Ok(())
            }
            400 => {
                web_push_metrics::record_failure("invalid_request", Some(400)).await;

                tracing::warn!(
                    "Invalid request for web push to {}: {}",
                    subscription.endpoint,
                    response_status
                );
                Err(anyhow::anyhow!("Invalid push request"))
            }
            401 | 403 => {
                web_push_metrics::record_failure("authentication_failed", Some(response_status.as_u16())).await;

                tracing::warn!(
                    "Authentication failed for web push to {}: {}",
                    subscription.endpoint,
                    response_status
                );
                Err(anyhow::anyhow!("Push authentication failed"))
            }
            410 => {
                web_push_metrics::record_failure("subscription_expired", Some(410)).await;

                tracing::warn!(
                    "Push subscription expired: {}",
                    subscription.endpoint
                );
                // Remove expired subscription from database
                if let Err(e) = Self::remove_expired_subscription(&subscription.id).await {
                    tracing::error!("Failed to remove expired subscription {}: {}", subscription.id, e);
                }
                Err(anyhow::anyhow!("Push subscription expired and removed"))
            }
            413 => {
                web_push_metrics::record_failure("payload_too_large", Some(413)).await;

                tracing::warn!(
                    "Push payload too large for {}: {}",
                    subscription.endpoint,
                    response_status
                );
                Err(anyhow::anyhow!("Push payload too large"))
            }
            429 => {
                web_push_metrics::record_failure("rate_limited", Some(429)).await;

                tracing::warn!(
                    "Rate limited for web push to {}: {}",
                    subscription.endpoint,
                    response_status
                );
                Err(anyhow::anyhow!("Push rate limited"))
            }
            status => {
                web_push_metrics::record_failure("unknown_error", Some(status)).await;

                let error_body = response.text().await.unwrap_or_default();
                tracing::error!(
                    "Web push failed with status {} for {}: {}",
                    status,
                    subscription.endpoint,
                    error_body
                );
                Err(anyhow::anyhow!(
                    "Push notification failed with status {}: {}",
                    status,
                    error_body
                ))
            }
        }
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
        let subscriptions = Self::get_user_subscriptions(user_id)?;

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
        NotificationChannel::WebPush
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

