use axum::response::IntoResponse;
use serde_json::json;
use crate::app::http::responses::template_response::TemplateResponse;

pub struct TemplateDemoController;

impl TemplateDemoController {
    pub async fn showcase() -> impl IntoResponse {
        let data = json!({
            "title": "Handlebars Template Showcase - RustAxum",
            "page_title": "Template Features Demo",
            "example_array": [1, 2, 3, 4, 5],
            "example_object": {
                "name": "John Doe",
                "age": 30,
                "active": true
            },
            "null_value": null,
            "country_options": [
                {"value": "us", "label": "United States"},
                {"value": "ca", "label": "Canada"},
                {"value": "uk", "label": "United Kingdom"},
                {"value": "de", "label": "Germany"}
            ],
            "selected_country": "us",
            "pagination_range": [1, 2, 3, "...", 8, 9, 10],
            "breadcrumbs": [
                {"title": "Templates", "url": "/templates"},
                {"title": "Showcase", "url": null}
            ],
            "show_breadcrumb": true
        });

        TemplateResponse::new("template-showcase", &data)
            .with_layout("layouts/app")
    }
}