use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct CSRFConfig {
    pub enabled: bool,
    pub token_name: String,
    pub header_name: String,
    pub lifetime: u64, // in seconds
    pub except: Vec<String>, // Routes to exempt from CSRF protection
}

impl CSRFConfig {
    pub fn from_env() -> Result<Self> {
        let enabled = env::var("CSRF_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        let token_name = env::var("CSRF_TOKEN_NAME")
            .unwrap_or_else(|_| "_token".to_string());

        let header_name = env::var("CSRF_HEADER_NAME")
            .unwrap_or_else(|_| "X-CSRF-TOKEN".to_string());

        let lifetime = env::var("CSRF_LIFETIME")
            .unwrap_or_else(|_| "3600".to_string()) // 1 hour default
            .parse::<u64>()
            .unwrap_or(3600);

        // Parse exempt routes from environment variable
        let except = env::var("CSRF_EXCEPT")
            .unwrap_or_else(|_| "".to_string())
            .split(',')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .collect();

        Ok(CSRFConfig {
            enabled,
            token_name,
            header_name,
            lifetime,
            except,
        })
    }

    pub fn default_except_routes() -> Vec<String> {
        vec![
            "/api/oauth/*".to_string(),
            "/api/webhooks/*".to_string(),
            "/health".to_string(),
            "/metrics".to_string(),
        ]
    }

    pub fn with_default_exemptions(mut self) -> Self {
        let mut default_except = Self::default_except_routes();
        self.except.append(&mut default_except);
        self
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn token_name(&self) -> &str {
        &self.token_name
    }

    pub fn header_name(&self) -> &str {
        &self.header_name
    }

    pub fn lifetime(&self) -> u64 {
        self.lifetime
    }

    pub fn except_routes(&self) -> &Vec<String> {
        &self.except
    }
}

impl Default for CSRFConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            token_name: "_token".to_string(),
            header_name: "X-CSRF-TOKEN".to_string(),
            lifetime: 3600,
            except: Self::default_except_routes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_csrf_config_from_env() {
        env::set_var("CSRF_ENABLED", "false");
        env::set_var("CSRF_TOKEN_NAME", "custom_token");
        env::set_var("CSRF_HEADER_NAME", "X-Custom-CSRF");
        env::set_var("CSRF_LIFETIME", "7200");
        env::set_var("CSRF_EXCEPT", "/api/public,/webhooks/stripe");

        let config = CSRFConfig::from_env().unwrap();

        assert!(!config.enabled);
        assert_eq!(config.token_name, "custom_token");
        assert_eq!(config.header_name, "X-Custom-CSRF");
        assert_eq!(config.lifetime, 7200);
        assert_eq!(config.except, vec!["/api/public", "/webhooks/stripe"]);

        // Clean up
        env::remove_var("CSRF_ENABLED");
        env::remove_var("CSRF_TOKEN_NAME");
        env::remove_var("CSRF_HEADER_NAME");
        env::remove_var("CSRF_LIFETIME");
        env::remove_var("CSRF_EXCEPT");
    }

    #[test]
    fn test_csrf_config_defaults() {
        let config = CSRFConfig::default();

        assert!(config.enabled);
        assert_eq!(config.token_name, "_token");
        assert_eq!(config.header_name, "X-CSRF-TOKEN");
        assert_eq!(config.lifetime, 3600);
        assert!(config.except.contains(&"/api/oauth/*".to_string()));
    }
}