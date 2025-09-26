# Mail System Documentation

## Overview

The RustAxum framework includes a comprehensive Laravel-inspired mail system that provides a clean, expressive API for sending emails. The mail system supports multiple drivers, content types, attachments, and queue integration for reliable email delivery.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Mail Drivers](#mail-drivers)
- [Creating Mail Classes](#creating-mail-classes)
- [Content Types](#content-types)
- [Attachments](#attachments)
- [Queue Integration](#queue-integration)
- [Artisan CLI Commands](#artisan-cli-commands)
- [Advanced Features](#advanced-features)
- [Troubleshooting](#troubleshooting)
- [Best Practices](#best-practices)

## Quick Start

### 1. Configure Mail Settings

Set up your mail configuration in your `.env` file:

```env
MAIL_MAILER=smtp
MAIL_HOST=smtp.mailtrap.io
MAIL_PORT=2525
MAIL_USERNAME=your_username
MAIL_PASSWORD=your_password
MAIL_ENCRYPTION=tls
MAIL_FROM_ADDRESS=noreply@yourapp.com
MAIL_FROM_NAME="Your App Name"
```

### 2. Generate a Mail Class

Use the Artisan CLI to generate a new mail class:

```bash
cargo run --bin artisan -- make mail WelcomeEmail --markdown
```

### 3. Send an Email

```rust
use crate::app::mail::{WelcomeMail, mail};

// Create and send a welcome email
let welcome_email = WelcomeMail::new(
    "user@example.com".to_string(),
    "John Doe".to_string()
);

mail(&welcome_email).await?;
```

## Configuration

### Environment Variables

The mail system is configured through environment variables in your `.env` file:

| Variable               | Default                | Description                              |
| ---------------------- | ---------------------- | ---------------------------------------- |
| `MAIL_MAILER`          | `smtp`                 | Mail driver to use (`smtp`, `log`)       |
| `MAIL_HOST`            | `localhost`            | SMTP server hostname                     |
| `MAIL_PORT`            | `1025`                 | SMTP server port                         |
| `MAIL_USERNAME`        | ``                     | SMTP authentication username             |
| `MAIL_PASSWORD`        | ``                     | SMTP authentication password             |
| `MAIL_ENCRYPTION`      | `tls`                  | Encryption method (`tls`, `ssl`, `none`) |
| `MAIL_FROM_ADDRESS`    | `noreply@rustaxum.com` | Default sender email address             |
| `MAIL_FROM_NAME`       | `RustAxum`             | Default sender name                      |
| `MAIL_TIMEOUT_SECONDS` | `30`                   | Connection timeout in seconds            |

### Configuration Structure

The mail configuration is handled by the `MailConfig` struct in `src/config/mail.rs`:

```rust
#[derive(Debug, Clone)]
pub struct MailConfig {
    pub mailer: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub encryption: String,
    pub from_address: String,
    pub from_name: String,
    pub timeout_seconds: u64,
}
```

## Mail Drivers

### SMTP Driver

The SMTP driver provides full-featured email delivery through SMTP servers with support for authentication and encryption.

**Features:**

- TLS/StartTLS encryption support
- Authentication (username/password)
- Multipart messages (text + HTML)
- Attachment support
- Custom headers

**Configuration Example:**

```env
MAIL_MAILER=smtp
MAIL_HOST=smtp.gmail.com
MAIL_PORT=587
MAIL_USERNAME=your-email@gmail.com
MAIL_PASSWORD=your-app-password
MAIL_ENCRYPTION=tls
```

### Log Driver

The log driver writes email content to the application logs instead of sending actual emails. This is useful for development and testing.

**Features:**

- Logs email content to tracing output
- No actual email delivery
- Useful for development/testing

**Configuration Example:**

```env
MAIL_MAILER=log
```

### Adding Custom Drivers

To create a custom mail driver, implement the `MailDriver` trait:

```rust
use async_trait::async_trait;
use crate::app::mail::{MailDriver, MailMessage};

pub struct CustomDriver {
    // Driver configuration
}

#[async_trait]
impl MailDriver for CustomDriver {
    async fn send(&self, message: MailMessage) -> Result<()> {
        // Implement your custom sending logic
        Ok(())
    }

    fn driver_name(&self) -> &'static str {
        "custom"
    }
}
```

## Creating Mail Classes

### Using Artisan CLI

Generate a new mail class using the Artisan CLI:

```bash
# Generate a basic mail class
cargo run --bin artisan -- make mail OrderConfirmation

# Generate a markdown-based mail class
cargo run --bin artisan -- make mail WelcomeEmail --markdown
```

### Manual Implementation

Implement the `Mailable` trait for your mail class:

```rust
use anyhow::Result;
use async_trait::async_trait;
use crate::app::mail::{Mailable, MailMessage, MailContent};

#[derive(Debug, Clone)]
pub struct OrderConfirmationMail {
    pub to_email: String,
    pub order_id: String,
    pub customer_name: String,
    pub total_amount: String,
}

impl OrderConfirmationMail {
    pub fn new(to_email: String, order_id: String, customer_name: String, total_amount: String) -> Self {
        Self {
            to_email,
            order_id,
            customer_name,
            total_amount,
        }
    }
}

#[async_trait]
impl Mailable for OrderConfirmationMail {
    async fn build(&self) -> Result<MailMessage> {
        let html_content = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Order Confirmation</title>
</head>
<body>
    <h1>Order Confirmation</h1>
    <p>Dear {},</p>
    <p>Your order #{} has been confirmed.</p>
    <p>Total Amount: {}</p>
    <p>Thank you for your purchase!</p>
</body>
</html>
"#, self.customer_name, self.order_id, self.total_amount);

        Ok(MailMessage::new()
            .to(self.to_email.clone())
            .subject(format!("Order Confirmation - #{}", self.order_id))
            .content(MailContent::Html(html_content)))
    }

    fn to(&self) -> Vec<String> {
        vec![self.to_email.clone()]
    }

    fn subject(&self) -> String {
        format!("Order Confirmation - #{}", self.order_id)
    }

    fn should_queue(&self) -> bool {
        true
    }

    fn queue_name(&self) -> Option<&str> {
        Some("emails")
    }
}
```

## Content Types

### HTML Content

Send rich HTML emails with full styling support:

```rust
let html_content = r#"
<!DOCTYPE html>
<html>
<head>
    <style>
        .header { background-color: #f0f0f0; padding: 20px; }
        .content { padding: 20px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Welcome to Our Platform!</h1>
    </div>
    <div class="content">
        <p>Thank you for joining us.</p>
    </div>
</body>
</html>
"#;

MailMessage::new()
    .content(MailContent::Html(html_content.to_string()))
```

### Markdown Content

Use Markdown for clean, readable email content that compiles to HTML:

```rust
let markdown_content = r#"
# Welcome to Our Platform!

Thank you for joining us, **John Doe**.

## Getting Started

Here are some next steps:
- Complete your profile
- Explore our features
- Join our community

[Get Started](https://yourapp.com/onboarding)

Best regards,
The Team
"#;

MailMessage::new()
    .content(MailContent::Markdown {
        markdown: markdown_content.to_string(),
        compiled_html: None,
    })
```

### Plain Text Content

For simple text-based emails:

```rust
let text_content = "Welcome to our platform! Thank you for joining us.";

MailMessage::new()
    .content(MailContent::Text(text_content.to_string()))
```

### Multipart Content

Send both text and HTML versions for maximum compatibility:

```rust
MailMessage::new()
    .content(MailContent::Multipart {
        text: "Welcome to our platform!".to_string(),
        html: "<h1>Welcome to our platform!</h1>".to_string(),
    })
```

### Template Content (Future Feature)

Template-based content with data binding:

```rust
MailMessage::new()
    .content(MailContent::Template {
        name: "welcome-email".to_string(),
        data: serde_json::json!({
            "user_name": "John Doe",
            "activation_link": "https://example.com/activate"
        }),
    })
```

## Attachments

### File Attachments

Attach files from the filesystem:

```rust
use crate::app::mail::{Attachment, AttachmentData};

let attachment = Attachment::from_path(
    "invoice.pdf".to_string(),
    "/path/to/invoice.pdf".to_string(),
    Some("application/pdf".to_string())
);

MailMessage::new()
    .attach(attachment)
```

### Binary Data Attachments

Attach binary data directly:

```rust
let pdf_data = vec![0x25, 0x50, 0x44, 0x46]; // PDF bytes

let attachment = Attachment::from_bytes(
    "document.pdf".to_string(),
    pdf_data,
    Some("application/pdf".to_string())
);

MailMessage::new()
    .attach(attachment)
```

### Base64 Attachments

Attach base64-encoded content:

```rust
let base64_data = "JVBERi0xLjQKJcOkw7zDssOkw7bDtwo..."; // Base64 PDF

let attachment = Attachment::from_base64(
    "report.pdf".to_string(),
    base64_data.to_string(),
    Some("application/pdf".to_string())
);

MailMessage::new()
    .attach(attachment)
```

## Queue Integration

### Enabling Queue Support

Mark your mail class to be queued for background processing:

```rust
#[async_trait]
impl Mailable for WelcomeMail {
    fn should_queue(&self) -> bool {
        true
    }

    fn queue_name(&self) -> Option<&str> {
        Some("emails")
    }
}
```

### Priority Levels

Set email priority (1-5, where 1 is highest):

```rust
#[async_trait]
impl Mailable for UrgentNotificationMail {
    fn priority(&self) -> Option<u8> {
        Some(1) // Highest priority
    }
}
```

## Artisan CLI Commands

### Generate Mail Classes

```bash
# Generate a basic mail class
cargo run --bin artisan -- make mail OrderShipped

# Generate a markdown-based mail class
cargo run --bin artisan -- make mail WelcomeEmail --markdown

# Generate with custom name
cargo run --bin artisan -- make mail "User Registration Confirmation" --markdown
```

### Generated File Structure

When you generate a mail class, the following files are created:

```txt
src/app/mail/
├── mod.rs                    # Updated with new module
├── order_shipped_mail.rs     # Generated mail class
└── ...

resources/views/mail/         # Markdown templates (if --markdown)
├── order_shipped_mail.md     # Generated template
└── ...
```

## Advanced Features

### Custom Headers

Add custom email headers:

```rust
#[async_trait]
impl Mailable for CustomMail {
    fn headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("X-Priority".to_string(), "1".to_string());
        headers.insert("X-Mailer".to_string(), "RustAxum".to_string());
        headers
    }
}
```

### Reply-To Addresses

Set custom reply-to addresses:

```rust
#[async_trait]
impl Mailable for SupportMail {
    fn reply_to(&self) -> Option<String> {
        Some("support@yourapp.com".to_string())
    }
}
```

### Custom From Address

Override the default from address per mail:

```rust
#[async_trait]
impl Mailable for NoReplyMail {
    fn from(&self) -> Option<String> {
        Some("noreply@yourapp.com".to_string())
    }
}
```

### CC and BCC Recipients

Add CC and BCC recipients:

```rust
MailMessage::new()
    .to("primary@example.com".to_string())
    .cc("manager@example.com".to_string())
    .bcc("admin@example.com".to_string())
```

### Multiple Recipients

Send to multiple recipients:

```rust
#[async_trait]
impl Mailable for NewsletterMail {
    fn to(&self) -> Vec<String> {
        vec![
            "user1@example.com".to_string(),
            "user2@example.com".to_string(),
            "user3@example.com".to_string(),
        ]
    }
}
```

## Troubleshooting

### Common Issues

#### SMTP Authentication Failures

**Problem:** SMTP authentication fails with "Authentication failed" error.

**Solutions:**

- Verify username and password are correct
- Check if the email provider requires app-specific passwords
- Ensure TLS/SSL settings match your provider's requirements

```env
# For Gmail
MAIL_HOST=smtp.gmail.com
MAIL_PORT=587
MAIL_ENCRYPTION=tls
MAIL_USERNAME=your-email@gmail.com
MAIL_PASSWORD=your-app-password  # Not your regular password!
```

#### Connection Timeouts

**Problem:** Email sending times out.

**Solutions:**

- Check firewall settings
- Verify SMTP server hostname and port
- Increase timeout value
- Test connectivity manually

```env
MAIL_TIMEOUT_SECONDS=60  # Increase timeout
```

#### Markdown Not Compiling

**Problem:** Markdown content not converting to HTML.

**Solution:** Ensure the `pulldown-cmark` dependency is available and call `compile_markdown()`:

```rust
async fn build(&self) -> Result<MailMessage> {
    let mut content = MailContent::Markdown {
        markdown: markdown_string,
        compiled_html: None,
    };
    content.compile_markdown().await?;
    // ... rest of build
}
```

#### Attachments Not Working

**Problem:** File attachments not being included.

**Solutions:**

- Verify file paths are absolute and accessible
- Check file permissions
- Ensure MIME type is correctly specified

```rust
// Use absolute paths
let attachment = Attachment::from_path(
    "document.pdf".to_string(),
    std::fs::canonicalize("./uploads/document.pdf")?.to_string_lossy().to_string(),
    Some("application/pdf".to_string())
);
```

### Debugging

Enable detailed logging to debug mail issues:

```env
RUST_LOG=debug
```

Or in your application:

```rust
tracing::debug!("Sending email to: {}", email_address);
```

### Testing Email Delivery

Use a testing service like Mailtrap for development:

```env
MAIL_MAILER=smtp
MAIL_HOST=smtp.mailtrap.io
MAIL_PORT=2525
MAIL_USERNAME=your_mailtrap_username
MAIL_PASSWORD=your_mailtrap_password
MAIL_ENCRYPTION=tls
```

## Best Practices

### 1. Use Queues for Heavy Email Operations

Always queue emails that don't need to be sent immediately:

```rust
fn should_queue(&self) -> bool {
    true  // Queue for background processing
}
```

### 2. Handle Failures Gracefully

```rust
match mail(&welcome_email).await {
    Ok(_) => tracing::info!("Welcome email sent successfully"),
    Err(e) => {
        tracing::error!("Failed to send welcome email: {}", e);
        // Consider retry logic or fallback notification
    }
}
```

### 3. Use Markdown for Maintainable Templates

Markdown is easier to maintain than HTML:

```rust
let content = MailContent::Markdown {
    markdown: markdown_template,
    compiled_html: None,
};
```

### 4. Validate Email Addresses

```rust
fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
    // Use a proper email validation library in production
}
```

### 5. Set Appropriate Timeouts

Configure reasonable timeouts for your use case:

```env
MAIL_TIMEOUT_SECONDS=30  # 30 seconds is usually sufficient
```

### 6. Use Environment-Specific Configuration

Different settings for development, staging, and production:

```env
# Development
MAIL_MAILER=log

# Staging
MAIL_MAILER=smtp
MAIL_HOST=smtp.mailtrap.io

# Production
MAIL_MAILER=smtp
MAIL_HOST=smtp.yourmailprovider.com
```

### 7. Monitor Email Delivery

Implement logging and monitoring for email delivery:

```rust
tracing::info!(
    email_type = "welcome",
    recipient = %self.to_email,
    "Sending welcome email"
);
```

### 8. Keep Templates Simple

Design email templates that work across different email clients:

- Use inline CSS
- Test on multiple email clients
- Provide text alternatives
- Keep layouts simple

### 9. Secure Sensitive Information

Never log or expose sensitive email content:

```rust
tracing::info!(
    "Sending password reset email to user: {}",
    self.to_email.chars().take(3).collect::<String>() + "***"
);
```

### 10. Test Thoroughly

Test your email system with:

- Different email providers
- Various email clients
- Mobile devices
- Spam filters

---

## Additional Resources

- [Lettre Documentation](https://lettre.rs/) - Rust email library
- [Email Client Testing](https://www.emailonacid.com/) - Cross-client testing
- [Mailtrap](https://mailtrap.io/) - Email testing service
- [Mailgun](https://www.mailgun.com/) - Production email service
- [SendGrid](https://sendgrid.com/) - Production email service

## File Locations

- **Mail Classes**: `src/app/mail/`
- **Mail Configuration**: `src/config/mail.rs`
- **Mail Drivers**: `src/app/mail/drivers/`
- **CLI Generators**: `src/cli/generators/mail.rs`
- **Templates**: `resources/views/mail/` (for Markdown templates)
