use anyhow::Result;
use std::env;

pub mod file_driver;
pub mod database_driver;
pub mod redis_driver;
pub mod array_driver;

pub use file_driver::FileSessionHandler;
pub use database_driver::DatabaseSessionHandler;
pub use redis_driver::RedisSessionHandler;
pub use array_driver::ArraySessionHandler;

use super::SessionHandler;

pub async fn create_session_handler(
    driver: &str,
    config: &crate::config::session::SessionConfig,
    pool: Option<&crate::database::DbPool>,
    _redis_pool: Option<()>, // Placeholder for future Redis pool integration
) -> Result<Box<dyn SessionHandler>> {
    match driver {
        "file" => Ok(Box::new(FileSessionHandler::new(&config.files)?)),
        "database" => {
            let pool = pool.ok_or_else(|| anyhow::anyhow!("Database pool required for database session driver"))?;
            Ok(Box::new(DatabaseSessionHandler::new(pool.clone())))
        }
        "redis" => {
            let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
            Ok(Box::new(RedisSessionHandler::new(&redis_url)?))
        }
        "array" => Ok(Box::new(ArraySessionHandler::new())),
        _ => Err(anyhow::anyhow!("Unsupported session driver: {}", driver)),
    }
}