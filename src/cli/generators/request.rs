use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;

pub async fn generate_request(name: &str) -> Result<()> {
    let request_name = format_request_name(name);
    let file_name = to_snake_case(&request_name);
    let file_path = format!("src/app/http/requests/{}.rs", file_name);

    if Path::new(&file_path).exists() {
        return Err(anyhow!("Request {} already exists", request_name));
    }

    let content = generate_form_request_content(&request_name);

    fs::write(&file_path, content)?;
    println!("FormRequest created: {}", file_path);

    // Update the requests mod.rs file
    update_requests_mod(&file_name, &request_name)?;

    Ok(())
}

fn format_request_name(name: &str) -> String {
    if name.ends_with("Request") {
        name.to_string()
    } else {
        format!("{}Request", name)
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

fn generate_form_request_content(request_name: &str) -> String {
    format!(r#"use std::collections::HashMap;
use serde::{{Deserialize, Serialize}};
use async_trait::async_trait;

use crate::app::http::form_request::FormRequest;
use crate::app::validation::{{Rule, required, string, email, min, max}};
use crate::impl_form_request_extractor;

/// {request_name} form request
#[derive(Deserialize, Serialize)]
pub struct {request_name} {{
    pub name: String,
    pub email: String,
    // Add your fields here
}}

#[async_trait]
impl FormRequest for {request_name} {{
    fn rules() -> HashMap<&'static str, Vec<Rule>> {{
        let mut rules = HashMap::new();
        rules.insert("name", vec![required(), string(), min(2)]);
        rules.insert("email", vec![required(), email()]);
        // Add your validation rules here
        rules
    }}

    fn messages() -> HashMap<&'static str, &'static str> {{
        let mut messages = HashMap::new();
        messages.insert("name.required", "Name is required");
        messages.insert("name.min", "Name must be at least 2 characters");
        messages.insert("email.required", "Email is required");
        messages.insert("email.email", "Email must be a valid email address");
        // Add your custom messages here
        messages
    }}

    fn attributes() -> HashMap<&'static str, &'static str> {{
        let mut attributes = HashMap::new();
        // Add custom attribute names here
        // attributes.insert("field_name", "display name");
        attributes
    }}

    // Optional: Override authorization logic
    // fn authorize(&self) -> bool {{
    //     // Add your authorization logic here
    //     true
    // }}

    // Optional: Prepare data before validation
    // fn prepare_for_validation(&mut self) {{
    //     // Modify request data before validation
    //     // e.g., self.email = self.email.to_lowercase();
    // }}
}}

impl_form_request_extractor!({request_name});
"#, request_name = request_name)
}

fn update_requests_mod(file_name: &str, request_name: &str) -> Result<()> {
    let mod_path = "src/app/http/requests/mod.rs";

    // Read existing content
    let existing_content = fs::read_to_string(mod_path)
        .unwrap_or_else(|_| String::from(""));

    // Check if module is already declared
    let module_declaration = format!("pub mod {};", file_name);
    let export_declaration = format!("pub use {}::{};", file_name, request_name);

    if existing_content.contains(&module_declaration) {
        return Ok(()); // Already exists
    }

    let lines: Vec<&str> = existing_content.lines().collect();
    let mut new_lines = Vec::new();
    let mut found_mod_section = false;
    let mut _found_use_section = false;

    // Add module declaration in the right place
    for line in &lines {
        if line.starts_with("pub mod ") {
            found_mod_section = true;
        } else if found_mod_section && !line.starts_with("pub mod ") && !line.trim().is_empty() {
            // We've moved past the mod declarations
            if !new_lines.iter().any(|l: &&str| l.contains(&module_declaration)) {
                new_lines.push(&module_declaration);
            }
            found_mod_section = false;
        }

        if line.starts_with("pub use ") {
            _found_use_section = true;
        }

        new_lines.push(line);
    }

    // If no mod section exists, add at the beginning
    if !found_mod_section && !new_lines.iter().any(|l| l.contains(&module_declaration)) {
        new_lines.insert(0, &module_declaration);
        new_lines.insert(1, "");
    }

    // Add use declaration
    if !new_lines.iter().any(|l: &&str| l.contains(&export_declaration)) {
        new_lines.push(&export_declaration);
    }

    let new_content = new_lines.join("\n");
    fs::write(mod_path, new_content)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_request_name() {
        assert_eq!(format_request_name("User"), "UserRequest");
        assert_eq!(format_request_name("UserRequest"), "UserRequest");
        assert_eq!(format_request_name("CreateUser"), "CreateUserRequest");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("UserRequest"), "user_request");
        assert_eq!(to_snake_case("CreateUserRequest"), "create_user_request");
        assert_eq!(to_snake_case("HTTPRequest"), "h_t_t_p_request");
    }
}