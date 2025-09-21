use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;

pub async fn generate_service(name: &str) -> Result<()> {
    let service_name = format_service_name(name);
    let file_name = to_snake_case(&service_name);
    let file_path = format!("src/app/services/{}.rs", file_name);

    if Path::new(&file_path).exists() {
        return Err(anyhow!("Service {} already exists", service_name));
    }

    let content = generate_service_content(&service_name);

    fs::write(&file_path, content)?;
    println!("Service created: {}", file_path);

    // Update the services mod.rs file
    update_services_mod(&file_name)?;

    Ok(())
}

fn format_service_name(name: &str) -> String {
    if name.ends_with("Service") {
        name.to_string()
    } else {
        format!("{}Service", name)
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

fn generate_service_content(service_name: &str) -> String {
    let model_name = service_name.replace("Service", "");
    let model_snake = to_snake_case(&model_name);

    format!(r#"use anyhow::Result;
use ulid::Ulid;
use crate::database::DbPool;

// TODO: Import your model
// use crate::app::models::{}::{{{}, Create{}, Update{}}};

pub struct {};

impl {} {{
    pub async fn create(pool: &DbPool, data: Create{}) -> Result<{}> {{
        // TODO: Implement create logic
        todo!("Implement create method")
    }}

    pub async fn find_by_id(pool: &DbPool, id: Ulid) -> Result<Option<{}>> {{
        // TODO: Implement find by id logic
        todo!("Implement find_by_id method")
    }}

    pub async fn update(pool: &DbPool, id: Ulid, data: Update{}) -> Result<{}> {{
        // TODO: Implement update logic
        todo!("Implement update method")
    }}

    pub async fn delete(pool: &DbPool, id: Ulid) -> Result<()> {{
        // TODO: Implement delete logic
        todo!("Implement delete method")
    }}

    pub async fn list(pool: &DbPool, limit: i64, offset: i64) -> Result<Vec<{}>> {{
        // TODO: Implement list logic
        todo!("Implement list method")
    }}
}}
"#, model_snake, model_name, model_name, model_name, service_name, service_name, model_name, model_name, model_name, model_name, model_name, model_name)
}

fn update_services_mod(file_name: &str) -> Result<()> {
    let mod_path = "src/app/services/mod.rs";
    let module_declaration = format!("pub mod {};", file_name);

    if let Ok(current_content) = fs::read_to_string(mod_path) {
        if !current_content.contains(&module_declaration) {
            let new_content = format!("{}\n{}", current_content.trim(), module_declaration);
            fs::write(mod_path, new_content)?;
            println!("Updated services/mod.rs");
        }
    }

    Ok(())
}