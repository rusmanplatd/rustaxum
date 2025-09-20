use anyhow::Result;
use async_trait::async_trait;
use crate::app::mail::{Mailable, MailMessage, MailContent};

#[derive(Debug, Clone)]
pub struct OrderShippedMail {
    pub to_email: String,
    pub customer_name: String,
    pub order_number: String,
    pub tracking_number: Option<String>,
    pub shipping_address: String,
    pub items: Vec<OrderItem>,
}

#[derive(Debug, Clone)]
pub struct OrderItem {
    pub name: String,
    pub quantity: u32,
    pub price: f64,
}

impl OrderShippedMail {
    pub fn new(
        to_email: String,
        customer_name: String,
        order_number: String,
        shipping_address: String,
    ) -> Self {
        Self {
            to_email,
            customer_name,
            order_number,
            tracking_number: None,
            shipping_address,
            items: Vec::new(),
        }
    }

    pub fn with_tracking(mut self, tracking_number: String) -> Self {
        self.tracking_number = Some(tracking_number);
        self
    }

    pub fn add_item(mut self, name: String, quantity: u32, price: f64) -> Self {
        self.items.push(OrderItem { name, quantity, price });
        self
    }

    fn calculate_total(&self) -> f64 {
        self.items.iter().map(|item| item.price * item.quantity as f64).sum()
    }
}

#[async_trait]
impl Mailable for OrderShippedMail {
    async fn build(&self) -> Result<MailMessage> {
        let tracking_info = if let Some(ref tracking) = self.tracking_number {
            format!("**Tracking Number:** {}\n\nYou can track your package at: [Track Package](https://example.com/track/{})\n", tracking, tracking)
        } else {
            "You will receive a tracking number via email once your package is picked up by the carrier.\n".to_string()
        };

        let items_list = self.items.iter()
            .map(|item| format!("- {} x{} - ${:.2}", item.name, item.quantity, item.price))
            .collect::<Vec<_>>()
            .join("\n");

        let total = self.calculate_total();

        let markdown_content = format!(r#"# Your Order Has Shipped! 📦

Hi **{}**,

Great news! Your order has been shipped and is on its way to you.

## Order Details

**Order Number:** {}
**Shipping Address:** {}

{}

## Items Ordered

{}

**Total:** ${:.2}

## What's Next?

Your package should arrive within 3-5 business days. If you have any questions about your order, please don't hesitate to contact our support team.

Thank you for your business!

Best regards,
The Shipping Team
"#,
            self.customer_name,
            self.order_number,
            self.shipping_address,
            tracking_info,
            items_list,
            total
        );

        Ok(MailMessage::new()
            .to(self.to_email.clone())
            .subject(format!("Your Order #{} Has Shipped!", self.order_number))
            .content(MailContent::Markdown {
                markdown: markdown_content,
                compiled_html: None,
            }))
    }

    fn to(&self) -> Vec<String> {
        vec![self.to_email.clone()]
    }

    fn subject(&self) -> String {
        format!("Your Order #{} Has Shipped!", self.order_number)
    }

    fn should_queue(&self) -> bool {
        true
    }

    fn queue_name(&self) -> Option<&str> {
        Some("emails")
    }
}