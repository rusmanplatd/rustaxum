use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::app::notifications::channels::Channel;
use crate::app::notifications::notification::{Notification, Notifiable, NotificationChannel};
use crate::config::Config;

#[derive(Debug, Clone)]
pub struct SmsChannel {
    provider: SmsProvider,
}

#[derive(Debug, Clone)]
pub enum SmsProvider {
    Twilio {
        account_sid: String,
        auth_token: String,
        from_number: String
    },
    Nexmo {
        api_key: String,
        api_secret: String,
        from_number: String
    },
    Log, // For testing
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsMessage {
    pub to: String,
    pub from: String,
    pub message: String,
}

impl SmsChannel {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;

        let provider = match config.notifications.sms_provider.as_str() {
            "twilio" => {
                let account_sid = std::env::var("TWILIO_ACCOUNT_SID")
                    .map_err(|_| anyhow::anyhow!("TWILIO_ACCOUNT_SID not set"))?;
                let auth_token = std::env::var("TWILIO_AUTH_TOKEN")
                    .map_err(|_| anyhow::anyhow!("TWILIO_AUTH_TOKEN not set"))?;
                let from_number = std::env::var("TWILIO_FROM_NUMBER")
                    .map_err(|_| anyhow::anyhow!("TWILIO_FROM_NUMBER not set"))?;

                SmsProvider::Twilio { account_sid, auth_token, from_number }
            },
            "nexmo" => {
                let api_key = std::env::var("NEXMO_API_KEY")
                    .map_err(|_| anyhow::anyhow!("NEXMO_API_KEY not set"))?;
                let api_secret = std::env::var("NEXMO_API_SECRET")
                    .map_err(|_| anyhow::anyhow!("NEXMO_API_SECRET not set"))?;
                let from_number = std::env::var("NEXMO_FROM_NUMBER")
                    .map_err(|_| anyhow::anyhow!("NEXMO_FROM_NUMBER not set"))?;

                SmsProvider::Nexmo { api_key, api_secret, from_number }
            },
            _ => SmsProvider::Log,
        };

        Ok(Self { provider })
    }

    pub fn with_provider(provider: SmsProvider) -> Self {
        Self { provider }
    }

    async fn send_sms(&self, message: SmsMessage) -> Result<()> {
        match &self.provider {
            SmsProvider::Twilio { account_sid, auth_token, .. } => {
                self.send_twilio_sms(message, account_sid, auth_token).await
            },
            SmsProvider::Nexmo { api_key, api_secret, .. } => {
                self.send_nexmo_sms(message, api_key, api_secret).await
            },
            SmsProvider::Log => {
                self.log_sms(message).await
            },
        }
    }

    async fn send_twilio_sms(&self, message: SmsMessage, account_sid: &str, auth_token: &str) -> Result<()> {
        // In a real implementation, you would use the Twilio API
        tracing::info!("Sending SMS via Twilio to {}: {}", message.to, message.message);

        let client = reqwest::Client::new();
        let url = format!("https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json", account_sid);

        let params = [
            ("To", message.to.as_str()),
            ("From", message.from.as_str()),
            ("Body", message.message.as_str()),
        ];

        let response = client
            .post(&url)
            .basic_auth(account_sid, Some(auth_token))
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("SMS sent successfully via Twilio");
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Twilio SMS failed: {}", error_text))
        }
    }

    async fn send_nexmo_sms(&self, message: SmsMessage, api_key: &str, api_secret: &str) -> Result<()> {
        // In a real implementation, you would use the Nexmo/Vonage API
        tracing::info!("Sending SMS via Nexmo to {}: {}", message.to, message.message);

        let client = reqwest::Client::new();
        let url = "https://rest.nexmo.com/sms/json";

        let payload = serde_json::json!({
            "api_key": api_key,
            "api_secret": api_secret,
            "to": message.to,
            "from": message.from,
            "text": message.message
        });

        let response = client
            .post(url)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            tracing::info!("SMS sent successfully via Nexmo");
            Ok(())
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("Nexmo SMS failed: {}", error_text))
        }
    }

    async fn log_sms(&self, message: SmsMessage) -> Result<()> {
        println!("===============================");
        println!("SMS LOG ENTRY");
        println!("===============================");
        println!("To: {}", message.to);
        println!("From: {}", message.from);
        println!("Message: {}", message.message);
        println!("Timestamp: {}", chrono::Utc::now());
        println!("===============================");

        tracing::info!("SMS logged: {} -> {}", message.from, message.to);
        Ok(())
    }

    fn get_from_number(&self) -> String {
        match &self.provider {
            SmsProvider::Twilio { from_number, .. } => from_number.clone(),
            SmsProvider::Nexmo { from_number, .. } => from_number.clone(),
            SmsProvider::Log => "+1234567890".to_string(),
        }
    }
}

impl Default for SmsChannel {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self::with_provider(SmsProvider::Log))
    }
}

#[async_trait]
impl Channel for SmsChannel {
    async fn send(&self, notification: &dyn Notification, notifiable: &dyn Notifiable) -> Result<()> {
        // Get phone number for the notifiable entity
        let phone_number = match notifiable.route_notification_for(&NotificationChannel::Sms).await {
            Some(phone) => phone,
            None => {
                tracing::warn!("No phone number found for notifiable entity: {}", notifiable.get_key());
                return Ok(());
            }
        };

        // Get the SMS message content from notification
        // For now, we'll convert the database message to SMS text
        let database_message = notification.to_database(notifiable).await?;
        let message_text = database_message.data
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("You have a new notification");

        // Create SMS message
        let sms_message = SmsMessage {
            to: phone_number.clone(),
            from: self.get_from_number(),
            message: message_text.to_string(),
        };

        // Send the SMS
        match self.send_sms(sms_message).await {
            Ok(()) => {
                tracing::info!(
                    "SMS notification sent successfully to: {} (type: {})",
                    phone_number,
                    notification.notification_type()
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    "Failed to send SMS notification to {}: {}",
                    phone_number,
                    e
                );
                Err(e)
            }
        }
    }

    fn channel_type(&self) -> NotificationChannel {
        NotificationChannel::Sms
    }
}