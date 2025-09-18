use anyhow::Result;
use std::fs;
use std::path::Path;

pub async fn generate_job(name: &str, sync: bool) -> Result<()> {
    let job_name = if name.ends_with("Job") {
        name.to_string()
    } else {
        format!("{}Job", name)
    };

    let dir_path = "src/app/jobs";
    fs::create_dir_all(dir_path)?;

    let file_path = format!("{}/{}.rs", dir_path, to_snake_case(&job_name));

    let content = if sync {
        generate_sync_job_template(&job_name)
    } else {
        generate_async_job_template(&job_name)
    };

    fs::write(&file_path, content)?;

    update_jobs_mod(&job_name)?;

    println!("Job created successfully: {}", file_path);
    Ok(())
}

fn generate_async_job_template(job_name: &str) -> String {
    format!(r#"use anyhow::Result;
use serde::{{Deserialize, Serialize}};
use tokio::time::{{sleep, Duration}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {} {{
    pub id: String,
    pub data: JobData,
    pub attempts: u32,
    pub max_attempts: u32,
    pub delay: Option<Duration>,
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobData {{
    // Add job-specific data fields here
    pub message: String,
}}

impl {} {{
    pub fn new(data: JobData) -> Self {{
        Self {{
            id: ulid::Ulid::new().to_string(),
            data,
            attempts: 0,
            max_attempts: 3,
            delay: None,
        }}
    }}

    pub fn with_delay(mut self, delay: Duration) -> Self {{
        self.delay = Some(delay);
        self
    }}

    pub fn max_attempts(mut self, attempts: u32) -> Self {{
        self.max_attempts = attempts;
        self
    }}

    pub async fn handle(&mut self) -> Result<()> {{
        self.attempts += 1;

        // Add delay if specified
        if let Some(delay) = self.delay {{
            sleep(delay).await;
        }}

        // Implement your job logic here
        println!("Processing job {{}} (attempt {{}})", self.id, self.attempts);
        println!("Job data: {{}}", self.data.message);

        // Simulate some async work
        sleep(Duration::from_millis(100)).await;

        // Example job logic - replace with your implementation
        self.process_job().await?;

        println!("Job {{}} completed successfully", self.id);
        Ok(())
    }}

    async fn process_job(&self) -> Result<()> {{
        // Implement your actual job processing logic here
        // This could be:
        // - Sending emails
        // - Processing files
        // - Making API calls
        // - Database operations
        // - etc.

        println!("Executing job logic for: {{}}", self.data.message);
        Ok(())
    }}

    pub fn should_retry(&self) -> bool {{
        self.attempts < self.max_attempts
    }}

    pub async fn failed(&self, error: &anyhow::Error) {{
        // Handle job failure
        println!("Job {{}} failed: {{}}", self.id, error);

        if !self.should_retry() {{
            println!("Job {{}} exceeded max attempts, moving to failed queue", self.id);
            // Here you could store the job in a failed jobs table
        }}
    }}
}}

// Queue trait for job processing
#[async_trait::async_trait]
pub trait Queueable {{
    async fn dispatch(self) -> Result<()>;
}}

#[async_trait::async_trait]
impl Queueable for {} {{
    async fn dispatch(mut self) -> Result<()> {{
        // This would typically add the job to a queue (Redis, database, etc.)
        // For now, we'll just execute it directly
        println!("Dispatching job: {{}}", self.id);

        loop {{
            match self.handle().await {{
                Ok(()) => break,
                Err(e) => {{
                    self.failed(&e).await;
                    if !self.should_retry() {{
                        return Err(e);
                    }}
                    // In a real implementation, you'd re-queue the job with a delay
                    sleep(Duration::from_secs(1)).await;
                }}
            }}
        }}

        Ok(())
    }}
}}
"#, job_name, job_name, job_name)
}

fn generate_sync_job_template(job_name: &str) -> String {
    format!(r#"use anyhow::Result;
use serde::{{Deserialize, Serialize}};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {} {{
    pub id: String,
    pub data: JobData,
    pub attempts: u32,
    pub max_attempts: u32,
}}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobData {{
    // Add job-specific data fields here
    pub message: String,
}}

impl {} {{
    pub fn new(data: JobData) -> Self {{
        Self {{
            id: ulid::Ulid::new().to_string(),
            data,
            attempts: 0,
            max_attempts: 3,
        }}
    }}

    pub fn max_attempts(mut self, attempts: u32) -> Self {{
        self.max_attempts = attempts;
        self
    }}

    pub fn handle(&mut self) -> Result<()> {{
        self.attempts += 1;

        // Implement your job logic here
        println!("Processing sync job {{}} (attempt {{}})", self.id, self.attempts);
        println!("Job data: {{}}", self.data.message);

        // Example job logic - replace with your implementation
        self.process_job()?;

        println!("Sync job {{}} completed successfully", self.id);
        Ok(())
    }}

    fn process_job(&self) -> Result<()> {{
        // Implement your actual job processing logic here
        // This could be:
        // - File operations
        // - Calculations
        // - Data transformations
        // - Synchronous API calls
        // - etc.

        println!("Executing sync job logic for: {{}}", self.data.message);
        Ok(())
    }}

    pub fn should_retry(&self) -> bool {{
        self.attempts < self.max_attempts
    }}

    pub fn failed(&self, error: &anyhow::Error) {{
        // Handle job failure
        println!("Sync job {{}} failed: {{}}", self.id, error);

        if !self.should_retry() {{
            println!("Sync job {{}} exceeded max attempts", self.id);
        }}
    }}
}}

// Sync execution trait
pub trait SyncExecutable {{
    fn execute(self) -> Result<()>;
}}

impl SyncExecutable for {} {{
    fn execute(mut self) -> Result<()> {{
        // Execute the job synchronously
        println!("Executing sync job: {{}}", self.id);

        loop {{
            match self.handle() {{
                Ok(()) => break,
                Err(e) => {{
                    self.failed(&e);
                    if !self.should_retry() {{
                        return Err(e);
                    }}
                    // In a sync job, we might add a small delay before retry
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }}
            }}
        }}

        Ok(())
    }}
}}
"#, job_name, job_name, job_name)
}

fn update_jobs_mod(job_name: &str) -> Result<()> {
    let mod_path = "src/app/jobs/mod.rs";
    let module_name = to_snake_case(job_name);

    if !Path::new("src/app/jobs").exists() {
        fs::create_dir_all("src/app/jobs")?;
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