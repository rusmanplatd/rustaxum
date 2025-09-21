use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;

pub async fn generate_controller(name: &str, resource: bool) -> Result<()> {
    let controller_name = format_controller_name(name);
    let file_name = to_snake_case(&controller_name);
    let file_path = format!("src/app/controllers/{}.rs", file_name);

    if Path::new(&file_path).exists() {
        return Err(anyhow!("Controller {} already exists", controller_name));
    }

    let content = if resource {
        generate_resource_controller(&controller_name)
    } else {
        generate_basic_controller(&controller_name)
    };

    fs::write(&file_path, content)?;
    println!("Controller created: {}", file_path);

    // Update the controllers mod.rs file
    update_controllers_mod(&file_name)?;

    Ok(())
}

fn format_controller_name(name: &str) -> String {
    if name.ends_with("Controller") {
        name.to_string()
    } else {
        format!("{}Controller", name)
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

fn generate_basic_controller(controller_name: &str) -> String {
    format!(r#"use axum::{{
    http::StatusCode,
    response::{{IntoResponse, Json}},
}};
use serde_json::json;

pub async fn index() -> impl IntoResponse {{
    (StatusCode::OK, Json(json!({{
        "message": "Hello from {}!"
    }})))
}}
"#, controller_name)
}

fn generate_resource_controller(controller_name: &str) -> String {
    let model_name = controller_name.replace("Controller", "");

    format!(r#"use axum::{{
    extract::{{Path, State}},
    http::StatusCode,
    response::{{IntoResponse, Json}},
}};
use crate::database::DbPool;
use serde::{{Deserialize, Serialize}};
use serde_json::json;

#[derive(Deserialize)]
pub struct Create{}Request {{
    // Add your fields here
}}

#[derive(Deserialize)]
pub struct Update{}Request {{
    // Add your fields here
}}

#[derive(Serialize)]
pub struct {}Response {{
    pub id: String,
    // Add your fields here
}}

pub async fn index(State(pool): State<DbPool>) -> impl IntoResponse {{
    // TODO: Implement index logic
    (StatusCode::OK, Json(json!({{
        "data": [],
        "message": "List of {}"
    }})))
}}

pub async fn store(
    State(pool): State<DbPool>,
    Json(payload): Json<Create{}Request>
) -> impl IntoResponse {{
    // TODO: Implement store logic
    (StatusCode::CREATED, Json(json!({{
        "message": "{} created successfully"
    }})))
}}

pub async fn show(
    State(pool): State<DbPool>,
    Path(id): Path<String>
) -> impl IntoResponse {{
    // TODO: Implement show logic
    (StatusCode::OK, Json(json!({{
        "data": {{}},
        "message": "{} retrieved successfully"
    }})))
}}

pub async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<Update{}Request>
) -> impl IntoResponse {{
    // TODO: Implement update logic
    (StatusCode::OK, Json(json!({{
        "message": "{} updated successfully"
    }})))
}}

pub async fn destroy(
    State(pool): State<DbPool>,
    Path(id): Path<String>
) -> impl IntoResponse {{
    // TODO: Implement destroy logic
    (StatusCode::OK, Json(json!({{
        "message": "{} deleted successfully"
    }})))
}}
"#, model_name, model_name, model_name, model_name, model_name, model_name, model_name, model_name, model_name, model_name)
}

fn update_controllers_mod(file_name: &str) -> Result<()> {
    let mod_path = "src/app/controllers/mod.rs";
    let module_declaration = format!("pub mod {};", file_name);

    if let Ok(current_content) = fs::read_to_string(mod_path) {
        if !current_content.contains(&module_declaration) {
            let new_content = format!("{}\n{}", current_content.trim(), module_declaration);
            fs::write(mod_path, new_content)?;
            println!("Updated controllers/mod.rs");
        }
    }

    Ok(())
}