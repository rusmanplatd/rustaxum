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
    let event_type = event.as_deref().unwrap_or("ExampleEvent");
    format!(r#"use anyhow::Result;
use serde::{{Deserialize, Serialize}};

// Import the event this listener handles
// use crate::app::events::{};

#[derive(Debug, Clone)]
pub struct {} {{
    // Add listener-specific configuration here
}}

impl {} {{
    pub fn new() -> Self {{
        Self {{}}
    }}

    // Handle the event synchronously
    pub async fn handle(&self, event: &{}) -> Result<()> {{
        println!("Handling event in {{}}: {{:?}}", stringify!({}), event);

        // Implement your event handling logic here
        self.process_event(event).await?;

        println!("Event handled successfully by {{}}", stringify!({}));
        Ok(())
    }}

    async fn process_event(&self, event: &{}) -> Result<()> {{
        // Implement your actual event processing logic here
        // This could be:
        // - Sending notifications
        // - Updating database records
        // - Triggering other events
        // - Calling external APIs
        // - etc.

        println!("Processing event: {{:?}}", event);

        // Example processing logic
        match event.event_type() {{
            "{}" => {{
                println!("Handling {} event");
                // Add specific handling for this event type
            }},
            _ => {{
                println!("Unknown event type: {{}}", event.event_type());
            }}
        }}

        Ok(())
    }}
}}

impl Default for {} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

// Listener trait for type-safe event handling
#[async_trait::async_trait]
pub trait EventListener<E> {{
    async fn handle(&self, event: &E) -> Result<()>;
}}

#[async_trait::async_trait]
impl EventListener<{}> for {} {{
    async fn handle(&self, event: &{}) -> Result<()> {{
        Self::handle(self, event).await
    }}
}}
"#, event_type, listener_name, listener_name, event_type, listener_name, listener_name, event_type, event_type, event_type, listener_name, event_type, listener_name, event_type)
}

fn generate_queued_listener_template(listener_name: &str, event: &Option<String>) -> String {
    let event_type = event.as_deref().unwrap_or("ExampleEvent");
    format!(r#"use anyhow::Result;
use serde::{{Deserialize, Serialize}};
use tokio::time::{{sleep, Duration}};

// Import the event this listener handles
// use crate::app::events::{};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {} {{
    pub id: String,
    pub event_data: String, // Serialized event data
    pub attempts: u32,
    pub max_attempts: u32,
    pub delay: Option<Duration>,
}}

impl {} {{
    pub fn new(event: &{}) -> Result<Self> {{
        Ok(Self {{
            id: ulid::Ulid::new().to_string(),
            event_data: serde_json::to_string(event)?,
            attempts: 0,
            max_attempts: 3,
            delay: None,
        }})
    }}

    pub fn with_delay(mut self, delay: Duration) -> Self {{
        self.delay = Some(delay);
        self
    }}

    pub fn max_attempts(mut self, attempts: u32) -> Self {{
        self.max_attempts = attempts;
        self
    }}

    // Handle the event asynchronously (queued)
    pub async fn handle(&mut self) -> Result<()> {{
        self.attempts += 1;

        // Add delay if specified
        if let Some(delay) = self.delay {{
            sleep(delay).await;
        }}

        println!("Handling queued event in {{}} (attempt {{}})", stringify!({}), self.attempts);

        // Deserialize the event
        let event: {} = serde_json::from_str(&self.event_data)?;

        // Process the event
        self.process_event(&event).await?;

        println!("Queued event handled successfully by {{}}", stringify!({}));
        Ok(())
    }}

    async fn process_event(&self, event: &{}) -> Result<()> {{
        // Implement your actual event processing logic here
        // This could be:
        // - Sending emails/notifications (queued)
        // - Processing files
        // - Making API calls
        // - Heavy computations
        // - Database operations
        // - etc.

        println!("Processing queued event: {{:?}}", event);

        // Example processing logic with simulated work
        match event.event_type() {{
            "{}" => {{
                println!("Handling {} event in queue");
                // Simulate some async work
                sleep(Duration::from_millis(100)).await;
                // Add specific handling for this event type
            }},
            _ => {{
                println!("Unknown event type: {{}}", event.event_type());
            }}
        }}

        Ok(())
    }}

    pub fn should_retry(&self) -> bool {{
        self.attempts < self.max_attempts
    }}

    pub async fn failed(&self, error: &anyhow::Error) {{
        println!("Queued listener {{}} failed: {{}}", self.id, error);

        if !self.should_retry() {{
            println!("Listener {{}} exceeded max attempts, moving to failed queue", self.id);
            // Here you could store the failed job in a dead letter queue
        }}
    }}
}}

// Queueable trait for async job processing
#[async_trait::async_trait]
pub trait QueueableListener {{
    async fn dispatch(self) -> Result<()>;
}}

#[async_trait::async_trait]
impl QueueableListener for {} {{
    async fn dispatch(mut self) -> Result<()> {{
        // This would typically add the listener job to a queue (Redis, database, etc.)
        // For now, we'll just execute it directly
        println!("Dispatching queued listener: {{}}", self.id);

        loop {{
            match self.handle().await {{
                Ok(()) => break,
                Err(e) => {{
                    self.failed(&e).await;
                    if !self.should_retry() {{
                        return Err(e);
                    }}
                    // In a real implementation, you'd re-queue the listener with a delay
                    sleep(Duration::from_secs(1)).await;
                }}
            }}
        }}

        Ok(())
    }}
}}

// Synchronous listener trait for immediate processing
#[async_trait::async_trait]
pub trait EventListener<E> {{
    async fn handle(&self, event: &E) -> Result<()>;
}}

// Helper function to create and queue the listener
pub async fn dispatch_listener(event: &{}) -> Result<()> {{
    let listener = {}::new(event)?;
    listener.dispatch().await
}}
"#, event_type, listener_name, listener_name, event_type, listener_name, event_type, listener_name, event_type, event_type, event_type, listener_name, event_type, listener_name)
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