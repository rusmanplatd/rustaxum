use anyhow::Result;
use std::fs;
use std::path::Path;

pub async fn generate_mail(name: &str, markdown: bool) -> Result<()> {
    let mail_name = if name.ends_with("Mail") {
        name.to_string()
    } else {
        format!("{}Mail", name)
    };

    let dir_path = "src/app/mail";
    fs::create_dir_all(dir_path)?;

    let file_path = format!("{}/{}.rs", dir_path, to_snake_case(&mail_name));

    let content = if markdown {
        generate_markdown_mail_template(&mail_name)
    } else {
        generate_mail_template(&mail_name)
    };

    fs::write(&file_path, content)?;

    update_mail_mod(&mail_name)?;

    if markdown {
        create_markdown_template(&mail_name)?;
    }

    println!("Mail created successfully: {}", file_path);
    Ok(())
}

fn generate_mail_template(mail_name: &str) -> String {
    let mut content = String::new();
    content.push_str("use anyhow::Result;\n");
    content.push_str("use async_trait::async_trait;\n");
    content.push_str("use crate::app::mail::{Mailable, MailMessage, MailContent};\n\n");

    content.push_str("#[derive(Debug, Clone)]\n");
    content.push_str(&format!("pub struct {} {{\n", mail_name));
    content.push_str("    pub to_email: String,\n");
    content.push_str("    // Add mail data fields here\n");
    content.push_str("}\n\n");

    content.push_str(&format!("impl {} {{\n", mail_name));
    content.push_str("    pub fn new(to_email: String) -> Self {\n");
    content.push_str("        Self {\n");
    content.push_str("            to_email,\n");
    content.push_str("            // Initialize other fields\n");
    content.push_str("        }\n");
    content.push_str("    }\n");
    content.push_str("}\n\n");

    content.push_str("#[async_trait]\n");
    content.push_str(&format!("impl Mailable for {} {{\n", mail_name));
    content.push_str("    async fn build(&self) -> Result<MailMessage> {\n");
    content.push_str("        let content = format!(r#\"\n");
    content.push_str("<!DOCTYPE html>\n");
    content.push_str("<html>\n");
    content.push_str("<head>\n");
    content.push_str("    <title>{} Notification</title>\n");
    content.push_str("</head>\n");
    content.push_str("<body>\n");
    content.push_str("    <h1>{} Notification</h1>\n");
    content.push_str("    <p>This is a notification from your application.</p>\n");
    content.push_str("    <p>Add your email content here.</p>\n");
    content.push_str("</body>\n");
    content.push_str("</html>\n");
    content.push_str(&format!("\"#, \"{}\", \"{}\");\n\n", mail_name, mail_name));

    content.push_str("        Ok(MailMessage::new()\n");
    content.push_str("            .to(self.to_email.clone())\n");
    content.push_str(&format!("            .subject(format!(\"{} Notification\", \"{}\"))\n", "{}", mail_name));
    content.push_str("            .content(MailContent::Html(content)))\n");
    content.push_str("    }\n\n");

    content.push_str("    fn to(&self) -> Vec<String> {\n");
    content.push_str("        vec![self.to_email.clone()]\n");
    content.push_str("    }\n\n");

    content.push_str("    fn subject(&self) -> String {\n");
    content.push_str(&format!("        format!(\"{} Notification\", \"{}\")\n", "{}", mail_name));
    content.push_str("    }\n\n");

    content.push_str("    fn should_queue(&self) -> bool {\n");
    content.push_str("        true\n");
    content.push_str("    }\n\n");

    content.push_str("    fn queue_name(&self) -> Option<&str> {\n");
    content.push_str("        Some(\"emails\")\n");
    content.push_str("    }\n");
    content.push_str("}\n");

    content
}

fn generate_markdown_mail_template(mail_name: &str) -> String {
    let mut content = String::new();
    content.push_str("use anyhow::Result;\n");
    content.push_str("use async_trait::async_trait;\n");
    content.push_str("use crate::app::mail::{Mailable, MailMessage, MailContent};\n\n");

    content.push_str("#[derive(Debug, Clone)]\n");
    content.push_str(&format!("pub struct {} {{\n", mail_name));
    content.push_str("    pub to_email: String,\n");
    content.push_str("    // Add mail data fields here\n");
    content.push_str("}\n\n");

    content.push_str(&format!("impl {} {{\n", mail_name));
    content.push_str("    pub fn new(to_email: String) -> Self {\n");
    content.push_str("        Self {\n");
    content.push_str("            to_email,\n");
    content.push_str("            // Initialize other fields\n");
    content.push_str("        }\n");
    content.push_str("    }\n");
    content.push_str("}\n\n");

    content.push_str("#[async_trait]\n");
    content.push_str(&format!("impl Mailable for {} {{\n", mail_name));
    content.push_str("    async fn build(&self) -> Result<MailMessage> {\n");
    content.push_str("        let markdown_content = format!(r#\"# {} Notification\n");
    content.push_str("\n");
    content.push_str("This is a notification from your application.\n");
    content.push_str("\n");
    content.push_str("## Details\n");
    content.push_str("\n");
    content.push_str("Add your email content here in **Markdown** format.\n");
    content.push_str("\n");
    content.push_str("You can use:\n");
    content.push_str("- *Italic text*\n");
    content.push_str("- **Bold text**\n");
    content.push_str("- Lists\n");
    content.push_str("- Links\n");
    content.push_str("- And more!\n");
    content.push_str("\n");
    content.push_str("Best regards,\n");
    content.push_str("Your Application Team\n");
    content.push_str(&format!("\"#, \"{}\");\n\n", mail_name));

    content.push_str("        Ok(MailMessage::new()\n");
    content.push_str("            .to(self.to_email.clone())\n");
    content.push_str(&format!("            .subject(format!(\"{} Notification\", \"{}\"))\n", "{}", mail_name));
    content.push_str("            .content(MailContent::Markdown {\n");
    content.push_str("                markdown: markdown_content,\n");
    content.push_str("                compiled_html: None,\n");
    content.push_str("            }))\n");
    content.push_str("    }\n\n");

    content.push_str("    fn to(&self) -> Vec<String> {\n");
    content.push_str("        vec![self.to_email.clone()]\n");
    content.push_str("    }\n\n");

    content.push_str("    fn subject(&self) -> String {\n");
    content.push_str(&format!("        format!(\"{} Notification\", \"{}\")\n", "{}", mail_name));
    content.push_str("    }\n\n");

    content.push_str("    fn should_queue(&self) -> bool {\n");
    content.push_str("        true\n");
    content.push_str("    }\n\n");

    content.push_str("    fn queue_name(&self) -> Option<&str> {\n");
    content.push_str("        Some(\"emails\")\n");
    content.push_str("    }\n");
    content.push_str("}\n");

    content
}

fn create_markdown_template(mail_name: &str) -> Result<()> {
    let template_dir = "resources/views/mail";
    fs::create_dir_all(template_dir)?;

    let template_path = format!("{}/{}.md", template_dir, to_snake_case(mail_name));
    let template_content = format!(r#"# {} Notification

This is a notification from your application.

## Details

Add your email content here in Markdown format.

You can use variables like $variable in your template.

Best regards,
Your Application Team
"#, mail_name.replace("Mail", ""));

    fs::write(template_path, template_content)?;
    Ok(())
}

fn update_mail_mod(mail_name: &str) -> Result<()> {
    let mod_path = "src/app/mail/mod.rs";
    let module_name = to_snake_case(mail_name);

    if !Path::new("src/app/mail").exists() {
        fs::create_dir_all("src/app/mail")?;
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