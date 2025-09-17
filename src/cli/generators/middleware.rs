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
    format!(r#"use axum::{{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{{IntoResponse, Response}},
}};

pub async fn {}(request: Request, next: Next) -> Result<Response, impl IntoResponse> {{
    // TODO: Implement middleware logic here

    // Example: Add a header or perform validation
    // let headers = request.headers();

    // Continue with the request
    let response = next.run(request).await;

    Ok(response)
}}
"#, to_snake_case(&middleware_name.replace("Middleware", "")))
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