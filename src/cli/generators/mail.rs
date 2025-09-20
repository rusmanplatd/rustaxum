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
    format!(r#"use anyhow::Result;
use async_trait::async_trait;
use crate::app::mail::{{Mailable, MailMessage, MailContent}};

#[derive(Debug, Clone)]
pub struct {} {{
    pub to_email: String,
    // Add mail data fields here
}}

impl {} {{
    pub fn new(to_email: String) -> Self {{
        Self {{
            to_email,
            // Initialize other fields
        }}
    }}
}}

#[async_trait]
impl Mailable for {} {{
    async fn build(&self) -> Result<MailMessage> {{
        let content = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>{} Notification</title>
</head>
<body>
    <h1>{} Notification</h1>
    <p>This is a notification from your application.</p>
    <p>Add your email content here.</p>
</body>
</html>
"#);

        Ok(MailMessage::new()
            .to(self.to_email.clone())
            .subject("{} Notification".to_string())
            .content(MailContent::Html(content)))
    }}

    fn to(&self) -> Vec<String> {{
        vec![self.to_email.clone()]
    }}

    fn subject(&self) -> String {{
        "{} Notification".to_string()
    }}

    fn should_queue(&self) -> bool {{
        true
    }}

    fn queue_name(&self) -> Option<&str> {{
        Some("emails")
    }}
}}
"#, mail_name, mail_name, mail_name, mail_name, mail_name, mail_name, mail_name)
}

fn generate_markdown_mail_template(mail_name: &str) -> String {
    format!(r#"use anyhow::Result;
use async_trait::async_trait;
use crate::app::mail::{{Mailable, MailMessage, MailContent}};

#[derive(Debug, Clone)]
pub struct {} {{
    pub to_email: String,
    // Add mail data fields here
}}

impl {} {{
    pub fn new(to_email: String) -> Self {{
        Self {{
            to_email,
            // Initialize other fields
        }}
    }}
}}

#[async_trait]
impl Mailable for {} {{
    async fn build(&self) -> Result<MailMessage> {{
        let markdown_content = format!(r#"# {} Notification

This is a notification from your application.

## Details

Add your email content here in **Markdown** format.

You can use:
- *Italic text*
- **Bold text**
- Lists
- Links
- And more!

Best regards,
Your Application Team
"#);

        Ok(MailMessage::new()
            .to(self.to_email.clone())
            .subject("{} Notification".to_string())
            .content(MailContent::Markdown {{
                markdown: markdown_content,
                compiled_html: None,
            }}))
    }}

    fn to(&self) -> Vec<String> {{
        vec![self.to_email.clone()]
    }}

    fn subject(&self) -> String {{
        "{} Notification".to_string()
    }}

    fn should_queue(&self) -> bool {{
        true
    }}

    fn queue_name(&self) -> Option<&str> {{
        Some("emails")
    }}
}}
"#, mail_name, mail_name, mail_name, mail_name, mail_name, mail_name)
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