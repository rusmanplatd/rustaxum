use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::app::notifications::notification::{
    Notification, Notifiable, NotificationChannel, MailMessage, MailContent, DatabaseMessage
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoicePaidNotification {
    pub invoice_id: String,
    pub amount: f64,
    pub currency: String,
    pub paid_at: chrono::DateTime<chrono::Utc>,
}

impl InvoicePaidNotification {
    pub fn new(invoice_id: String, amount: f64, currency: String) -> Self {
        Self {
            invoice_id,
            amount,
            currency,
            paid_at: chrono::Utc::now(),
        }
    }
}

#[async_trait]
impl Notification for InvoicePaidNotification {
    async fn via(&self, _notifiable: &dyn Notifiable) -> Vec<NotificationChannel> {
        vec![
            NotificationChannel::Database,
            NotificationChannel::Mail,
        ]
    }

    async fn to_mail(&self, notifiable: &dyn Notifiable) -> Result<MailMessage> {
        let email = notifiable.route_notification_for(&NotificationChannel::Mail)
            .await
            .unwrap_or_else(|| "user@example.com".to_string());

        let content = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Invoice Payment Received</title>
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
    <div style="max-width: 600px; margin: 0 auto; padding: 20px;">
        <h2 style="color: #27ae60;">Payment Received!</h2>

        <p>Good news! We have received your payment for invoice <strong>#{}</strong>.</p>

        <div style="background-color: #d4edda; color: #155724; padding: 15px; border-radius: 5px; margin: 20px 0; border-left: 4px solid #27ae60;">
            <h3 style="margin-top: 0;">Payment Details</h3>
            <ul style="margin-bottom: 0;">
                <li><strong>Invoice ID:</strong> #{}</li>
                <li><strong>Amount:</strong> {:.2} {}</li>
                <li><strong>Paid At:</strong> {}</li>
            </ul>
        </div>

        <p>Thank you for your prompt payment. Your account has been updated accordingly.</p>

        <hr style="border: none; border-top: 1px solid #eee; margin: 30px 0;">

        <p style="font-size: 12px; color: #666;">
            Best regards,<br>
            The RustAxum Team
        </p>
    </div>
</body>
</html>"#,
            self.invoice_id,
            self.invoice_id,
            self.amount,
            self.currency,
            self.paid_at.format("%Y-%m-%d %H:%M:%S UTC")
        );

        Ok(MailMessage::new(
            email,
            format!("Payment Received - Invoice #{}", self.invoice_id),
            MailContent::Html(content),
        ))
    }

    async fn to_database(&self, _notifiable: &dyn Notifiable) -> Result<DatabaseMessage> {
        let data = json!({
            "title": "Payment Received",
            "message": format!("Payment of {:.2} {} received for invoice #{}",
                self.amount, self.currency, self.invoice_id),
            "action_url": format!("/invoices/{}", self.invoice_id),
            "type": self.notification_type(),
            "invoice_id": self.invoice_id,
            "amount": self.amount,
            "currency": self.currency,
            "paid_at": self.paid_at
        });

        Ok(DatabaseMessage::new(data))
    }

    fn notification_type(&self) -> &'static str {
        "InvoicePaidNotification"
    }
}