use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::app::notifications::notification::{
    Notification, Notifiable, NotificationChannel, MailMessage, MailContent, DatabaseMessage
};
use crate::app::notifications::channels::web_push_channel::{WebPushMessage, NotificationAction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderShippedNotification {
    pub order_id: String,
    pub tracking_number: String,
    pub customer_name: String,
    pub shipping_company: String,
    pub estimated_delivery: String,
    pub tracking_url: Option<String>,
}

impl OrderShippedNotification {
    pub fn new(
        order_id: String,
        tracking_number: String,
        customer_name: String,
        shipping_company: String,
        estimated_delivery: String,
    ) -> Self {
        Self {
            order_id,
            tracking_number,
            customer_name,
            shipping_company,
            estimated_delivery,
            tracking_url: None,
        }
    }

    pub fn tracking_url(mut self, url: String) -> Self {
        self.tracking_url = Some(url);
        self
    }
}

impl Notification for OrderShippedNotification {
    fn via(&self, _notifiable: &dyn Notifiable) -> Vec<NotificationChannel> {
        vec![
            NotificationChannel::Database,
            NotificationChannel::Mail,
            NotificationChannel::WebPush,
        ]
    }

    fn to_mail(&self, _notifiable: &dyn Notifiable) -> Result<MailMessage> {
        let email = "customer@example.com".to_string(); // TODO: Get from notifiable

        let tracking_link = if let Some(url) = &self.tracking_url {
            format!(r#"<div style="text-align: center; margin: 30px 0;">
                <a href="{}" style="background-color: #3498db; color: white; padding: 12px 24px; text-decoration: none; border-radius: 5px; display: inline-block;">Track Your Package</a>
            </div>"#, url)
        } else {
            format!("<p><strong>Tracking Number:</strong> {}</p>", self.tracking_number)
        };

        let content = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Your Order Has Shipped!</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h2 style="color: #27ae60;">📦 Your Order Has Shipped!</h2>

        <p>Hi <strong>{}</strong>,</p>

        <p>Great news! Your order <strong>#{}</strong> has been shipped and is on its way to you.</p>

        <div style="background-color: #f8f9fa; padding: 20px; border-radius: 5px; margin: 20px 0;">
            <h3 style="margin-top: 0; color: #2c3e50;">Shipping Details</h3>
            <ul style="margin-bottom: 0;">
                <li><strong>Order ID:</strong> #{}</li>
                <li><strong>Tracking Number:</strong> {}</li>
                <li><strong>Shipping Company:</strong> {}</li>
                <li><strong>Estimated Delivery:</strong> {}</li>
            </ul>
        </div>

        {}

        <p>You'll receive another notification once your package has been delivered.</p>

        <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">

        <p style="font-size: 12px; color: #666;">
            Happy shopping!<br>
            The RustAxum Team
        </p>
    </div>
</body>
</html>"#,
            self.customer_name,
            self.order_id,
            self.order_id,
            self.tracking_number,
            self.shipping_company,
            self.estimated_delivery,
            tracking_link
        );

        Ok(MailMessage::new(
            email,
            format!("📦 Order #{} has shipped!", self.order_id),
            MailContent::Html(content),
        ))
    }

    fn to_database(&self, _notifiable: &dyn Notifiable) -> Result<DatabaseMessage> {
        let data = json!({
            "title": "Order Shipped",
            "message": format!("Your order #{} has been shipped via {} and should arrive by {}",
                self.order_id, self.shipping_company, self.estimated_delivery),
            "action_url": self.tracking_url.clone().unwrap_or_else(|| format!("/orders/{}", self.order_id)),
            "type": self.notification_type(),
            "order_id": self.order_id,
            "tracking_number": self.tracking_number,
            "shipping_company": self.shipping_company,
            "estimated_delivery": self.estimated_delivery,
            "tracking_url": self.tracking_url
        });

        Ok(DatabaseMessage::new(data))
    }

    fn to_web_push(&self, _notifiable: &dyn Notifiable) -> Result<WebPushMessage> {
        let mut notification = WebPushMessage::new(
            "📦 Order Shipped!".to_string(),
            format!("Your order #{} has been shipped and is on the way!", self.order_id),
        )
        .icon("/static/images/package-icon.png".to_string())
        .badge("/static/images/shipping-badge.png".to_string())
        .tag(format!("order-shipped-{}", self.order_id))
        .require_interaction(true)
        .data(json!({
            "order_id": self.order_id,
            "tracking_number": self.tracking_number,
            "action_url": self.tracking_url.clone().unwrap_or_else(|| format!("/orders/{}", self.order_id))
        }));

        // Add action buttons
        notification = notification
            .add_action(NotificationAction::new(
                "track".to_string(),
                "Track Package".to_string(),
            ).icon("/static/images/track-icon.png".to_string()))
            .add_action(NotificationAction::new(
                "view-order".to_string(),
                "View Order".to_string(),
            ).icon("/static/images/order-icon.png".to_string()));

        Ok(notification)
    }

    fn notification_type(&self) -> &'static str {
        "OrderShippedNotification"
    }
}

// Example usage:
//
// use crate::app::services::notification_service::NotificationService;
// use crate::app::models::user::User;
// use crate::app::notifications::order_shipped_notification::OrderShippedNotification;
//
// let notification = OrderShippedNotification::new(
//     "ORD-123456".to_string(),
//     "1Z999AA1234567890".to_string(),
//     "John Doe".to_string(),
//     "UPS".to_string(),
//     "December 25, 2025".to_string(),
// ).tracking_url("https://tracking.ups.com/track?id=1Z999AA1234567890".to_string());
//
// let service = NotificationService::new().await;
// service.send(&notification, &user).await?;