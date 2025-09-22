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
    Newsletter { subject: String, content: String },
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

    pub fn newsletter(to_email: String, user_name: String, subject: String, content: String) -> Self {
        Self {
            to_email,
            user_name,
            email_type: EmailType::Newsletter { subject, content },
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

            EmailType::Newsletter { subject, content: _ } => {
                // Create a newsletter email (simplified)
                let welcome_mail = WelcomeMail::new(
                    self.to_email.clone(),
                    self.user_name.clone(),
                );

                manager.send(&welcome_mail).await?;
                tracing::info!("Newsletter '{}' sent to {}", subject, self.to_email);
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
            EmailType::Newsletter { .. } => 10,     // Lowest priority
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
        // TODO: Implement proper audit trail with Diesel when needed
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
            // TODO: Implement AdminEmailFailureNotification when notification system is ready
            tracing::warn!(
                "Email failure for {} should be reported to admin: {}",
                self.to_email,
                admin.email
            );
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
        // TODO: Implement user preference updates with Diesel when needed
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
            EmailType::Newsletter { .. } => "Newsletter".to_string(),
        }
    }

    fn is_critical_email(&self) -> bool {
        matches!(self.email_type, EmailType::PasswordReset { .. } | EmailType::Welcome { .. })
    }

    async fn get_admin_users(&self) -> Result<Vec<crate::app::models::user::User>> {
        // TODO: Implement admin user lookup with Diesel when needed
        // For now, return empty list to avoid errors
        Ok(Vec::new())
    }

    async fn get_user_by_email(&self) -> Result<crate::app::models::user::User> {
        use crate::app::services::user_service::UserService;
        use crate::database::connection::get_connection;

        let pool = get_connection().await?;

        UserService::find_by_email(pool, &self.to_email)?
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
}