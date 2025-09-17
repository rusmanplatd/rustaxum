use anyhow::{Result, bail};

pub struct PasswordValidator;

impl PasswordValidator {
    pub fn validate(password: &str) -> Result<()> {
        if password.len() < 8 {
            bail!("Password must be at least 8 characters long");
        }

        if password.len() > 128 {
            bail!("Password must not exceed 128 characters");
        }

        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));

        if !has_lowercase {
            bail!("Password must contain at least one lowercase letter");
        }

        if !has_uppercase {
            bail!("Password must contain at least one uppercase letter");
        }

        if !has_digit {
            bail!("Password must contain at least one number");
        }

        if !has_special {
            bail!("Password must contain at least one special character");
        }

        Ok(())
    }

    pub fn validate_confirmation(password: &str, confirmation: &str) -> Result<()> {
        if password != confirmation {
            bail!("Password confirmation does not match");
        }
        Ok(())
    }
}