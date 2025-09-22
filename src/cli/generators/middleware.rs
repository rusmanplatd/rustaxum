use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;

pub async fn generate_middleware(name: &str) -> Result<()> {
    let middleware_name = format_middleware_name(name);
    let file_name = to_snake_case(&middleware_name);
    let file_path = format!("src/app/middleware/{}.rs", file_name);

    if Path::new(&file_path).exists() {
        return Err(anyhow!("Middleware {} already exists", middleware_name));
    }

    let content = generate_middleware_content(&middleware_name);

    fs::write(&file_path, content)?;
    println!("Middleware created: {}", file_path);

    // Update the middleware mod.rs file
    update_middleware_mod(&file_name)?;

    Ok(())
}

fn format_middleware_name(name: &str) -> String {
    if name.ends_with("Middleware") {
        name.to_string()
    } else {
        format!("{}Middleware", name)
    }
}

fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    let mut prev_char_was_uppercase = false;

    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_char_was_uppercase {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_char_was_uppercase = true;
        } else {
            result.push(c);
            prev_char_was_uppercase = false;
        }
    }

    result
}

fn generate_middleware_content(middleware_name: &str) -> String {
    let function_name = to_snake_case(&middleware_name.replace("Middleware", ""));

    format!(r#"use axum::{{
    extract::{{Request, State}},
    http::{{StatusCode, HeaderValue}},
    middleware::Next,
    response::{{IntoResponse, Response}},
}};
use anyhow::Result;
use tracing::{{info, warn, error}};

/// {} middleware for request processing
pub async fn {}(
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {{
    let start_time = std::time::Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();

    info!("Processing {} request to: {{}}", method, uri);

    // Pre-processing: Add your middleware logic here
    // Examples:
    // - Authentication/authorization checks
    // - Rate limiting
    // - Request validation
    // - Headers manipulation
    // - Logging and metrics

    // Process the request
    let mut response = next.run(request).await;

    // Post-processing: Add response modifications here
    // Examples:
    // - Add security headers
    // - Log response metrics
    // - Response transformation

    // Add processing time header
    let duration = start_time.elapsed();
    if let Ok(duration_header) = HeaderValue::from_str(&format!("{{}}.{{:03}}",
        duration.as_secs(),
        duration.subsec_millis())) {{
        response.headers_mut().insert("X-Processing-Time-Ms", duration_header);
    }}

    info!("Completed {} request to {{}} in {{:.2?}}", method, uri, duration);

    Ok(response)
}}

/// Error handler for {} middleware
pub fn handle_middleware_error(error: &str) -> impl IntoResponse {{
    error!("Middleware error: {{}}", error);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Middleware error: {{}}", error)
    )
}}
"#, middleware_name, function_name, middleware_name, middleware_name, middleware_name)
}

fn update_middleware_mod(file_name: &str) -> Result<()> {
    let mod_path = "src/app/middleware/mod.rs";
    let module_declaration = format!("pub mod {};", file_name);

    if let Ok(current_content) = fs::read_to_string(mod_path) {
        if !current_content.contains(&module_declaration) {
            let new_content = format!("{}\n{}", current_content.trim(), module_declaration);
            fs::write(mod_path, new_content)?;
            println!("Updated middleware/mod.rs");
        }
    }

    Ok(())
}