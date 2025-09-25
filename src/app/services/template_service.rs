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
            } else if path.extension().and_then(|s| s.to_str()) == Some("hbs") {
                let relative_path = path.strip_prefix(base_path)
                    .context("Failed to get relative path")?;

                let template_name = relative_path
                    .with_extension("")
                    .to_string_lossy()
                    .replace(std::path::MAIN_SEPARATOR, "/");

                let mut handlebars = futures::executor::block_on(self.handlebars.write());
                handlebars
                    .register_template_file(&template_name, &path)
                    .with_context(|| format!("Failed to register template: {}", path.display()))?;
            }
        }

        Ok(())
    }

    fn register_helpers(handlebars: &mut Handlebars<'static>) -> Result<()> {
        use serde_json::Value;
        use handlebars::{Helper, RenderContext, RenderError, Output};

        // Format currency helper
        handlebars.register_helper(
            "formatCurrency",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> Result<(), RenderError> {
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
                    _ => format!("{:.2} {}", amount, currency),
                };

                out.write(&formatted)?;
                Ok(())
            }),
        );

        // Truncate helper
        handlebars.register_helper(
            "truncate",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> Result<(), RenderError> {
                let text = h.param(0)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("");

                let limit = h.param(1)
                    .and_then(|v| v.value().as_u64())
                    .unwrap_or(100) as usize;

                let truncated = if text.len() > limit {
                    format!("{}...", &text[..limit])
                } else {
                    text.to_string()
                };

                out.write(&truncated)?;
                Ok(())
            }),
        );

        // JSON helper
        handlebars.register_helper(
            "json",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> Result<(), RenderError> {
                let value = h.param(0).map(|v| v.value()).unwrap_or(&Value::Null);
                let json_str = serde_json::to_string(value).unwrap_or_else(|_| "null".to_string());
                out.write(&json_str)?;
                Ok(())
            }),
        );

        // Math helpers
        handlebars.register_helper(
            "add",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> Result<(), RenderError> {
                let a = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
                let b = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
                out.write(&format!("{}", a + b))?;
                Ok(())
            }),
        );

        handlebars.register_helper(
            "subtract",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> Result<(), RenderError> {
                let a = h.param(0).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
                let b = h.param(1).and_then(|v| v.value().as_f64()).unwrap_or(0.0);
                out.write(&format!("{}", a - b))?;
                Ok(())
            }),
        );

        // String case helpers
        handlebars.register_helper(
            "uppercase",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> Result<(), RenderError> {
                let text = h.param(0)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("");
                out.write(&text.to_uppercase())?;
                Ok(())
            }),
        );

        handlebars.register_helper(
            "lowercase",
            Box::new(|h: &Helper, _: &Handlebars, _: &handlebars::Context, _: &mut RenderContext, out: &mut dyn Output| -> Result<(), RenderError> {
                let text = h.param(0)
                    .and_then(|v| v.value().as_str())
                    .unwrap_or("");
                out.write(&text.to_lowercase())?;
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