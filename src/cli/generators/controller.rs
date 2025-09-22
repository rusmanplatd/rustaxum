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
    let service_name = format!("{}Service", model_name);
    let model_snake = to_snake_case(&model_name);
    let service_snake = to_snake_case(&service_name);

    format!(r#"use axum::{{
    extract::{{Path, Query, State}},
    http::StatusCode,
    response::{{IntoResponse, Json}},
}};
use anyhow::Result;
use crate::database::DbPool;
use crate::app::services::{}::{{{}, Create{}Request, Update{}Request}};
use crate::app::query_builder::pagination::PaginationParams;
use crate::app::resources::{}::{}Resource;
use serde::{{Deserialize, Serialize}};
use serde_json::json;

/// List all {} records with pagination
pub async fn index(
    State(pool): State<DbPool>,
    Query(pagination): Query<PaginationParams>
) -> impl IntoResponse {{
    match {}::list(&pool, pagination).await {{
        Ok(paginator) => {{
            let resources: Vec<{}Resource> = paginator.data
                .into_iter()
                .map({}Resource::from)
                .collect();

            (StatusCode::OK, Json(json!({{
                "data": resources,
                "pagination": {{
                    "current_page": paginator.current_page,
                    "per_page": paginator.per_page,
                    "total": paginator.total,
                    "last_page": paginator.last_page(),
                    "from": paginator.from(),
                    "to": paginator.to()
                }},
                "message": "List of {} retrieved successfully"
            }})))
        }}
        Err(e) => {{
            tracing::error!("Failed to list {}: {{}}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({{
                "error": "Failed to retrieve {}"
            }})))
        }}
    }}
}}

/// Create a new {} record
pub async fn store(
    State(pool): State<DbPool>,
    Json(payload): Json<Create{}Request>
) -> impl IntoResponse {{
    match {}::create(&pool, payload).await {{
        Ok(record) => {{
            let resource = {}Resource::from(record);
            (StatusCode::CREATED, Json(json!({{
                "data": resource,
                "message": "{} created successfully"
            }})))
        }}
        Err(e) => {{
            tracing::error!("Failed to create {}: {{}}", e);
            (StatusCode::BAD_REQUEST, Json(json!({{
                "error": "Failed to create {}"
            }})))
        }}
    }}
}}

/// Show a specific {} record
pub async fn show(
    State(pool): State<DbPool>,
    Path(id): Path<String>
) -> impl IntoResponse {{
    match {}::find_by_id(&pool, id.clone()).await {{
        Ok(Some(record)) => {{
            let resource = {}Resource::from(record);
            (StatusCode::OK, Json(json!({{
                "data": resource,
                "message": "{} retrieved successfully"
            }})))
        }}
        Ok(None) => {{
            (StatusCode::NOT_FOUND, Json(json!({{
                "error": "{} not found"
            }})))
        }}
        Err(e) => {{
            tracing::error!("Failed to find {} {{}}: {{}}", id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({{
                "error": "Failed to retrieve {}"
            }})))
        }}
    }}
}}

/// Update a specific {} record
pub async fn update(
    State(pool): State<DbPool>,
    Path(id): Path<String>,
    Json(payload): Json<Update{}Request>
) -> impl IntoResponse {{
    match {}::update(&pool, id.clone(), payload).await {{
        Ok(record) => {{
            let resource = {}Resource::from(record);
            (StatusCode::OK, Json(json!({{
                "data": resource,
                "message": "{} updated successfully"
            }})))
        }}
        Err(e) => {{
            tracing::error!("Failed to update {} {{}}: {{}}", id, e);
            let status = if e.to_string().contains("not found") {{
                StatusCode::NOT_FOUND
            }} else {{
                StatusCode::BAD_REQUEST
            }};
            (status, Json(json!({{
                "error": "Failed to update {}"
            }})))
        }}
    }}
}}

/// Delete a specific {} record
pub async fn destroy(
    State(pool): State<DbPool>,
    Path(id): Path<String>
) -> impl IntoResponse {{
    match {}::delete(&pool, id.clone()).await {{
        Ok(()) => {{
            (StatusCode::OK, Json(json!({{
                "message": "{} deleted successfully"
            }})))
        }}
        Err(e) => {{
            tracing::error!("Failed to delete {} {{}}: {{}}", id, e);
            let status = if e.to_string().contains("not found") {{
                StatusCode::NOT_FOUND
            }} else {{
                StatusCode::INTERNAL_SERVER_ERROR
            }};
            (status, Json(json!({{
                "error": "Failed to delete {}"
            }})))
        }}
    }}
}}
"#,
        service_snake, service_name, model_name, model_name,  // imports
        model_snake, model_name,  // resource import
        model_name, service_name, model_name, model_name, model_name, model_name, model_name,  // index function
        model_name, service_name, model_name, model_name, model_name, model_name,  // store function
        model_name, service_name, model_name, model_name, model_name, model_name, model_name, model_name,  // show function
        model_name, service_name, model_name, model_name, model_name, model_name, model_name,  // update function
        model_name, service_name, model_name, model_name, model_name)
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