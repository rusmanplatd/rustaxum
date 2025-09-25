use async_trait::async_trait;
use std::collections::HashMap;
use crate::app::notifications::notification::{Notifiable, NotificationChannel};
use crate::app::models::user::User;

#[async_trait]
impl Notifiable for User {
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
    async fn route_notification_for(&self, channel: &NotificationChannel) -> Option<String> {
        match channel {
            NotificationChannel::Mail => Some(self.email.clone()),
            NotificationChannel::Database => Some(self.id.to_string()),
            NotificationChannel::Broadcast => Some(self.id.to_string()),
            NotificationChannel::WebPush => Some(self.id.to_string()),
            NotificationChannel::Sms | NotificationChannel::Vonage => {
                self.phone_number.clone()
            }
            NotificationChannel::Slack => {
                None // TODO: Add slack_user_id field back when needed
            }
            NotificationChannel::Custom(_) => None,
        }
    }

    fn get_key(&self) -> String {
        format!("User_{}", self.id)
    }

    async fn notification_preferences(&self) -> HashMap<String, bool> {
        // Load preferences from database
        match self.load_preferences_from_database().await {
            Ok(prefs) => prefs,
            Err(e) => {
                tracing::warn!("Failed to load user preferences for {}: {}. Using defaults.", self.id, e);
                self.default_preferences()
            }
        }
    }
}

// Helper function to check if a user can receive a specific type of notification
impl User {
    /// Load notification preferences from database
    async fn load_preferences_from_database(&self) -> anyhow::Result<HashMap<String, bool>> {
        // TODO: Implement user preferences with Diesel when needed
        Ok(self.default_preferences())
    }

    /// Get default notification preferences
    fn default_preferences(&self) -> HashMap<String, bool> {
        let mut preferences = HashMap::new();
        preferences.insert("email_notifications".to_string(), true);
        preferences.insert("database_notifications".to_string(), true);
        preferences.insert("broadcast_notifications".to_string(), true);
        preferences.insert("web_push_notifications".to_string(), true);
        preferences.insert("sms_notifications".to_string(), true);
        preferences.insert("slack_notifications".to_string(), true);
        preferences.insert("marketing_emails".to_string(), false);
        preferences.insert("security_alerts".to_string(), true);
        preferences.insert("order_updates".to_string(), true);
        preferences.insert("newsletter".to_string(), false);
        preferences.insert("promotional_emails".to_string(), false);
        preferences.insert("account_notifications".to_string(), true);
        preferences
    }

    /// Update a specific notification preference
    pub async fn update_notification_preference(&self, key: &str, value: bool) -> anyhow::Result<()> {
        // TODO: Implement user preference updates with Diesel when needed
        tracing::info!("Would update preference '{}' to '{}' for user {}", key, value, self.id);
        Ok(())
    }

    /// Disable all marketing communications for this user
    pub async fn disable_marketing_communications(&self) -> anyhow::Result<()> {
        let marketing_prefs = vec![
            "marketing_emails",
            "newsletter",
            "promotional_emails",
        ];

        for pref in marketing_prefs {
            self.update_notification_preference(pref, false).await?;
        }

        tracing::info!("Disabled all marketing communications for user {}", self.id);
        Ok(())
    }
    pub async fn can_receive_notification(&self, notification_type: &str) -> bool {
        let preferences = self.notification_preferences().await;

        // Check specific notification type preferences
        if let Some(&enabled) = preferences.get(notification_type) {
            return enabled;
        }

        // Default to allowing notifications if no specific preference is set
        true
    }

    pub async fn prefers_channel(&self, channel: &NotificationChannel) -> bool {
        let preferences = self.notification_preferences().await;

        match channel {
            NotificationChannel::Mail => {
                preferences.get("email_notifications").unwrap_or(&true).clone()
            }
            NotificationChannel::Database => {
                preferences.get("database_notifications").unwrap_or(&true).clone()
            }
            NotificationChannel::Broadcast => {
                preferences.get("broadcast_notifications").unwrap_or(&true).clone()
            }
            NotificationChannel::WebPush => {
                preferences.get("web_push_notifications").unwrap_or(&true).clone()
            }
            NotificationChannel::Sms | NotificationChannel::Vonage => {
                preferences.get("sms_notifications").unwrap_or(&true).clone()
            }
            NotificationChannel::Slack => {
                preferences.get("slack_notifications").unwrap_or(&true).clone()
            }
            _ => true, // Default to allowing other channels
        }
    }
}

// You can implement Notifiable for other models as well
// For example, if you had an Organization model:
//
// #[async_trait]
// impl Notifiable for Organization {
//     async fn route_notification_for(&self, channel: &NotificationChannel) -> Option<String> {
//         match channel {
//             NotificationChannel::Mail => self.contact_email.clone(),
//             NotificationChannel::Database => Some(self.id.to_string()),
//             NotificationChannel::Slack => self.slack_webhook_url.clone(),
//             _ => None,
//         }
//     }
//
//     fn get_key(&self) -> String {
//         format!("Organization_{}", self.id)
//     }
// }