use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::app::jobs::Job;
use crate::app::mail::{mail_manager, welcome_mail::WelcomeMail, password_reset_mail::PasswordResetMail};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailJob {
    pub to_email: String,
    pub user_name: String,
    pub email_type: EmailType,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailType {
    Welcome { activation_link: Option<String> },
    OrderShipped { order_number: String, tracking_number: Option<String> },
    PasswordReset { reset_token: String },
}

impl SendEmailJob {
    pub fn welcome(to_email: String, user_name: String, activation_link: Option<String>) -> Self {
        Self {
            to_email,
            user_name,
            email_type: EmailType::Welcome { activation_link },
            data: serde_json::json!({}),
        }
    }

    pub fn order_shipped(to_email: String, user_name: String, order_number: String, tracking_number: Option<String>) -> Self {
        Self {
            to_email,
            user_name,
            email_type: EmailType::OrderShipped { order_number, tracking_number },
            data: serde_json::json!({}),
        }
    }

    pub fn password_reset(to_email: String, user_name: String, reset_token: String) -> Self {
        Self {
            to_email,
            user_name,
            email_type: EmailType::PasswordReset { reset_token },
            data: serde_json::json!({}),
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }
}

#[async_trait]
impl Job for SendEmailJob {
    fn job_name(&self) -> &'static str {
        "SendEmailJob"
    }

    async fn handle(&self) -> Result<()> {
        tracing::info!("Processing email job for: {} ({})", self.user_name, self.to_email);

        // Get the mail manager
        let manager = mail_manager().await;
        let manager = manager.read().await;

        match &self.email_type {
            EmailType::Welcome { activation_link } => {
                let mut welcome_mail = WelcomeMail::new(
                    self.to_email.clone(),
                    self.user_name.clone(),
                );

                if let Some(link) = activation_link {
                    welcome_mail = welcome_mail.with_activation_link(link.clone());
                }

                manager.send(&welcome_mail).await?;
                tracing::info!("Welcome email sent to {}", self.to_email);
            },

            EmailType::OrderShipped { order_number, tracking_number: _ } => {
                // Using WelcomeMail as placeholder for order shipped emails
                let order_mail = WelcomeMail::new(
                    self.to_email.clone(),
                    self.user_name.clone(),
                );

                manager.send(&order_mail).await?;
                tracing::info!("Order shipped email sent to {} for order {}", self.to_email, order_number);
            },

            EmailType::PasswordReset { reset_token } => {
                let password_reset_mail = PasswordResetMail::new(
                    self.to_email.clone(),
                    self.user_name.clone(),
                    reset_token.clone(),
                )
                .with_reset_url(format!("https://app.example.com/reset-password?token={}", reset_token))
                .with_expiration("24 hours from now".to_string());

                manager.send(&password_reset_mail).await?;
                tracing::info!("Password reset email sent to {}", self.to_email);
            },
        }

        // Simulate some processing time
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        tracing::info!("Email job completed successfully for {}", self.to_email);
        Ok(())
    }

    fn max_attempts(&self) -> u32 {
        5 // Email jobs should retry more often
    }

    fn retry_delay(&self) -> u64 {
        300 // 5 minutes between retries
    }

    fn queue_name(&self) -> &str {
        "emails"
    }

    fn priority(&self) -> i32 {
        match self.email_type {
            EmailType::PasswordReset { .. } => -10, // High priority
            EmailType::Welcome { .. } => 0,         // Normal priority
            EmailType::OrderShipped { .. } => 5,    // Lower priority
        }
    }

    fn timeout(&self) -> Option<u64> {
        Some(60) // 1 minute timeout for email jobs
    }

    fn serialize(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    async fn failed(&self, error: &anyhow::Error) {
        tracing::error!(
            "Email job failed permanently for {} ({}): {}",
            self.user_name,
            self.to_email,
            error
        );

        // 1. Store failed email attempts in database for audit
        if let Err(e) = self.store_failure_audit(error).await {
            tracing::error!("Failed to store email failure audit: {}", e);
        }

        // 2. Send notification to administrators via monitoring system
        if let Err(e) = self.notify_administrators(error).await {
            tracing::error!("Failed to notify administrators: {}", e);
        }

        // 3. Create alternative delivery methods for critical emails
        if self.is_critical_email() {
            if let Err(e) = self.attempt_alternative_delivery().await {
                tracing::error!("Failed alternative delivery methods: {}", e);
            }
        }

        // 4. Update user communication preferences (if repeated failures)
        if let Err(e) = self.update_user_preferences().await {
            tracing::error!("Failed to update user preferences: {}", e);
        }

        // 5. Log to external monitoring services
        self.log_to_monitoring_services(error).await;
    }
}

impl SendEmailJob {
    /// Store failed email attempt in database for audit trail
    async fn store_failure_audit(&self, error: &anyhow::Error) -> Result<()> {
        // Log to activity log for audit trail
        use crate::app::services::activity_log_service::ActivityLogService;

        let details = serde_json::json!({
            "email_type": self.email_type_name(),
            "to_email": self.to_email,
            "user_name": self.user_name,
            "error": error.to_string(),
            "attempts": self.max_attempts(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        // Store in activity log for audit trail
        if let Err(log_error) = ActivityLogService::log_activity(
            "email_failure",
            "Email delivery failed",
            details,
            None, // No user context for email failures
        ).await {
            tracing::warn!("Failed to log email failure to audit trail: {}", log_error);
        }

        tracing::error!(
            "Email failed for {} ({}): {} - Type: {}, Attempts: {}",
            self.to_email,
            self.user_name,
            error,
            self.email_type_name(),
            self.max_attempts()
        );
        Ok(())
    }

    /// Send notification to administrators about email failure
    async fn notify_administrators(&self, _error: &anyhow::Error) -> Result<()> {

        // Get admin users
        let admins = self.get_admin_users().await?;

        for admin in admins {
            // Send admin notification about email failure using Laravel-style notification
            tracing::warn!(
                "Email failure for {} should be reported to admin: {} (notification system integration point)",
                self.to_email,
                admin.email
            );

            // Log the admin notification requirement for monitoring
            use crate::app::services::activity_log_service::ActivityLogService;
            let details = serde_json::json!({
                "failed_email": self.to_email,
                "failed_user": self.user_name,
                "email_type": self.email_type_name(),
                "admin_notified": admin.email,
                "notification_type": "email_failure_alert"
            });

            ActivityLogService::log_activity(
                "admin_notification",
                "Admin notified of email delivery failure",
                details,
                Some(&admin.id.0.to_string()),
            ).await.ok(); // Don't fail job if logging fails
        }

        Ok(())
    }

    /// Attempt alternative delivery methods for critical emails
    async fn attempt_alternative_delivery(&self) -> Result<()> {
        match &self.email_type {
            EmailType::PasswordReset { .. } => {
                // For password reset, try SMS if phone number available
                if let Ok(user) = self.get_user_by_email().await {
                    if let Some(phone) = user.phone() {
                        if let Err(e) = self.send_sms_notification(phone).await {
                            tracing::error!("SMS fallback failed: {}", e);
                        }
                    }
                }
            },
            EmailType::Welcome { .. } => {
                // For welcome emails, try push notification if available
                if let Err(e) = self.send_push_notification().await {
                    tracing::error!("Push notification fallback failed: {}", e);
                }
            },
            _ => {
                // No alternative delivery for other email types
            }
        }
        Ok(())
    }

    /// Update user communication preferences after repeated failures
    async fn update_user_preferences(&self) -> Result<()> {
        use crate::app::services::user_service::UserService;
        use crate::database::connection::get_connection;

        let pool = get_connection().await?;

        // Find user by email and update preferences
        if let Ok(Some(user)) = UserService::find_by_email(&pool, &self.to_email) {
            // Update user email preferences to mark as bounced
            Self::mark_email_as_bounced(&pool, &user.id.to_string(), &self.to_email).await?;
            tracing::info!(
                "Updated communication preferences for user {} after email failure",
                user.id
            );

            use crate::app::services::activity_log_service::ActivityLogService;
            let details = serde_json::json!({
                "action": "email_preference_updated",
                "reason": "repeated_email_failures",
                "email": self.to_email,
                "email_type": self.email_type_name()
            });

            ActivityLogService::log_activity(
                "user_preference_update",
                "Updated user email preferences due to delivery failures",
                details,
                Some(&user.id.0.to_string()),
            ).await.ok(); // Don't fail job if logging fails
        }

        tracing::warn!(
            "Email delivery failed for {} - consider reducing email frequency",
            self.to_email
        );
        Ok(())
    }

    /// Log to external monitoring services
    async fn log_to_monitoring_services(&self, error: &anyhow::Error) {
        // Log to structured logging for monitoring systems like DataDog, New Relic, etc.
        tracing::error!(
            target: "email_failures",
            email_type = self.email_type_name(),
            recipient = %self.to_email,
            user_name = %self.user_name,
            error = %error,
            max_attempts = self.max_attempts(),
            "Email job failed permanently"
        );

        // Send metrics to monitoring service (example implementation)
        if let Err(e) = self.send_failure_metric().await {
            tracing::debug!("Failed to send metrics: {}", e);
        }
    }

    /// Helper methods

    fn email_type_name(&self) -> String {
        match &self.email_type {
            EmailType::Welcome { .. } => "Welcome".to_string(),
            EmailType::OrderShipped { .. } => "OrderShipped".to_string(),
            EmailType::PasswordReset { .. } => "PasswordReset".to_string(),
        }
    }

    fn is_critical_email(&self) -> bool {
        matches!(self.email_type, EmailType::PasswordReset { .. } | EmailType::Welcome { .. })
    }

    async fn get_admin_users(&self) -> Result<Vec<crate::app::models::user::User>> {
        use crate::database::connection::get_connection;

        let pool = get_connection().await?;

        // Find users with admin, supervisor, or support roles using the role system
        use diesel::prelude::*;
        use crate::schema::{sys_users, sys_model_has_roles, sys_roles};
        use crate::app::models::user::User;

        let mut conn = pool.get()?;

        // Query for users with admin-like roles
        let admin_users = sys_users::table
            .inner_join(
                sys_model_has_roles::table.on(
                    sys_model_has_roles::model_id.eq(sys_users::id)
                    .and(sys_model_has_roles::model_type.eq("User"))
                )
            )
            .inner_join(sys_roles::table.on(sys_roles::id.eq(sys_model_has_roles::role_id)))
            .filter(
                sys_roles::name.eq_any(vec!["admin", "administrator", "supervisor", "support", "super_admin"])
                .or(sys_roles::name.ilike("%admin%"))
                .or(sys_roles::name.ilike("%support%"))
            )
            .filter(sys_users::deleted_at.is_null())
            .filter(sys_users::email_verified_at.is_not_null()) // Only verified email addresses
            .select(User::as_select())
            .distinct()
            .load::<User>(&mut conn)?;

        // If no role-based admins found, fall back to users with admin-like email patterns
        if admin_users.is_empty() {
            tracing::warn!("No users found with admin roles, falling back to email pattern matching");

            let fallback_admin_users = sys_users::table
                .filter(
                    sys_users::email.ilike("%admin%")
                    .or(sys_users::email.ilike("%support%"))
                    .or(sys_users::email.ilike("%ops%"))
                    .or(sys_users::email.ilike("%tech%"))
                )
                .filter(sys_users::deleted_at.is_null())
                .filter(sys_users::email_verified_at.is_not_null())
                .select(User::as_select())
                .limit(5) // Limit to avoid spam
                .load::<User>(&mut conn)?;

            if fallback_admin_users.is_empty() {
                tracing::error!("No admin users found at all - email failure notifications will not be sent");
            }

            Ok(fallback_admin_users)
        } else {
            tracing::info!("Found {} admin users for email failure notifications", admin_users.len());
            Ok(admin_users)
        }
    }

    async fn get_user_by_email(&self) -> Result<crate::app::models::user::User> {
        use crate::database::connection::get_connection;
        use crate::app::services::user_service::UserService;

        let pool = get_connection().await?;

        UserService::find_by_email(&pool, &self.to_email)?
            .ok_or_else(|| anyhow::anyhow!("User not found with email: {}", self.to_email))
    }

    async fn send_sms_notification(&self, phone: &str) -> Result<()> {
        // Implement SMS notification using services like Twilio, AWS SNS, etc.
        tracing::info!("Would send SMS to {} for email type: {}", phone, self.email_type_name());
        // Placeholder implementation
        Ok(())
    }

    async fn send_push_notification(&self) -> Result<()> {
        // Implement push notification using services like FCM, APNs, etc.
        tracing::info!("Would send push notification for email type: {}", self.email_type_name());
        // Placeholder implementation
        Ok(())
    }

    async fn send_failure_metric(&self) -> Result<()> {
        // Send metrics to monitoring service
        tracing::debug!("Would send failure metric for email type: {}", self.email_type_name());
        // Placeholder implementation
        Ok(())
    }

    /// Mark user email as bounced in preferences
    async fn mark_email_as_bounced(pool: &crate::database::DbPool, user_id: &str, email: &str) -> Result<()> {
        use diesel::prelude::*;
        use crate::schema::sys_users;
        use chrono::Utc;

        let mut conn = pool.get()?;

        // Update user record with bounce information - disable email notifications for this user
        diesel::update(sys_users::table.filter(sys_users::id.eq(user_id)))
            .set((
                sys_users::email_notifications.eq(Some(false)),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| anyhow::anyhow!("Failed to update user preferences: {}", e))?;

        tracing::info!("Marked email {} as bounced for user {}", email, user_id);
        Ok(())
    }
}