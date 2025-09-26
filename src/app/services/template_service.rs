use anyhow::{Result, Context};
use handlebars::Handlebars;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::Config;
use crate::app::traits::ServiceActivityLogger;

pub struct TemplateService {
    handlebars: Arc<RwLock<Handlebars<'static>>>,
    templates_path: PathBuf,
}

impl ServiceActivityLogger for TemplateService {}

impl TemplateService {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let templates_path = PathBuf::from(&config.app.templates_path);

        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(false);

        // Register built-in helpers
        Self::register_helpers(&mut handlebars)?;

        let service = Self {
            handlebars: Arc::new(RwLock::new(handlebars)),
            templates_path,
        };

        // Load all templates
        service.load_templates().context("Failed to load templates")?;

        Ok(service)
    }

    pub async fn render<T: Serialize>(&self, template_name: &str, data: &T) -> Result<String> {
        let handlebars = self.handlebars.read().await;

        handlebars
            .render(template_name, data)
            .with_context(|| format!("Failed to render template '{}'", template_name))
    }

    pub async fn render_string<T: Serialize>(&self, template: &str, data: &T) -> Result<String> {
        let handlebars = self.handlebars.read().await;

        handlebars
            .render_template(template, data)
            .context("Failed to render template string")
    }

    pub async fn register_template(&self, name: &str, template: &str) -> Result<()> {
        let mut handlebars = self.handlebars.write().await;

        handlebars
            .register_template_string(name, template)
            .with_context(|| format!("Failed to register template '{}'", name))
    }

    pub async fn register_template_file(&self, name: &str, file_path: &Path) -> Result<()> {
        let mut handlebars = self.handlebars.write().await;

        handlebars
            .register_template_file(name, file_path)
            .with_context(|| format!("Failed to register template file '{}'", file_path.display()))
    }

    pub async fn unregister_template(&self, name: &str) {
        let mut handlebars = self.handlebars.write().await;
        handlebars.unregister_template(name);
    }

    pub async fn clear_templates(&self) {
        let mut handlebars = self.handlebars.write().await;
        handlebars.clear_templates();
    }

    pub async fn reload_templates(&self) -> Result<()> {
        self.clear_templates().await;
        self.load_templates().context("Failed to reload templates")
    }

    pub async fn has_template(&self, name: &str) -> bool {
        let handlebars = self.handlebars.read().await;
        handlebars.has_template(name)
    }

    pub async fn get_templates(&self) -> Vec<String> {
        let handlebars = self.handlebars.read().await;
        handlebars.get_templates().keys().cloned().collect()
    }

    fn load_templates(&self) -> Result<()> {
        if !self.templates_path.exists() {
            std::fs::create_dir_all(&self.templates_path)
                .with_context(|| format!("Failed to create templates directory: {}", self.templates_path.display()))?;
        }

        self.load_templates_recursive(&self.templates_path, &self.templates_path)
    }

    fn load_templates_recursive(&self, base_path: &Path, current_path: &Path) -> Result<()> {
        for entry in std::fs::read_dir(current_path)
            .with_context(|| format!("Failed to read directory: {}", current_path.display()))?
        {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.is_dir() {
                self.load_templates_recursive(base_path, &path)?;
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext == "hbs" {
                    let relative_path = path.strip_prefix(base_path)
                        .context("Failed to get relative path")?;

                    let template_name = relative_path
                        .with_extension("")
                        .to_string_lossy()
                        .replace(std::path::MAIN_SEPARATOR, "/");

                    let mut handlebars = futures::executor::block_on(self.handlebars.write());

                    // Check if this is a partial (in partials/ directory)
                    if template_name.starts_with("partials/") {
                        let partial_name = template_name.strip_prefix("partials/").unwrap();
                        let template_content = std::fs::read_to_string(&path)
                            .with_context(|| format!("Failed to read partial file: {}", path.display()))?;
                        handlebars
                            .register_partial(partial_name, template_content)
                            .with_context(|| format!("Failed to register partial: {}", path.display()))?;
                    } else {
                        handlebars
                            .register_template_file(&template_name, &path)
                            .with_context(|| format!("Failed to register template: {}", path.display()))?;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn register_partial(&self, name: &str, template: &str) -> Result<()> {
        let mut handlebars = self.handlebars.write().await;
        handlebars
            .register_partial(name, template)
            .with_context(|| format!("Failed to register partial '{}'", name))
    }

    pub async fn register_partial_file(&self, name: &str, file_path: &Path) -> Result<()> {
        let template_content = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read partial file '{}'", file_path.display()))?;

        let mut handlebars = self.handlebars.write().await;
        handlebars
            .register_partial(name, template_content)
            .with_context(|| format!("Failed to register partial file '{}'", file_path.display()))
    }

    pub async fn unregister_partial(&self, name: &str) {
        let mut handlebars = self.handlebars.write().await;
        handlebars.unregister_template(name);
    }

    pub async fn has_partial(&self, name: &str) -> bool {
        let handlebars = self.handlebars.read().await;
        handlebars.has_template(name)
    }

    fn register_helpers(handlebars: &mut Handlebars<'static>) -> Result<()> {
        use serde_json::Value;
        use handlebars::{Helper, RenderContext, Output, HelperResult};
        use chrono::{DateTime, Utc};

        // Format currency helper
        handlebars.register_helper(
            "formatCurrency",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let amount = h.param(0)
                    .and_then(|v| v.value().as_f64())
                    .unwrap_or(0.0);

                let currency = h.param(1)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("USD");

                let formatted = match currency {
                    "USD" => format!("${:.2}", amount),
                    "EUR" => format!("€{:.2}", amount),
                    "GBP" => format!("£{:.2}", amount),
                    "JPY" => format!("¥{:.0}", amount),
                    "CAD" => format!("C${:.2}", amount),
                    "AUD" => format!("A${:.2}", amount),
                    _ => format!("{:.2} {}", amount, currency),
                };

                out.write(&formatted)?;
                Ok(())
            }),
        );

        // Truncate helper with word boundary support
        handlebars.register_helper(
            "truncate",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let text = h.param(0)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("");

                let limit = h.param(1)
                    .and_then(|v| v.value().as_u64())
                    .unwrap_or(100) as usize;

                let word_boundary = h.hash_get("words")
                    .and_then(|v| v.value().as_bool())
                    .unwrap_or(false);

                let suffix = h.hash_get("suffix")
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("...");

                let truncated = if text.len() > limit {
                    if word_boundary {
                        let mut end = limit;
                        while end > 0 && !text.chars().nth(end).unwrap_or(' ').is_whitespace() {
                            end -= 1;
                        }
                        if end == 0 { end = limit; }
                        format!("{}{}", &text[..end], suffix)
                    } else {
                        format!("{}{}", &text[..limit], suffix)
                    }
                } else {
                    text.to_string()
                };

                out.write(&truncated)?;
                Ok(())
            }),
        );

        // JSON helper with pretty print option
        handlebars.register_helper(
            "json",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let value = h.param(0).map(|v| v.value()).unwrap_or(&Value::Null);
                let pretty = h.hash_get("pretty")
                    .and_then(|v| v.value().as_bool())
                    .unwrap_or(false);

                let json_str = if pretty {
                    serde_json::to_string_pretty(value).unwrap_or_else(|_| "null".to_string())
                } else {
                    serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
                };
                out.write(&json_str)?;
                Ok(())
            }),
        );

        // Math helpers
        handlebars.register_helper(
            "add",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let a = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
                let b = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
                out.write(&format!("{}", a + b))?;
                Ok(())
            }),
        );

        handlebars.register_helper(
            "subtract",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let a = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
                let b = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
                out.write(&format!("{}", a - b))?;
                Ok(())
            }),
        );

        handlebars.register_helper(
            "multiply",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let a = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
                let b = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
                out.write(&format!("{}", a * b))?;
                Ok(())
            }),
        );

        handlebars.register_helper(
            "divide",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let a = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
                let b = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(1.0);
                if b != 0.0 {
                    out.write(&format!("{}", a / b))?;
                } else {
                    out.write("0")?;
                }
                Ok(())
            }),
        );

        handlebars.register_helper(
            "modulo",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let a = h.param(0).and_then(|v| v.value().as_i64()).unwrap_or(0);
                let b = h.param(1).and_then(|v| v.value().as_i64()).unwrap_or(1);
                if b != 0 {
                    out.write(&format!("{}", a % b))?;
                } else {
                    out.write("0")?;
                }
                Ok(())
            }),
        );

        // String case helpers
        handlebars.register_helper(
            "uppercase",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let text = h.param(0)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("");
                out.write(&text.to_uppercase())?;
                Ok(())
            }),
        );

        handlebars.register_helper(
            "lowercase",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let text = h.param(0)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("");
                out.write(&text.to_lowercase())?;
                Ok(())
            }),
        );

        handlebars.register_helper(
            "capitalize",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let text = h.param(0)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("");

                let mut chars = text.chars();
                let capitalized = match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                };
                out.write(&capitalized)?;
                Ok(())
            }),
        );

        handlebars.register_helper(
            "title",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let text = h.param(0)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("");

                let title_case: String = text
                    .split_whitespace()
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(" ");

                out.write(&title_case)?;
                Ok(())
            }),
        );

        // Date and time helpers
        handlebars.register_helper(
            "formatDate",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let date_str = h.param(0)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("");

                let format = h.param(1)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("%Y-%m-%d");

                if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
                    out.write(&dt.format(format).to_string())?;
                } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
                    out.write(&dt.format(format).to_string())?;
                } else {
                    out.write(date_str)?;
                }
                Ok(())
            }),
        );

        handlebars.register_helper(
            "timeAgo",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let date_str = h.param(0)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("");

                if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
                    let now = Utc::now();
                    let duration = now.signed_duration_since(dt);

                    let time_ago = if duration.num_days() > 0 {
                        format!("{} days ago", duration.num_days())
                    } else if duration.num_hours() > 0 {
                        format!("{} hours ago", duration.num_hours())
                    } else if duration.num_minutes() > 0 {
                        format!("{} minutes ago", duration.num_minutes())
                    } else {
                        "just now".to_string()
                    };

                    out.write(&time_ago)?;
                } else {
                    out.write(date_str)?;
                }
                Ok(())
            }),
        );

        // Array/Object helpers
        handlebars.register_helper(
            "length",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let value = h.param(0).map(|v| v.value()).unwrap_or(&Value::Null);
                let length = match value {
                    Value::Array(arr) => arr.len(),
                    Value::String(s) => s.len(),
                    Value::Object(obj) => obj.len(),
                    _ => 0,
                };
                out.write(&length.to_string())?;
                Ok(())
            }),
        );

        handlebars.register_helper(
            "isEmpty",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let value = h.param(0).map(|v| v.value()).unwrap_or(&Value::Null);
                let is_empty = match value {
                    Value::Null => true,
                    Value::Array(arr) => arr.is_empty(),
                    Value::String(s) => s.is_empty(),
                    Value::Object(obj) => obj.is_empty(),
                    _ => false,
                };
                out.write(&is_empty.to_string())?;
                Ok(())
            }),
        );

        // Comparison helpers
        handlebars.register_helper(
            "eq",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let a = h.param(0).map(|v| v.value()).unwrap_or(&Value::Null);
                let b = h.param(1).map(|v| v.value()).unwrap_or(&Value::Null);
                out.write(&(a == b).to_string())?;
                Ok(())
            }),
        );

        handlebars.register_helper(
            "gt",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let a = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
                let b = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
                out.write(&(a > b).to_string())?;
                Ok(())
            }),
        );

        handlebars.register_helper(
            "lt",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let a = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
                let b = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
                out.write(&(a < b).to_string())?;
                Ok(())
            }),
        );

        // URL helpers
        handlebars.register_helper(
            "urlEncode",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let text = h.param(0)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("");
                out.write(&urlencoding::encode(text))?;
                Ok(())
            }),
        );

        // HTML escape/unescape helpers
        handlebars.register_helper(
            "htmlEscape",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let text = h.param(0)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("");
                out.write(&html_escape::encode_text(text))?;
                Ok(())
            }),
        );

        // Default helper
        handlebars.register_helper(
            "default",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> HelperResult {
                let value = h.param(0).map(|v| v.value()).unwrap_or(&Value::Null);
                let default_value = h.param(1)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("");

                match value {
                    Value::Null => out.write(default_value)?,
                    Value::String(s) if s.is_empty() => out.write(default_value)?,
                    _ => out.write(&value.to_string())?,
                }
                Ok(())
            }),
        );

        Ok(())
    }
}

// Global instance for easy access
use std::sync::OnceLock;

static TEMPLATE_SERVICE: OnceLock<TemplateService> = OnceLock::new();

impl TemplateService {
    pub fn global() -> &'static TemplateService {
        TEMPLATE_SERVICE.get_or_init(|| {
            TemplateService::new().expect("Failed to initialize TemplateService")
        })
    }
}