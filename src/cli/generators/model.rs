use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;

pub async fn generate_model(name: &str, with_migration: bool) -> Result<()> {
    let model_name = to_pascal_case(name);
    let file_name = to_snake_case(&model_name);
    let file_path = format!("src/app/models/{}.rs", file_name);

    if Path::new(&file_path).exists() {
        return Err(anyhow!("Model {} already exists", model_name));
    }

    let content = generate_model_content(&model_name);

    fs::write(&file_path, content)?;
    println!("Model created: {}", file_path);

    // Update the models mod.rs file
    update_models_mod(&file_name)?;

    if with_migration {
        let migration_name = format!("create_{}_table", to_snake_case(&format!("{}s", model_name)));
        super::migration::generate_migration(&migration_name).await?;
    }

    Ok(())
}

fn to_pascal_case(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect()
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

fn generate_model_content(model_name: &str) -> String {

    format!(r#"use serde::{{Deserialize, Serialize}};
// use sqlx::{{FromRow, Row, postgres::PgRow}};
use ulid::Ulid;
use chrono::{{DateTime, Utc}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {} {{
    pub id: Ulid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}}

#[derive(Debug, Serialize, Deserialize)]
pub struct Create{} {{
    pub name: String,
}}

#[derive(Debug, Serialize, Deserialize)]
pub struct Update{} {{
    pub name: Option<String>,
}}

#[derive(Debug, Serialize)]
pub struct {}Response {{
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}}

impl {} {{
    pub fn new(name: String) -> Self {{
        let now = Utc::now();
        Self {{
            id: Ulid::new(),
            name,
            created_at: now,
            updated_at: now,
        }}
    }}

    pub fn to_response(&self) -> {}Response {{
        {}Response {{
            id: self.id.to_string(),
            name: self.name.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }}
    }}
}}

"#, model_name, model_name, model_name, model_name, model_name, model_name, model_name)
}

fn update_models_mod(file_name: &str) -> Result<()> {
    let mod_path = "src/app/models/mod.rs";
    let module_declaration = format!("pub mod {};", file_name);

    if let Ok(current_content) = fs::read_to_string(mod_path) {
        if !current_content.contains(&module_declaration) {
            let new_content = format!("{}\n{}", current_content.trim(), module_declaration);
            fs::write(mod_path, new_content)?;
            println!("Updated models/mod.rs");
        }
    }

    Ok(())
}