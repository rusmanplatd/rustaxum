use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::app::notifications::notification::{
    Notification, Notifiable, NotificationChannel, MailMessage, MailContent, DatabaseMessage
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelcomeNotification {
    pub user_name: String,
    pub app_url: String,
}

impl WelcomeNotification {
    pub fn new(user_name: String, app_url: String) -> Self {
        Self {
            user_name,
            app_url,
        }
    }
}

impl Notification for WelcomeNotification {
    fn via(&self, _notifiable: &dyn Notifiable) -> Vec<NotificationChannel> {
        vec![
            NotificationChannel::Database,
            NotificationChannel::Mail,
            NotificationChannel::Broadcast,
            NotificationChannel::WebPush,
        ]
    }

    fn to_mail(&self, _notifiable: &dyn Notifiable) -> Result<MailMessage> {
        let email = "user@example.com".to_string(); // TODO: Get from notifiable

        let content = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Welcome to RustAxum!</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h2 style="color: #2c3e50;">Welcome to RustAxum!</h2>

        <p>Hello <strong>{}</strong>,</p>

        <p>Welcome to RustAxum! Your account has been successfully created and you're now ready to explore our platform.</p>

        <div style="background-color: #f8f9fa; padding: 20px; border-radius: 5px; margin: 20px 0;">
            <h3 style="margin-top: 0; color: #2c3e50;">Getting Started</h3>
            <ul>
                <li>Complete your profile setup</li>
                <li>Explore the dashboard</li>
                <li>Check out our documentation</li>
            </ul>
        </div>

        <div style="text-align: center; margin: 30px 0;">
            <a href="{}" style="background-color: #3498db; color: white; padding: 12px 24px; text-decoration: none; border-radius: 5px; display: inline-block;">Get Started</a>
        </div>

        <p>If you have any questions or need assistance, please don't hesitate to contact our support team.</p>

        <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">

        <p style="font-size: 12px; color: #666;">
            Thank you for joining us!<br>
            The RustAxum Team
        </p>
    </div>
</body>
</html>"#,
            self.user_name,
            self.app_url
        );

        Ok(MailMessage::new(
            email,
            "Welcome to RustAxum!".to_string(),
            MailContent::Html(content),
        ))
    }

    fn to_database(&self, _notifiable: &dyn Notifiable) -> Result<DatabaseMessage> {
        let data = json!({
            "title": "Welcome to RustAxum!",
            "message": format!("Welcome {}! Your account has been successfully created.", self.user_name),
            "action_url": self.app_url,
            "type": self.notification_type(),
            "user_name": self.user_name
        });

        Ok(DatabaseMessage::new(data))
    }

    fn notification_type(&self) -> &'static str {
        "WelcomeNotification"
    }
}