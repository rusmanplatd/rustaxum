use anyhow::Result;
use std::fs;
use std::path::Path;

pub async fn generate_resource(name: &str, collection: bool) -> Result<()> {
    let base_name = name.replace("Resource", "");
    let resource_name = if name.ends_with("Resource") {
        name.to_string()
    } else {
        format!("{}Resource", name)
    };

    let dir_path = "src/app/resources";
    fs::create_dir_all(dir_path)?;

    let file_path = format!("{}/{}.rs", dir_path, to_snake_case(&resource_name));

    let content = if collection {
        generate_collection_template(&resource_name, &base_name)
    } else {
        generate_resource_template(&resource_name, &base_name)
    };

    fs::write(&file_path, content)?;

    update_resources_mod(&resource_name)?;

    println!("Resource created successfully: {}", file_path);
    Ok(())
}

fn generate_resource_template(resource_name: &str, base_name: &str) -> String {
    format!(r#"use serde::{{Deserialize, Serialize}};
use crate::app::models::{};

#[derive(Debug, Serialize, Deserialize)]
pub struct {} {{
    pub id: String,
    // Add resource fields here
}}

impl {} {{
    pub fn from_model(model: {}) -> Self {{
        Self {{
            id: model.id,
            // Map model fields to resource fields
        }}
    }}

    pub fn collection(models: Vec<{}>) -> Vec<Self> {{
        models.into_iter().map(Self::from_model).collect()
    }}
}}
"#, base_name, resource_name, resource_name, base_name, base_name)
}

fn generate_collection_template(resource_name: &str, base_name: &str) -> String {
    let collection_name = resource_name.replace("Resource", "Collection");
    format!(r#"use serde::{{Deserialize, Serialize}};
use crate::app::models::{};

#[derive(Debug, Serialize, Deserialize)]
pub struct {} {{
    pub data: Vec<{}Item>,
    pub meta: CollectionMeta,
}}

#[derive(Debug, Serialize, Deserialize)]
pub struct {}Item {{
    pub id: String,
    // Add resource fields here
}}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionMeta {{
    pub total: usize,
    pub per_page: usize,
    pub current_page: usize,
    pub last_page: usize,
}}

impl {} {{
    pub fn from_models(models: Vec<{}>, page: usize, per_page: usize, total: usize) -> Self {{
        let last_page = (total + per_page - 1) / per_page;

        Self {{
            data: models.into_iter().map({}Item::from_model).collect(),
            meta: CollectionMeta {{
                total,
                per_page,
                current_page: page,
                last_page,
            }},
        }}
    }}
}}

impl {}Item {{
    pub fn from_model(model: {}) -> Self {{
        Self {{
            id: model.id,
            // Map model fields to resource fields
        }}
    }}
}}
"#, base_name, collection_name, resource_name, resource_name, collection_name, base_name, resource_name, resource_name, base_name)
}

fn update_resources_mod(resource_name: &str) -> Result<()> {
    let mod_path = "src/app/resources/mod.rs";
    let module_name = to_snake_case(resource_name);

    if !Path::new("src/app/resources").exists() {
        fs::create_dir_all("src/app/resources")?;
    }

    let mod_content = if Path::new(mod_path).exists() {
        let existing = fs::read_to_string(mod_path)?;
        if existing.contains(&format!("pub mod {};", module_name)) {
            return Ok(());
        }
        format!("{}\npub mod {};", existing.trim(), module_name)
    } else {
        format!("pub mod {};", module_name)
    };

    fs::write(mod_path, mod_content)?;
    Ok(())
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;

    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
            prev_is_upper = true;
        } else {
            result.push(c);
            prev_is_upper = false;
        }
    }

    result
}