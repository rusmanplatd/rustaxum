use anyhow::Result;
use std::fs;
use std::path::Path;

pub async fn generate_event(name: &str) -> Result<()> {
    let event_name = if name.ends_with("Event") {
        name.to_string()
    } else {
        format!("{}Event", name)
    };

    let dir_path = "src/app/events";
    fs::create_dir_all(dir_path)?;

    let file_path = format!("{}/{}.rs", dir_path, to_snake_case(&event_name));

    let content = generate_event_template(&event_name);

    fs::write(&file_path, content)?;

    update_events_mod(&event_name)?;

    println!("Event created successfully: {}", file_path);
    Ok(())
}

fn generate_event_template(event_name: &str) -> String {
    format!(r#"use anyhow::Result;
use serde::{{Deserialize, Serialize}};
use std::collections::HashMap;
use crate::app::events::{{Event, Dispatchable, InteractsWithSockets, SerializesModels, ShouldQueue}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {} {{
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    // Add event-specific data fields here
    pub message: String,
    pub user_id: Option<String>,
    pub metadata: HashMap<String, String>,
}}

impl {} {{
    pub fn new(message: String) -> Self {{
        Self {{
            id: ulid::Ulid::new().to_string(),
            timestamp: chrono::Utc::now(),
            message,
            user_id: None,
            metadata: HashMap::new(),
        }}
    }}

    pub fn with_user_id(mut self, user_id: String) -> Self {{
        self.user_id = Some(user_id);
        self
    }}

    pub fn with_metadata(mut self, key: String, value: String) -> Self {{
        self.metadata.insert(key, value);
        self
    }}
}}

impl Event for {} {{
    fn event_name(&self) -> &'static str {{
        "{}"
    }}

    fn to_json(&self) -> serde_json::Value {{
        serde_json::to_value(self).unwrap_or_default()
    }}

    fn should_queue(&self) -> bool {{
        true
    }}

    fn queue_name(&self) -> Option<&str> {{
        Some("events")
    }}
}}

// Uncomment the following to make this event queueable:
// impl ShouldQueue for {} {{}}

// Uncomment the following to enable socket interactions:
// impl InteractsWithSockets for {} {{
//     fn socket_id(&self) -> Option<String> {{
//         // Return socket ID if available
//         None
//     }}
//
//     fn broadcast_to_everyone_except(&self, socket_ids: Vec<String>) -> Vec<String> {{
//         socket_ids
//     }}
// }}

// Uncomment the following to enable model serialization:
// impl SerializesModels for {} {{
//     fn prepare_for_serialization(&self) {{
//         // Prepare models for serialization
//     }}
//
//     fn restore_after_serialization(&self) {{
//         // Restore models after serialization
//     }}
// }}
"#, event_name, event_name, event_name, event_name.replace("Event", ""), event_name, event_name, event_name)
}

fn update_events_mod(event_name: &str) -> Result<()> {
    let mod_path = "src/app/events/mod.rs";
    let module_name = to_snake_case(event_name);

    if !Path::new("src/app/events").exists() {
        fs::create_dir_all("src/app/events")?;
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