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

    let mut content = String::new();

    content.push_str("use anyhow::{Result, anyhow};\n");
    content.push_str("use sqlx::{FromRow, Row};\n");
    content.push_str("use ulid::Ulid;\n");
    content.push_str("use serde::{Serialize, Deserialize};\n");
    content.push_str("use crate::database::DbPool;\n");
    content.push_str("use crate::app::query_builder::pagination::{Paginator, PaginationParams};\n\n");

    content.push_str(&format!("// use crate::app::models::{}::{{{}}};\n\n", model_snake, model_name));

    content.push_str("#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]\n");
    content.push_str(&format!("pub struct {} {{\n", model_name));
    content.push_str("    pub id: String,\n");
    content.push_str("    pub created_at: chrono::DateTime<chrono::Utc>,\n");
    content.push_str("    pub updated_at: chrono::DateTime<chrono::Utc>,\n");
    content.push_str("}\n\n");

    content.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    content.push_str(&format!("pub struct Create{}Request {{\n", model_name));
    content.push_str("    // Add your fields here\n");
    content.push_str("}\n\n");

    content.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    content.push_str(&format!("pub struct Update{}Request {{\n", model_name));
    content.push_str("    // Add your fields here\n");
    content.push_str("}\n\n");

    content.push_str(&format!("pub struct {};\n\n", service_name));

    content.push_str(&format!("impl {} {{\n", service_name));

    // Create method
    content.push_str(&format!("    /// Create a new {} record\n", model_name));
    content.push_str(&format!("    pub async fn create(pool: &DbPool, data: Create{}Request) -> Result<{}> {{\n", model_name, model_name));
    content.push_str("        let id = Ulid::new().to_string();\n");
    content.push_str("        let now = chrono::Utc::now();\n\n");
    content.push_str(&format!("        let result = sqlx::query_as!(\n"));
    content.push_str(&format!("            {},\n", model_name));
    content.push_str("            r#\"\n");
    content.push_str(&format!("            INSERT INTO {} (id, created_at, updated_at)\n", model_snake));
    content.push_str("            VALUES ($1, $2, $3)\n");
    content.push_str("            RETURNING *\n");
    content.push_str("            \"#,\n");
    content.push_str("            id,\n");
    content.push_str("            now,\n");
    content.push_str("            now\n");
    content.push_str("        )\n");
    content.push_str("        .fetch_one(pool)\n");
    content.push_str("        .await\n");
    content.push_str(&format!("        .map_err(|e| anyhow!(\"Failed to create {}: {{}}\", e))?;\n\n", model_name));
    content.push_str("        Ok(result)\n");
    content.push_str("    }\n\n");

    // Find by ID method
    content.push_str(&format!("    /// Find {} by ID\n", model_name));
    content.push_str(&format!("    pub async fn find_by_id(pool: &DbPool, id: String) -> Result<Option<{}>> {{\n", model_name));
    content.push_str(&format!("        let result = sqlx::query_as!(\n"));
    content.push_str(&format!("            {},\n", model_name));
    content.push_str(&format!("            \"SELECT * FROM {} WHERE id = $1\",\n", model_snake));
    content.push_str("            id\n");
    content.push_str("        )\n");
    content.push_str("        .fetch_optional(pool)\n");
    content.push_str("        .await\n");
    content.push_str(&format!("        .map_err(|e| anyhow!(\"Failed to find {} by ID: {{}}\", e))?;\n\n", model_name));
    content.push_str("        Ok(result)\n");
    content.push_str("    }\n\n");

    // Update method
    content.push_str(&format!("    /// Update {} record\n", model_name));
    content.push_str(&format!("    pub async fn update(pool: &DbPool, id: String, data: Update{}Request) -> Result<{}> {{\n", model_name, model_name));
    content.push_str("        let now = chrono::Utc::now();\n\n");
    content.push_str(&format!("        let result = sqlx::query_as!(\n"));
    content.push_str(&format!("            {},\n", model_name));
    content.push_str("            r#\"\n");
    content.push_str(&format!("            UPDATE {}\n", model_snake));
    content.push_str("            SET updated_at = $2\n");
    content.push_str("            WHERE id = $1\n");
    content.push_str("            RETURNING *\n");
    content.push_str("            \"#,\n");
    content.push_str("            id,\n");
    content.push_str("            now\n");
    content.push_str("        )\n");
    content.push_str("        .fetch_one(pool)\n");
    content.push_str("        .await\n");
    content.push_str(&format!("        .map_err(|e| anyhow!(\"Failed to update {}: {{}}\", e))?;\n\n", model_name));
    content.push_str("        Ok(result)\n");
    content.push_str("    }\n\n");

    // Delete method
    content.push_str(&format!("    /// Delete {} record\n", model_name));
    content.push_str("    pub async fn delete(pool: &DbPool, id: String) -> Result<()> {\n");
    content.push_str("        let rows_affected = sqlx::query!(\n");
    content.push_str(&format!("            \"DELETE FROM {} WHERE id = $1\",\n", model_snake));
    content.push_str("            id\n");
    content.push_str("        )\n");
    content.push_str("        .execute(pool)\n");
    content.push_str("        .await\n");
    content.push_str(&format!("        .map_err(|e| anyhow!(\"Failed to delete {}: {{}}\", e))?\n", model_name));
    content.push_str("        .rows_affected();\n\n");
    content.push_str("        if rows_affected == 0 {\n");
    content.push_str(&format!("            return Err(anyhow!(\"{} not found\"));\n", model_name));
    content.push_str("        }\n\n");
    content.push_str("        Ok(())\n");
    content.push_str("    }\n\n");

    // List method
    content.push_str(&format!("    /// List {} records with pagination\n", model_name));
    content.push_str(&format!("    pub async fn list(pool: &DbPool, pagination: PaginationParams) -> Result<Paginator<{}>> {{\n", model_name));
    content.push_str(&format!("        let total = sqlx::query!(\"SELECT COUNT(*) as count FROM {}\")\n", model_snake));
    content.push_str("            .fetch_one(pool)\n");
    content.push_str("            .await\n");
    content.push_str(&format!("            .map_err(|e| anyhow!(\"Failed to count {}: {{}}\", e))?\n", model_name));
    content.push_str("            .count\n");
    content.push_str("            .unwrap_or(0) as u64;\n\n");
    content.push_str(&format!("        let records = sqlx::query_as!(\n"));
    content.push_str(&format!("            {},\n", model_name));
    content.push_str(&format!("            \"SELECT * FROM {} ORDER BY created_at DESC LIMIT $1 OFFSET $2\",\n", model_snake));
    content.push_str("            pagination.per_page as i64,\n");
    content.push_str("            pagination.offset() as i64\n");
    content.push_str("        )\n");
    content.push_str("        .fetch_all(pool)\n");
    content.push_str("        .await\n");
    content.push_str(&format!("        .map_err(|e| anyhow!(\"Failed to list {}: {{}}\", e))?;\n\n", model_name));
    content.push_str("        Ok(Paginator::new(records, total, pagination))\n");
    content.push_str("    }\n\n");

    // Exists method
    content.push_str(&format!("    /// Check if {} exists\n", model_name));
    content.push_str("    pub async fn exists(pool: &DbPool, id: String) -> Result<bool> {\n");
    content.push_str(&format!("        let count = sqlx::query!(\"SELECT COUNT(*) as count FROM {} WHERE id = $1\", id)\n", model_snake));
    content.push_str("            .fetch_one(pool)\n");
    content.push_str("            .await\n");
    content.push_str(&format!("            .map_err(|e| anyhow!(\"Failed to check if {} exists: {{}}\", e))?\n", model_name));
    content.push_str("            .count\n");
    content.push_str("            .unwrap_or(0);\n\n");
    content.push_str("        Ok(count > 0)\n");
    content.push_str("    }\n");
    content.push_str("}\n");

    content
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