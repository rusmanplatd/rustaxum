use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::app::jobs::Job;
use crate::app::mail::{mail_manager, WelcomeMail};

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

            EmailType::OrderShipped { order_number, tracking_number } => {
                use crate::app::mail::order_shipped_mail::OrderShippedMail;

                let mut order_mail = OrderShippedMail::new(
                    self.to_email.clone(),
                    self.user_name.clone(),
                    order_number.clone(),
                    "123 Main St, City, State 12345".to_string(), // This would come from order data
                );

                if let Some(tracking) = tracking_number {
                    order_mail = order_mail.with_tracking(tracking.clone());
                }

                // Add sample items (in real app, this would come from order data)
                order_mail = order_mail
                    .add_item("Sample Product 1".to_string(), 2, 29.99)
                    .add_item("Sample Product 2".to_string(), 1, 49.99);

                manager.send(&order_mail).await?;
                tracing::info!("Order shipped email sent to {} for order {}", self.to_email, order_number);
            },

            EmailType::PasswordReset { reset_token } => {
                // For now, we'll create a simple password reset email using WelcomeMail as template
                // In a real app, you'd create a dedicated PasswordResetMail struct
                let welcome_mail = WelcomeMail::new(
                    self.to_email.clone(),
                    self.user_name.clone(),
                )
                .with_activation_link(format!("https://example.com/reset-password?token={}", reset_token));

                manager.send(&welcome_mail).await?;
                tracing::info!("Password reset email sent to {}", self.to_email);
            },

            EmailType::Newsletter { subject, content } => {
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

        // In a real app, you might want to:
        // 1. Store failed email attempts in database
        // 2. Send notification to administrators
        // 3. Create alternative delivery methods
    }
}