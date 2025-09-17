use sqlx::{PgPool, Row};
use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;
use crate::app::models::migration::Migration;

pub struct MigrationRunner {
    pool: PgPool,
    migrations_path: String,
}

#[derive(Debug)]
pub struct MigrationFile {
    pub filename: String,
    pub name: String,
    pub up_sql: String,
    pub down_sql: Option<String>,
}

impl MigrationRunner {
    pub fn new(pool: PgPool, migrations_path: String) -> Self {
        Self {
            pool,
            migrations_path,
        }
    }

    pub async fn ensure_migrations_table(&self) -> Result<()> {
        // Create table first
        let create_table = r#"
            CREATE TABLE IF NOT EXISTS migrations (
                id SERIAL PRIMARY KEY,
                migration VARCHAR(255) NOT NULL,
                batch INTEGER NOT NULL,
                executed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
            )
        "#;
        sqlx::query(create_table).execute(&self.pool).await?;

        // Create index separately
        let create_index = "CREATE INDEX IF NOT EXISTS idx_migrations_batch ON migrations(batch)";
        sqlx::query(create_index).execute(&self.pool).await?;

        Ok(())
    }

    pub async fn get_executed_migrations(&self) -> Result<Vec<Migration>> {
        let rows = sqlx::query("SELECT id, migration, batch, executed_at FROM migrations ORDER BY id")
            .fetch_all(&self.pool)
            .await?;

        let migrations = rows
            .into_iter()
            .map(|row| Migration {
                id: row.get("id"),
                migration: row.get("migration"),
                batch: row.get("batch"),
                executed_at: row.get("executed_at"),
            })
            .collect();

        Ok(migrations)
    }

    pub async fn get_pending_migrations(&self) -> Result<Vec<MigrationFile>> {
        let executed = self.get_executed_migrations().await?;
        let executed_names: std::collections::HashSet<String> =
            executed.into_iter().map(|m| m.migration).collect();

        let all_migrations = self.load_migration_files()?;
        let pending = all_migrations
            .into_iter()
            .filter(|m| !executed_names.contains(&m.name))
            .collect();

        Ok(pending)
    }

    pub async fn get_next_batch_number(&self) -> Result<i32> {
        let row = sqlx::query("SELECT COALESCE(MAX(batch), 0) + 1 as next_batch FROM migrations")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get("next_batch"))
    }

    pub async fn get_last_batch_number(&self) -> Result<Option<i32>> {
        let row = sqlx::query("SELECT MAX(batch) as last_batch FROM migrations")
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.and_then(|r| r.get("last_batch")))
    }

    pub async fn run_migrations(&self) -> Result<()> {
        self.ensure_migrations_table().await?;

        let pending = self.get_pending_migrations().await?;
        if pending.is_empty() {
            println!("No pending migrations found.");
            return Ok(());
        }

        let batch = self.get_next_batch_number().await?;
        println!("Running {} migrations in batch {}...", pending.len(), batch);

        for migration in pending {
            println!("Migrating: {}", migration.name);

            // Execute the migration SQL (split by semicolons and execute each statement)
            self.execute_sql_statements(&migration.up_sql, &migration.name).await?;

            // Record the migration
            sqlx::query("INSERT INTO migrations (migration, batch) VALUES ($1, $2)")
                .bind(&migration.name)
                .bind(batch)
                .execute(&self.pool)
                .await?;

            println!("Migrated: {}", migration.name);
        }

        println!("✅ All migrations completed successfully!");
        Ok(())
    }

    pub async fn rollback_migrations(&self, steps: Option<i32>) -> Result<()> {
        self.ensure_migrations_table().await?;

        let steps = steps.unwrap_or(1);
        let last_batch = self.get_last_batch_number().await?;

        if last_batch.is_none() {
            println!("No migrations to rollback.");
            return Ok(());
        }

        let last_batch = last_batch.unwrap();
        let target_batch = std::cmp::max(0, last_batch - steps + 1);

        let migrations_to_rollback = sqlx::query(
            "SELECT migration FROM migrations WHERE batch >= $1 ORDER BY id DESC"
        )
        .bind(target_batch)
        .fetch_all(&self.pool)
        .await?;

        if migrations_to_rollback.is_empty() {
            println!("No migrations to rollback.");
            return Ok(());
        }

        println!("Rolling back {} migrations...", migrations_to_rollback.len());

        for row in migrations_to_rollback {
            let migration_name: String = row.get("migration");
            println!("Rolling back: {}", migration_name);

            // Load migration file to get down SQL
            if let Some(migration_file) = self.load_migration_file(&migration_name)? {
                if let Some(down_sql) = migration_file.down_sql {
                    // Execute rollback SQL
                    self.execute_sql_statements(&down_sql, &migration_name).await
                        .map_err(|e| anyhow!("Failed to rollback migration {}: {}", migration_name, e))?;

                    // Remove from migrations table
                    sqlx::query("DELETE FROM migrations WHERE migration = $1")
                        .bind(&migration_name)
                        .execute(&self.pool)
                        .await?;

                    println!("Rolled back: {}", migration_name);
                } else {
                    println!("⚠️  No rollback SQL found for: {}", migration_name);
                    println!("   Remove manually or provide down migration SQL");
                }
            } else {
                println!("⚠️  Migration file not found for: {}", migration_name);
            }
        }

        println!("✅ Rollback completed!");
        Ok(())
    }

    pub async fn reset_migrations(&self) -> Result<()> {
        self.ensure_migrations_table().await?;

        println!("Resetting all migrations...");

        let migrations = sqlx::query("SELECT migration FROM migrations ORDER BY id DESC")
            .fetch_all(&self.pool)
            .await?;

        for row in migrations {
            let migration_name: String = row.get("migration");
            println!("Rolling back: {}", migration_name);

            if let Some(migration_file) = self.load_migration_file(&migration_name)? {
                if let Some(down_sql) = migration_file.down_sql {
                    self.execute_sql_statements(&down_sql, &migration_name).await
                        .map_err(|e| anyhow!("Failed to rollback migration {}: {}", migration_name, e))?;
                    println!("Rolled back: {}", migration_name);
                }
            }
        }

        // Clear migrations table
        sqlx::query("DELETE FROM migrations").execute(&self.pool).await?;
        println!("✅ All migrations reset!");
        Ok(())
    }

    pub async fn refresh_migrations(&self) -> Result<()> {
        println!("Refreshing migrations (reset + migrate)...");
        self.reset_migrations().await?;
        self.run_migrations().await?;
        Ok(())
    }

    pub async fn show_status(&self) -> Result<()> {
        self.ensure_migrations_table().await?;

        let executed = self.get_executed_migrations().await?;
        let pending = self.get_pending_migrations().await?;

        println!("\n📊 Migration Status");
        println!("==================");

        if !executed.is_empty() {
            println!("\n✅ Executed Migrations:");
            for migration in &executed {
                println!("  [Batch {}] {} ({})",
                    migration.batch,
                    migration.migration,
                    migration.executed_at.format("%Y-%m-%d %H:%M:%S")
                );
            }
        }

        if !pending.is_empty() {
            println!("\n⏳ Pending Migrations:");
            for migration in &pending {
                println!("  {}", migration.name);
            }
        }

        if executed.is_empty() && pending.is_empty() {
            println!("\nNo migrations found.");
        }

        println!("\nSummary: {} executed, {} pending", executed.len(), pending.len());
        Ok(())
    }

    fn load_migration_files(&self) -> Result<Vec<MigrationFile>> {
        let mut migrations = Vec::new();
        let path = Path::new(&self.migrations_path);

        if !path.exists() {
            return Ok(migrations);
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if let Some(extension) = file_path.extension() {
                if extension == "sql" {
                    if let Some(filename) = file_path.file_name() {
                        let filename = filename.to_string_lossy().to_string();
                        let name = filename.trim_end_matches(".sql").to_string();

                        let content = fs::read_to_string(&file_path)?;
                        let (up_sql, down_sql) = self.parse_migration_content(&content);

                        migrations.push(MigrationFile {
                            filename,
                            name,
                            up_sql,
                            down_sql,
                        });
                    }
                }
            }
        }

        // Sort migrations by filename to ensure proper order
        migrations.sort_by(|a, b| a.filename.cmp(&b.filename));
        Ok(migrations)
    }

    fn load_migration_file(&self, migration_name: &str) -> Result<Option<MigrationFile>> {
        let migrations = self.load_migration_files()?;
        Ok(migrations.into_iter().find(|m| m.name == migration_name))
    }

    fn parse_migration_content(&self, content: &str) -> (String, Option<String>) {
        // Look for -- DOWN or -- down marker to separate up and down migrations
        let down_markers = ["-- DOWN", "-- down", "-- Down"];

        for marker in &down_markers {
            if let Some(pos) = content.find(marker) {
                let up_sql = content[..pos].trim().to_string();
                let down_sql = content[pos + marker.len()..].trim().to_string();

                if !down_sql.is_empty() {
                    return (up_sql, Some(down_sql));
                }
            }
        }

        // No down migration found, return just up migration
        (content.trim().to_string(), None)
    }

    async fn execute_sql_statements(&self, sql: &str, migration_name: &str) -> Result<()> {
        // Split SQL by semicolons and execute each statement separately
        let statements: Vec<&str> = sql
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && !s.starts_with("--"))
            .collect();

        for statement in statements {
            if !statement.is_empty() {
                sqlx::query(statement)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| anyhow!("Failed to execute SQL statement in migration {}: {}\nStatement: {}", migration_name, e, statement))?;
            }
        }

        Ok(())
    }
}