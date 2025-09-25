use axum::response::{Html, IntoResponse, Response};
use axum::http::StatusCode;
use serde::Serialize;
use serde_json::json;
use chrono::{Utc, Datelike};
use anyhow::Result;
use crate::app::services::template_service::TemplateService;
use crate::config::Config;

pub struct TemplateResponse {
    template: String,
    data: serde_json::Value,
    layout: Option<String>,
    status: StatusCode,
}

impl TemplateResponse {
    pub fn new<T: Serialize>(template: &str, data: &T) -> Self {
        Self {
            template: template.to_string(),
            data: serde_json::to_value(data).unwrap_or_else(|_| json!({})),
            layout: Some("layouts/main".to_string()),
            status: StatusCode::OK,
        }
    }

    pub fn with_layout(mut self, layout: &str) -> Self {
        self.layout = Some(layout.to_string());
        self
    }

    pub fn without_layout(mut self) -> Self {
        self.layout = None;
        self
    }

    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    async fn render(&self) -> Result<String> {
        let template_service = TemplateService::global();
        let config = Config::load()?;

        // Add global template variables
        let mut template_data = self.data.clone();
        if let Some(obj) = template_data.as_object_mut() {
            obj.insert("app_name".to_string(), json!(config.app.name));
            obj.insert("app_url".to_string(), json!(config.app.url));
            obj.insert("year".to_string(), json!(Utc::now().year()));
        }

        match &self.layout {
            Some(layout) => {
                // Render the page content first
                let content = template_service.render(&self.template, &template_data).await?;

                // Add content to template data for layout
                if let Some(obj) = template_data.as_object_mut() {
                    obj.insert("content".to_string(), json!(content));
                }

                // Render with layout
                template_service.render(layout, &template_data).await
            }
            None => {
                // Render without layout
                template_service.render(&self.template, &template_data).await
            }
        }
    }
}

impl IntoResponse for TemplateResponse {
    fn into_response(self) -> Response {
        let runtime = tokio::runtime::Handle::current();
        match runtime.block_on(self.render()) {
            Ok(html) => (self.status, Html(html)).into_response(),
            Err(e) => {
                tracing::error!("Template rendering error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Html(format!(
                        "<h1>Template Error</h1><p>Failed to render template: {}</p>",
                        e
                    )),
                )
                    .into_response()
            }
        }
    }
}

// Helper functions for common responses
pub async fn view<T: Serialize>(template: &str, data: &T) -> TemplateResponse {
    TemplateResponse::new(template, data)
}

pub async fn view_with_layout<T: Serialize>(
    template: &str,
    data: &T,
    layout: &str,
) -> TemplateResponse {
    TemplateResponse::new(template, data).with_layout(layout)
}

pub async fn view_without_layout<T: Serialize>(template: &str, data: &T) -> TemplateResponse {
    TemplateResponse::new(template, data).without_layout()
}