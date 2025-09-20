use anyhow::Result;
use std::fs;
use std::path::Path;

pub async fn generate_listener(name: &str, event: Option<String>, queued: bool) -> Result<()> {
    let listener_name = if name.ends_with("Listener") {
        name.to_string()
    } else {
        format!("{}Listener", name)
    };

    let dir_path = "src/app/listeners";
    fs::create_dir_all(dir_path)?;

    let file_path = format!("{}/{}.rs", dir_path, to_snake_case(&listener_name));

    let content = if queued {
        generate_queued_listener_template(&listener_name, &event)
    } else {
        generate_sync_listener_template(&listener_name, &event)
    };

    fs::write(&file_path, content)?;

    update_listeners_mod(&listener_name)?;

    println!("Listener created successfully: {}", file_path);
    Ok(())
}

fn generate_sync_listener_template(listener_name: &str, event: &Option<String>) -> String {
    let event_name = event.as_deref().unwrap_or("UserRegistered");
    format!(r#"use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use crate::app::events::{{Event, EventListener}};

#[derive(Debug)]
pub struct {} {{
    pub id: String,
}}

impl {} {{
    pub fn new() -> Self {{
        Self {{
            id: ulid::Ulid::new().to_string(),
        }}
    }}
}}

impl Default for {} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

#[async_trait]
impl EventListener for {} {{
    async fn handle(&self, event: Arc<dyn Event>) -> Result<()> {{
        println!("{} handling event: {{}}", event.event_name());

        if event.event_name() == "{}" {{
            let event_data = event.to_json();

            println!("Processing {} event with data: {{:?}}", event_data);

            // Implement your event handling logic here
            // This could be:
            // - Sending notifications
            // - Updating database records
            // - Triggering other events
            // - Calling external APIs
            // - etc.

            // Simulate some processing
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            println!("{} processed successfully!");
        }}

        Ok(())
    }}

    fn should_queue(&self) -> bool {{
        false
    }}

    fn queue_name(&self) -> Option<&str> {{
        None
    }}
}}
"#, listener_name, listener_name, listener_name, listener_name, listener_name, event_name, event_name, event_name)
}

fn generate_queued_listener_template(listener_name: &str, event: &Option<String>) -> String {
    let event_name = event.as_deref().unwrap_or("UserRegistered");
    format!(r#"use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use crate::app::events::{{Event, EventListener}};

#[derive(Debug)]
pub struct {} {{
    pub id: String,
}}

impl {} {{
    pub fn new() -> Self {{
        Self {{
            id: ulid::Ulid::new().to_string(),
        }}
    }}
}}

impl Default for {} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

#[async_trait]
impl EventListener for {} {{
    async fn handle(&self, event: Arc<dyn Event>) -> Result<()> {{
        println!("{} handling event: {{}}", event.event_name());

        if event.event_name() == "{}" {{
            let event_data = event.to_json();

            println!("Processing {} event with data: {{:?}}", event_data);

            // Implement your event handling logic here
            // This could be:
            // - Sending emails/notifications (queued)
            // - Processing files
            // - Making API calls
            // - Heavy computations
            // - Database operations
            // - etc.

            // Simulate some async work for queued processing
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            println!("{} processed successfully!");
        }}

        Ok(())
    }}

    fn should_queue(&self) -> bool {{
        true
    }}

    fn queue_name(&self) -> Option<&str> {{
        Some("listeners")
    }}
}}
"#, listener_name, listener_name, listener_name, listener_name, listener_name, event_name, event_name, event_name)
}

fn update_listeners_mod(listener_name: &str) -> Result<()> {
    let mod_path = "src/app/listeners/mod.rs";
    let module_name = to_snake_case(listener_name);

    if !Path::new("src/app/listeners").exists() {
        fs::create_dir_all("src/app/listeners")?;
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