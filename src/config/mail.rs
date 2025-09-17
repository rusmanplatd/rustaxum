use anyhow::Result;
use std::env;

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

impl MailConfig {
    pub fn from_env() -> Result<Self> {
        Ok(MailConfig {
            mailer: env::var("MAIL_MAILER").unwrap_or_else(|_| "smtp".to_string()),
            host: env::var("MAIL_HOST")
                .or_else(|_| env::var("SMTP_HOST"))
                .unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("MAIL_PORT")
                .or_else(|_| env::var("SMTP_PORT"))
                .unwrap_or_else(|_| "1025".to_string())
                .parse()
                .unwrap_or(1025),
            username: env::var("MAIL_USERNAME")
                .or_else(|_| env::var("SMTP_USERNAME"))
                .unwrap_or_else(|_| "".to_string()),
            password: env::var("MAIL_PASSWORD")
                .or_else(|_| env::var("SMTP_PASSWORD"))
                .unwrap_or_else(|_| "".to_string()),
            encryption: env::var("MAIL_ENCRYPTION")
                .unwrap_or_else(|_| if env::var("SMTP_USE_TLS").unwrap_or_else(|_| "false".to_string()).parse().unwrap_or(false) {
                    "tls".to_string()
                } else {
                    "none".to_string()
                }),
            from_address: env::var("MAIL_FROM_ADDRESS")
                .or_else(|_| env::var("SMTP_FROM_EMAIL"))
                .unwrap_or_else(|_| "noreply@rustaxum.com".to_string()),
            from_name: env::var("MAIL_FROM_NAME")
                .or_else(|_| env::var("SMTP_FROM_NAME"))
                .unwrap_or_else(|_| "RustAxum".to_string()),
            timeout_seconds: env::var("MAIL_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
        })
    }

    pub fn use_tls(&self) -> bool {
        self.encryption == "tls" || self.encryption == "ssl"
    }

    pub fn use_ssl(&self) -> bool {
        self.encryption == "ssl"
    }
}