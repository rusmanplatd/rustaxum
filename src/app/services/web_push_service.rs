use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::database::DbPool;
use crate::app::notifications::channels::web_push_channel::{PushSubscription, WebPushChannel};
use crate::config::Config;

/// Web Push service for managing push subscriptions and sending notifications
pub struct WebPushService {
    web_push_channel: Option<WebPushChannel>,
}

/// Request payload for subscribing to web push notifications
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscribeRequest {
    pub endpoint: String,
    pub keys: PushKeys,
}

/// Push subscription keys from the browser
#[derive(Debug, Serialize, Deserialize)]
pub struct PushKeys {
    pub p256dh: String,
    pub auth: String,
}

/// Response for web push subscription operations
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionResponse {
    pub success: bool,
    pub message: String,
    pub subscription_id: Option<String>,
}

/// VAPID public key response
#[derive(Debug, Serialize, Deserialize)]
pub struct VapidPublicKeyResponse {
    pub public_key: String,
}

impl WebPushService {
    pub async fn new() -> Self {
        let web_push_channel = match WebPushChannel::new().await {
            Ok(channel) => Some(channel),
            Err(e) => {
                tracing::warn!("Web push channel not available: {}", e);
                None
            }
        };

        Self { web_push_channel }
    }

    /// Get VAPID public key for client-side subscription
    pub fn get_vapid_public_key() -> Result<String> {
        let config = Config::load()?;

        config.webpush.vapid_public_key
            .ok_or_else(|| anyhow::anyhow!("VAPID public key not configured"))
    }

    /// Subscribe a user to web push notifications
    pub async fn subscribe(
        &self,
        user_id: &str,
        request: SubscribeRequest,
        user_agent: Option<String>,
    ) -> Result<SubscriptionResponse> {
        // Validate the subscription request
        if request.endpoint.is_empty() || request.keys.p256dh.is_empty() || request.keys.auth.is_empty() {
            return Ok(SubscriptionResponse {
                success: false,
                message: "Invalid subscription data".to_string(),
                subscription_id: None,
            });
        }

        // Save the subscription to the database
        match WebPushChannel::save_subscription(
            user_id,
            &request.endpoint,
            &request.keys.p256dh,
            &request.keys.auth,
            user_agent.as_deref(),
        ).await {
            Ok(subscription) => {
                tracing::info!(
                    "User {} subscribed to web push notifications: {}",
                    user_id,
                    subscription.endpoint
                );

                Ok(SubscriptionResponse {
                    success: true,
                    message: "Successfully subscribed to push notifications".to_string(),
                    subscription_id: Some(subscription.id.to_string()),
                })
            }
            Err(e) => {
                tracing::error!("Failed to save push subscription for user {}: {}", user_id, e);
                Ok(SubscriptionResponse {
                    success: false,
                    message: "Failed to save subscription".to_string(),
                    subscription_id: None,
                })
            }
        }
    }

    /// Unsubscribe a user from web push notifications
    pub async fn unsubscribe(
        &self,
        user_id: &str,
        endpoint: &str,
    ) -> Result<SubscriptionResponse> {
        match WebPushChannel::remove_subscription(user_id, endpoint).await {
            Ok(_) => {
                tracing::info!(
                    "User {} unsubscribed from web push notifications: {}",
                    user_id,
                    endpoint
                );

                Ok(SubscriptionResponse {
                    success: true,
                    message: "Successfully unsubscribed from push notifications".to_string(),
                    subscription_id: None,
                })
            }
            Err(e) => {
                tracing::error!("Failed to remove push subscription for user {}: {}", user_id, e);
                Ok(SubscriptionResponse {
                    success: false,
                    message: "Failed to remove subscription".to_string(),
                    subscription_id: None,
                })
            }
        }
    }

    /// Get all push subscriptions for a user
    pub async fn get_user_subscriptions(&self, user_id: &str) -> Result<Vec<PushSubscription>> {
        WebPushChannel::get_user_subscriptions(user_id).await
    }

    /// Test if web push is configured and working
    pub fn test_web_push_configuration(&self) -> Result<bool> {
        let config = Config::load()?;

        // Check if VAPID keys are configured
        if !config.webpush.is_configured() {
            return Ok(false);
        }

        // Check if we can create a web push channel
        Ok(self.web_push_channel.is_some())
    }

    /// Send a test notification to a user
    pub async fn send_test_notification(
        &self,
        user_id: &str,
        title: &str,
        message: &str,
    ) -> Result<SubscriptionResponse> {
        if self.web_push_channel.is_none() {
            return Ok(SubscriptionResponse {
                success: false,
                message: "Web push not configured".to_string(),
                subscription_id: None,
            });
        }

        let subscriptions = self.get_user_subscriptions(user_id).await?;

        if subscriptions.is_empty() {
            return Ok(SubscriptionResponse {
                success: false,
                message: "No push subscriptions found for user".to_string(),
                subscription_id: None,
            });
        }

        // Create a simple test notification
        let web_push_message = crate::app::notifications::channels::web_push_channel::WebPushMessage::new(
            title.to_string(),
            message.to_string(),
        ).tag("test-notification".to_string());

        let mut success_count = 0;
        let mut error_count = 0;

        // Send to all user subscriptions
        for subscription in &subscriptions {
            if let Some(channel) = &self.web_push_channel {
                match channel.send_to_subscription(&subscription, &web_push_message).await {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }
        }

        Ok(SubscriptionResponse {
            success: success_count > 0,
            message: format!(
                "Sent to {} subscriptions, {} failed",
                success_count,
                error_count
            ),
            subscription_id: None,
        })
    }

    /// Clean up invalid subscriptions (run periodically)
    pub async fn cleanup_invalid_subscriptions(&self) -> Result<u64> {
        use diesel::prelude::*;
        use crate::schema::push_subscriptions;

        let pool = Self::get_database_pool().await?;
        let mut conn = pool.get()?;
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(30);

        let deleted_count = diesel::delete(
            push_subscriptions::table
                .filter(push_subscriptions::updated_at.lt(cutoff_date))
        ).execute(&mut conn)?;

        tracing::info!("Cleaned up {} old push subscriptions", deleted_count);
        Ok(deleted_count as u64)
    }

    async fn get_database_pool() -> Result<DbPool> {
        crate::database::create_pool()
    }
}

