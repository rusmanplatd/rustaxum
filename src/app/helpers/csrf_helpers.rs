use anyhow::Result;

use crate::app::services::csrf_service::CSRFService;
use crate::app::services::session::SessionStore;

/// CSRF helper functions for views and templates
pub struct CSRFHelpers;

impl CSRFHelpers {
    /// Generate CSRF token for the current session
    pub async fn csrf_token(session_store: &SessionStore) -> Result<String> {
        let csrf_service = CSRFService::new();
        csrf_service.token(session_store).await
    }

    /// Generate CSRF hidden input field for forms
    pub async fn csrf_field(session_store: &SessionStore) -> Result<String> {
        let token = Self::csrf_token(session_store).await?;
        let csrf_service = CSRFService::new();

        Ok(format!(
            r#"<input type="hidden" name="{}" value="{}">"#,
            csrf_service.token_name(),
            token
        ))
    }

    /// Generate CSRF meta tag for HTML head
    pub async fn csrf_meta(session_store: &SessionStore) -> Result<String> {
        let token = Self::csrf_token(session_store).await?;

        Ok(format!(
            r#"<meta name="csrf-token" content="{}">"#,
            token
        ))
    }

    /// Get CSRF token name
    pub fn csrf_token_name() -> String {
        let csrf_service = CSRFService::new();
        csrf_service.token_name().to_string()
    }

    /// Get CSRF header name
    pub fn csrf_header_name() -> String {
        let csrf_service = CSRFService::new();
        csrf_service.header_name().to_string()
    }

    /// Generate JavaScript code for CSRF token
    pub async fn csrf_script(session_store: &SessionStore) -> Result<String> {
        let token = Self::csrf_token(session_store).await?;
        let csrf_service = CSRFService::new();

        Ok(format!(
            r#"
<script>
window.Laravel = {{
    csrfToken: '{token}',
    csrfTokenName: '{token_name}',
    csrfHeaderName: '{header_name}'
}};

// Set up axios defaults if axios is available
if (typeof axios !== 'undefined') {{
    axios.defaults.headers.common['{header_name}'] = '{token}';
}}

// Set up jQuery AJAX defaults if jQuery is available
if (typeof $ !== 'undefined') {{
    $.ajaxSetup({{
        headers: {{
            '{header_name}': '{token}'
        }}
    }});
}}

// Set up fetch defaults
window.csrfFetch = function(url, options = {{}}) {{
    options.headers = options.headers || {{}};
    options.headers['{header_name}'] = '{token}';
    return fetch(url, options);
}};
</script>
"#,
            token = token,
            token_name = csrf_service.token_name(),
            header_name = csrf_service.header_name()
        ))
    }

    /// Generate CSRF form with method spoofing (Laravel-like)
    pub async fn csrf_method_field(session_store: &SessionStore, method: &str) -> Result<String> {
        let csrf_field = Self::csrf_field(session_store).await?;

        let method_field = if !matches!(method.to_uppercase().as_str(), "GET" | "POST") {
            format!(r#"<input type="hidden" name="_method" value="{}">"#, method.to_uppercase())
        } else {
            String::new()
        };

        Ok(format!("{}\n{}", csrf_field, method_field))
    }
}

/// Handlebars helper for CSRF token (optional feature)
pub mod handlebars_helpers {
    use handlebars::{
        Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext, RenderError,
    };
    use serde_json::Value;

    /// Register CSRF helpers with Handlebars
    pub fn register_csrf_helpers(handlebars: &mut Handlebars) {
        handlebars.register_helper("csrf_field", Box::new(csrf_field_helper));
        handlebars.register_helper("csrf_token", Box::new(csrf_token_helper));
        handlebars.register_helper("csrf_meta", Box::new(csrf_meta_helper));
    }

    fn csrf_field_helper(
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        // TODO extract the token from context
        let token = "placeholder_token"; // This would come from the template context
        let csrf_service = super::CSRFService::new();

        let html = format!(
            r#"<input type="hidden" name="{}" value="{}">"#,
            csrf_service.token_name(),
            token
        );

        out.write(&html)?;
        Ok(())
    }

    fn csrf_token_helper(
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        // Placeholder - would extract token from context
        let token = "placeholder_token";
        out.write(token)?;
        Ok(())
    }

    fn csrf_meta_helper(
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _: &mut RenderContext,
        out: &mut dyn Output,
    ) -> HelperResult {
        let token = "placeholder_token";
        let html = format!(r#"<meta name="csrf-token" content="{}">"#, token);
        out.write(&html)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csrf_token_name() {
        let token_name = CSRFHelpers::csrf_token_name();
        assert_eq!(token_name, "_token");
    }

    #[test]
    fn test_csrf_header_name() {
        let header_name = CSRFHelpers::csrf_header_name();
        assert_eq!(header_name, "X-CSRF-TOKEN");
    }
}