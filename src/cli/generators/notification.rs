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
    let mut template = String::new();

    template.push_str("use anyhow::Result;\n");
    template.push_str("use serde::{Deserialize, Serialize};\n");
    template.push_str("use serde_json::json;\n");
    template.push_str("use crate::app::notifications::{\n");
    template.push_str("    Notification, Notifiable, NotificationChannel, MailMessage, MailContent, DatabaseMessage,\n");
    template.push_str("    ShouldQueue, Queueable\n");
    template.push_str("};\n\n");

    template.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    template.push_str(&format!("pub struct {} {{\n", notification_name));
    template.push_str("    pub title: String,\n");
    template.push_str("    pub message: String,\n");
    template.push_str("    pub action_url: Option<String>,\n");
    template.push_str("    // Add your notification-specific data fields here\n");
    template.push_str("}\n\n");

    template.push_str(&format!("impl {} {{\n", notification_name));
    template.push_str("    pub fn new(title: String, message: String) -> Self {\n");
    template.push_str("        Self {\n");
    template.push_str("            title,\n");
    template.push_str("            message,\n");
    template.push_str("            action_url: None,\n");
    template.push_str("        }\n");
    template.push_str("    }\n\n");
    template.push_str("    pub fn action_url(mut self, url: String) -> Self {\n");
    template.push_str("        self.action_url = Some(url);\n");
    template.push_str("        self\n");
    template.push_str("    }\n");
    template.push_str("}\n\n");

    template.push_str("#[async_trait]\n");
    template.push_str(&format!("impl Notification for {} {{\n", notification_name));
    template.push_str("    async fn via(&self, _notifiable: &dyn Notifiable) -> Vec<NotificationChannel> {\n");
    template.push_str("        vec![\n");
    template.push_str("            NotificationChannel::Database,\n");
    template.push_str("            NotificationChannel::Mail,\n");
    template.push_str("            NotificationChannel::WebPush,\n");
    template.push_str("        ]\n");
    template.push_str("    }\n\n");
    template.push_str("    fn to_mail(&self, notifiable: &dyn Notifiable) -> Result<MailMessage> {\n");
    template.push_str("        // Note: In a real implementation, you would need to make this async\n");
    template.push_str("        let email = notifiable.route_notification_for(&NotificationChannel::Mail)\n");
    template.push_str("            .unwrap_or_else(|| \"noreply@example.com\".to_string());\n\n");
    template.push_str("        let content = format!(r#\"<!DOCTYPE html>\n");
    template.push_str("<html>\n");
    template.push_str("<head>\n");
    template.push_str("    <meta charset=\"utf-8\">\n");
    template.push_str("    <title>{}</title>\n");
    template.push_str("</head>\n");
    template.push_str("<body style=\"font-family: Arial, sans-serif; line-height: 1.6; color: #333;\">\n");
    template.push_str("    <div style=\"max-width: 600px; margin: 0 auto; padding: 20px;\">\n");
    template.push_str("        <h2 style=\"color: #2c3e50;\">{}</h2>\n");
    template.push_str("        <p>{}</p>\n");
    template.push_str("        <p>Best regards,<br>Your Application Team</p>\n");
    template.push_str("    </div>\n");
    template.push_str("</body>\n");
    template.push_str("</html>\"#, self.title, self.title, self.message);\n\n");
    template.push_str("        Ok(MailMessage::new(\n");
    template.push_str("            email,\n");
    template.push_str("            self.title.clone(),\n");
    template.push_str("            MailContent::Html(content),\n");
    template.push_str("        ))\n");
    template.push_str("    }\n\n");
    template.push_str("    fn to_database(&self, _notifiable: &dyn Notifiable) -> Result<DatabaseMessage> {\n");
    template.push_str("        let data = json!({\n");
    template.push_str("            \"title\": self.title,\n");
    template.push_str("            \"message\": self.message,\n");
    template.push_str("            \"action_url\": self.action_url,\n");
    template.push_str("            \"type\": self.notification_type()\n");
    template.push_str("        });\n\n");
    template.push_str("        Ok(DatabaseMessage::new(data))\n");
    template.push_str("    }\n\n");
    template.push_str("    fn notification_type(&self) -> &'static str {\n");
    template.push_str(&format!("        \"{}\"\n", notification_name));
    template.push_str("    }\n");
    template.push_str("}\n");

    template
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