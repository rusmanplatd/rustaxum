use anyhow::Result;
use std::fs;
use std::path::Path;

pub async fn generate_policy(name: &str, model: Option<String>) -> Result<()> {
    let policy_name = if name.ends_with("Policy") {
        name.to_string()
    } else {
        format!("{}Policy", name)
    };

    let dir_path = "src/app/policies";
    fs::create_dir_all(dir_path)?;

    let file_path = format!("{}/{}.rs", dir_path, to_snake_case(&policy_name));

    let content = generate_policy_template(&policy_name, &model);

    fs::write(&file_path, content)?;

    update_policies_mod(&policy_name)?;

    println!("Policy created successfully: {}", file_path);
    Ok(())
}

fn generate_policy_template(policy_name: &str, model: &Option<String>) -> String {
    let model_name = model.as_deref().unwrap_or("Model");

    format!(r#"use anyhow::Result;
use serde::{{Deserialize, Serialize}};

// Import the model this policy applies to
// use crate::app::models::{};

#[derive(Debug, Clone)]
pub struct {} {{
    // Add policy-specific configuration here
}}

impl {} {{
    pub fn new() -> Self {{
        Self {{}}
    }}

    // Determine whether the user can view any models
    pub async fn view_any(&self, user: &User) -> Result<bool> {{
        // Implement authorization logic
        // Example: Check if user has admin role
        Ok(user.is_admin() || user.has_permission("view_any"))
    }}

    // Determine whether the user can view the model
    pub async fn view(&self, user: &User, model: &{}) -> Result<bool> {{
        // Implement authorization logic
        // Example: User can view if they own it or have permission
        Ok(user.id() == model.user_id ||
           user.has_permission("view") ||
           self.view_any(user).await?)
    }}

    // Determine whether the user can create models
    pub async fn create(&self, user: &User) -> Result<bool> {{
        // Implement authorization logic
        // Example: Check if user has create permission
        Ok(user.has_permission("create") ||
           user.is_admin())
    }}

    // Determine whether the user can update the model
    pub async fn update(&self, user: &User, model: &{}) -> Result<bool> {{
        // Implement authorization logic
        // Example: User can update if they own it or have permission
        Ok(user.id() == model.user_id ||
           user.has_permission("update") ||
           user.is_admin())
    }}

    // Determine whether the user can delete the model
    pub async fn delete(&self, user: &User, model: &{}) -> Result<bool> {{
        // Implement authorization logic
        // Example: User can delete if they own it or have permission
        Ok(user.id() == model.user_id ||
           user.has_permission("delete") ||
           user.is_admin())
    }}
}}

impl Default for {} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

// User trait for authorization (you should implement this based on your User model)
pub trait User {{
    fn id(&self) -> &str;
    fn is_admin(&self) -> bool;
    fn has_permission(&self, permission: &str) -> bool;
}}
"#, model_name, policy_name, policy_name, model_name, model_name, model_name, policy_name)
}

fn update_policies_mod(policy_name: &str) -> Result<()> {
    let mod_path = "src/app/policies/mod.rs";
    let module_name = to_snake_case(policy_name);

    if !Path::new("src/app/policies").exists() {
        fs::create_dir_all("src/app/policies")?;
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