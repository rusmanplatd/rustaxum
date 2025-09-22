use crate::database::DbPool;
use anyhow::Result;
use std::sync::OnceLock;

static DB_POOL: OnceLock<DbPool> = OnceLock::new();

pub fn initialize_pool(pool: DbPool) {
    DB_POOL.set(pool).expect("Database pool already initialized");
}

pub async fn get_connection() -> Result<&'static DbPool> {
    DB_POOL.get()
        .ok_or_else(|| anyhow::anyhow!("Database pool not initialized"))
}