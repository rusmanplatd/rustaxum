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
                // For Slack notifications, we would need to store slack user IDs
                // This could be implemented as part of user profile or oauth connections
                None // Slack integration would require additional user mapping
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
    /// Load notification preferences from database (Laravel-style implementation)
    /// Uses individual notification preference columns
    async fn load_preferences_from_database(&self) -> anyhow::Result<HashMap<String, bool>> {
        use crate::database::connection::get_connection;
        use diesel::prelude::*;
        use crate::schema::sys_users;

        // Get database connection
        let pool = get_connection().await?;
        let mut conn = pool.get()?;

        // Load notification preferences from individual columns
        let user_prefs = sys_users::table
            .filter(sys_users::id.eq(self.id))
            .select((
                sys_users::email_notifications,
                sys_users::database_notifications,
                sys_users::broadcast_notifications,
                sys_users::web_push_notifications,
                sys_users::sms_notifications,
                sys_users::slack_notifications,
                sys_users::marketing_emails,
                sys_users::security_alerts,
                sys_users::order_updates,
                sys_users::newsletter,
                sys_users::promotional_emails,
                sys_users::account_notifications,
            ))
            .first::<(
                Option<bool>, Option<bool>, Option<bool>, Option<bool>,
                Option<bool>, Option<bool>, Option<bool>, Option<bool>,
                Option<bool>, Option<bool>, Option<bool>, Option<bool>,
            )>(&mut conn)
            .optional()?;

        match user_prefs {
            Some((email_notifications, database_notifications, broadcast_notifications, web_push_notifications,
                  sms_notifications, slack_notifications, marketing_emails, security_alerts,
                  order_updates, newsletter, promotional_emails, account_notifications)) => {

                let mut preferences = HashMap::new();
                preferences.insert("email_notifications".to_string(), email_notifications.unwrap_or(true));
                preferences.insert("database_notifications".to_string(), database_notifications.unwrap_or(true));
                preferences.insert("broadcast_notifications".to_string(), broadcast_notifications.unwrap_or(true));
                preferences.insert("web_push_notifications".to_string(), web_push_notifications.unwrap_or(true));
                preferences.insert("sms_notifications".to_string(), sms_notifications.unwrap_or(false));
                preferences.insert("slack_notifications".to_string(), slack_notifications.unwrap_or(false));
                preferences.insert("marketing_emails".to_string(), marketing_emails.unwrap_or(true));
                preferences.insert("security_alerts".to_string(), security_alerts.unwrap_or(true));
                preferences.insert("order_updates".to_string(), order_updates.unwrap_or(true));
                preferences.insert("newsletter".to_string(), newsletter.unwrap_or(false));
                preferences.insert("promotional_emails".to_string(), promotional_emails.unwrap_or(false));
                preferences.insert("account_notifications".to_string(), account_notifications.unwrap_or(true));

                Ok(preferences)
            },
            None => {
                // User not found, use defaults
                Ok(self.default_preferences())
            }
        }
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

    /// Update a specific notification preference (Laravel-style implementation)
    /// Updates individual notification preference columns in the database
    pub async fn update_notification_preference(&self, key: &str, value: bool) -> anyhow::Result<()> {
        use crate::database::connection::get_connection;
        use diesel::prelude::*;
        use crate::schema::sys_users;

        // Get database connection
        let pool = get_connection().await?;
        let mut conn = pool.get()?;

        // Update the specific preference column based on the key
        let update_result = match key {
            "email_notifications" => {
                diesel::update(sys_users::table.filter(sys_users::id.eq(self.id)))
                    .set(sys_users::email_notifications.eq(Some(value)))
                    .execute(&mut conn)
            },
            "database_notifications" => {
                diesel::update(sys_users::table.filter(sys_users::id.eq(self.id)))
                    .set(sys_users::database_notifications.eq(Some(value)))
                    .execute(&mut conn)
            },
            "broadcast_notifications" => {
                diesel::update(sys_users::table.filter(sys_users::id.eq(self.id)))
                    .set(sys_users::broadcast_notifications.eq(Some(value)))
                    .execute(&mut conn)
            },
            "web_push_notifications" => {
                diesel::update(sys_users::table.filter(sys_users::id.eq(self.id)))
                    .set(sys_users::web_push_notifications.eq(Some(value)))
                    .execute(&mut conn)
            },
            "sms_notifications" => {
                diesel::update(sys_users::table.filter(sys_users::id.eq(self.id)))
                    .set(sys_users::sms_notifications.eq(Some(value)))
                    .execute(&mut conn)
            },
            "slack_notifications" => {
                diesel::update(sys_users::table.filter(sys_users::id.eq(self.id)))
                    .set(sys_users::slack_notifications.eq(Some(value)))
                    .execute(&mut conn)
            },
            "marketing_emails" => {
                diesel::update(sys_users::table.filter(sys_users::id.eq(self.id)))
                    .set(sys_users::marketing_emails.eq(Some(value)))
                    .execute(&mut conn)
            },
            "security_alerts" => {
                diesel::update(sys_users::table.filter(sys_users::id.eq(self.id)))
                    .set(sys_users::security_alerts.eq(Some(value)))
                    .execute(&mut conn)
            },
            "order_updates" => {
                diesel::update(sys_users::table.filter(sys_users::id.eq(self.id)))
                    .set(sys_users::order_updates.eq(Some(value)))
                    .execute(&mut conn)
            },
            "newsletter" => {
                diesel::update(sys_users::table.filter(sys_users::id.eq(self.id)))
                    .set(sys_users::newsletter.eq(Some(value)))
                    .execute(&mut conn)
            },
            "promotional_emails" => {
                diesel::update(sys_users::table.filter(sys_users::id.eq(self.id)))
                    .set(sys_users::promotional_emails.eq(Some(value)))
                    .execute(&mut conn)
            },
            "account_notifications" => {
                diesel::update(sys_users::table.filter(sys_users::id.eq(self.id)))
                    .set(sys_users::account_notifications.eq(Some(value)))
                    .execute(&mut conn)
            },
            _ => {
                return Err(anyhow::anyhow!("Unknown notification preference key: {}", key));
            }
        };

        match update_result {
            Ok(rows_affected) => {
                if rows_affected == 0 {
                    return Err(anyhow::anyhow!("User not found or no changes made"));
                }

                // Log the preference change for audit trail
                use crate::app::services::activity_log_service::ActivityLogService;
                let details = serde_json::json!({
                    "preference_key": key,
                    "preference_value": value,
                    "updated_at": chrono::Utc::now().to_rfc3339(),
                    "rows_affected": rows_affected
                });

                ActivityLogService::log_activity(
                    "preference_update",
                    &format!("Updated notification preference: {}", key),
                    details,
                    Some(&self.id.0.to_string()),
                ).await.ok(); // Don't fail if logging fails

                Ok(())
            },
            Err(e) => Err(anyhow::anyhow!("Failed to update notification preference: {}", e))
        }
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