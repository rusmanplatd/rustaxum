use anyhow::Result;
use std::fs;
use std::path::Path;

pub async fn generate_notification(name: &str, markdown: bool) -> Result<()> {
    let notification_name = if name.ends_with("Notification") {
        name.to_string()
    } else {
        format!("{}Notification", name)
    };

    let dir_path = "src/app/notifications";
    fs::create_dir_all(dir_path)?;

    let file_path = format!("{}/{}.rs", dir_path, to_snake_case(&notification_name));

    let content = generate_notification_template(&notification_name);

    fs::write(&file_path, content)?;

    update_notifications_mod(&notification_name)?;

    if markdown {
        create_markdown_template(&notification_name)?;
    }

    println!("Notification created successfully: {}", file_path);
    Ok(())
}

fn generate_notification_template(notification_name: &str) -> String {
    format!(r#"use anyhow::Result;
use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {} {{
    pub channels: Vec<NotificationChannel>,
    pub data: NotificationData,
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {{
    Mail,
    Database,
    Broadcast,
    Sms,
    Slack,
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationData {{
    pub title: String,
    pub message: String,
    pub action_url: Option<String>,
    // Add notification-specific data fields here
}}

impl {} {{
    pub fn new(title: String, message: String) -> Self {{
        Self {{
            channels: vec![NotificationChannel::Database],
            data: NotificationData {{
                title,
                message,
                action_url: None,
            }},
        }}
    }}

    pub fn via(mut self, channels: Vec<NotificationChannel>) -> Self {{
        self.channels = channels;
        self
    }}

    pub fn action_url(mut self, url: String) -> Self {{
        self.data.action_url = Some(url);
        self
    }}

    pub async fn send_to(&self, recipient: &str) -> Result<()> {{
        for channel in &self.channels {{
            match channel {{
                NotificationChannel::Mail => self.send_mail(recipient).await?,
                NotificationChannel::Database => self.send_database(recipient).await?,
                NotificationChannel::Broadcast => self.send_broadcast(recipient).await?,
                NotificationChannel::Sms => self.send_sms(recipient).await?,
                NotificationChannel::Slack => self.send_slack(recipient).await?,
            }}
        }}
        Ok(())
    }}

    async fn send_mail(&self, recipient: &str) -> Result<()> {{
        println!("Sending mail notification to: {{}}", recipient);
        println!("Title: {{}}", self.data.title);
        println!("Message: {{}}", self.data.message);
        Ok(())
    }}

    async fn send_database(&self, recipient: &str) -> Result<()> {{
        println!("Storing database notification for: {{}}", recipient);
        println!("Title: {{}}", self.data.title);
        println!("Message: {{}}", self.data.message);
        Ok(())
    }}

    async fn send_broadcast(&self, recipient: &str) -> Result<()> {{
        println!("Broadcasting notification to: {{}}", recipient);
        println!("Title: {{}}", self.data.title);
        println!("Message: {{}}", self.data.message);
        Ok(())
    }}

    async fn send_sms(&self, recipient: &str) -> Result<()> {{
        println!("Sending SMS notification to: {{}}", recipient);
        println!("Message: {{}}", self.data.message);
        Ok(())
    }}

    async fn send_slack(&self, recipient: &str) -> Result<()> {{
        println!("Sending Slack notification to: {{}}", recipient);
        println!("Title: {{}}", self.data.title);
        println!("Message: {{}}", self.data.message);
        Ok(())
    }}
}}
"#, notification_name, notification_name)
}

fn create_markdown_template(notification_name: &str) -> Result<()> {
    let template_dir = "resources/views/notifications";
    fs::create_dir_all(template_dir)?;

    let template_path = format!("{}/{}.md", template_dir, to_snake_case(notification_name));
    let template_content = format!(r#"# {} Notification

This is a notification from your application.

## Details

Add your notification content here in Markdown format.

Best regards,
Your Application Team
"#, notification_name.replace("Notification", ""));

    fs::write(template_path, template_content)?;
    Ok(())
}

fn update_notifications_mod(notification_name: &str) -> Result<()> {
    let mod_path = "src/app/notifications/mod.rs";
    let module_name = to_snake_case(notification_name);

    if !Path::new("src/app/notifications").exists() {
        fs::create_dir_all("src/app/notifications")?;
    }

    let mod_content = if Path::new(mod_path).exists() {
        let existing = fs::read_to_string(mod_path)?;
        if existing.contains(&format!("pub mod {};", module_name)) {
            return Ok(());
        }
        format!("{}\npub mod {};", existing.trim(), module_name)
    } else {
        format!("pub mod {};", module_name)
    };

    fs::write(mod_path, mod_content)?;
    Ok(())
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_is_upper = true;
        } else {
            result.push(c);
            prev_is_upper = false;
        }
    }

    result
}