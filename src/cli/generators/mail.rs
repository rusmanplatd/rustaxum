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
use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {} {{
    pub to: String,
    pub subject: String,
    // Add mail data fields here
}}

impl {} {{
    pub fn new(to: String) -> Self {{
        Self {{
            to,
            subject: "{} Notification".to_string(),
            // Initialize other fields
        }}
    }}

    pub fn subject(mut self, subject: impl Into<String>) -> Self {{
        self.subject = subject.into();
        self
    }}

    pub async fn send(&self) -> Result<()> {{
        // Implement mail sending logic here
        // This could integrate with SMTP, SendGrid, etc.
        println!("Sending mail to: {{}}", self.to);
        println!("Subject: {{}}", self.subject);
        println!("Body: {{}}", self.build_body());
        Ok(())
    }}

    fn build_body(&self) -> String {{
        // Build the email body
        "HTML email body placeholder".to_string()
    }}
}}
"#, mail_name, mail_name, mail_name)
}

fn generate_markdown_mail_template(mail_name: &str) -> String {
    format!(r#"use anyhow::Result;
use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {} {{
    pub to: String,
    pub subject: String,
    // Add mail data fields here
}}

impl {} {{
    pub fn new(to: String) -> Self {{
        Self {{
            to,
            subject: "{} Notification".to_string(),
            // Initialize other fields
        }}
    }}

    pub fn subject(mut self, subject: impl Into<String>) -> Self {{
        self.subject = subject.into();
        self
    }}

    pub async fn send(&self) -> Result<()> {{
        // Implement mail sending logic here
        // This could integrate with SMTP, SendGrid, etc.
        let body = self.build_body_from_markdown()?;
        println!("Sending mail to: {{}}", self.to);
        println!("Subject: {{}}", self.subject);
        println!("Body: {{}}", body);
        Ok(())
    }}

    fn build_body_from_markdown(&self) -> Result<String> {{
        // Read and process the markdown template
        Ok("Markdown email body placeholder".to_string())
    }}

    fn default_markdown_content(&self) -> String {{
        "Default markdown content".to_string()
    }}
}}
"#, mail_name, mail_name, mail_name)
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