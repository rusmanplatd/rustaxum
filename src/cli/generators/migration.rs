use anyhow::Result;
use std::fs;
use std::path::Path;
use chrono::Utc;

pub async fn generate_migration(name: &str) -> Result<()> {
    let timestamp = Utc::now().format("%Y_%m_%d_%H%M%S").to_string();
    let migration_name = format!("{}_{}", timestamp, name);
    let migrations_dir = "src/database/migrations";
    let up_file_path = format!("{}/{}.up.sql", migrations_dir, migration_name);
    let down_file_path = format!("{}/{}.down.sql", migrations_dir, migration_name);

    // Ensure migrations directory exists
    fs::create_dir_all(migrations_dir)?;

    if Path::new(&up_file_path).exists() || Path::new(&down_file_path).exists() {
        println!("Migration {} already exists", migration_name);
        return Ok(());
    }

    let (up_content, down_content) = generate_migration_content(name);

    fs::write(&up_file_path, up_content)?;
    fs::write(&down_file_path, down_content)?;

    println!("Migration created:");
    println!("  Up:   {}", up_file_path);
    println!("  Down: {}", down_file_path);

    Ok(())
}

fn generate_migration_content(name: &str) -> (String, String) {
    if name.starts_with("create_") && name.ends_with("_table") {
        let table_name = name
            .strip_prefix("create_")
            .unwrap()
            .strip_suffix("_table")
            .unwrap();

        let up_content = format!(r#"-- Create {} table
CREATE TABLE {} (
    id TEXT PRIMARY KEY,
    name VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_{}_name ON {} (name);
CREATE INDEX idx_{}_created_at ON {} (created_at);
"#, table_name, table_name, table_name, table_name, table_name, table_name);

        let down_content = format!(r#"-- Drop {} table
DROP TABLE IF EXISTS {};
"#, table_name, table_name);

        (up_content, down_content)
    } else if name.starts_with("add_") && name.contains("_to_") {
        let parts: Vec<&str> = name.split("_to_").collect();
        if parts.len() == 2 {
            let column_part = parts[0].strip_prefix("add_").unwrap_or("");
            let table_name = parts[1];

            let up_content = format!(r#"-- Add column(s) to {} table
ALTER TABLE {} ADD COLUMN {} VARCHAR;

-- Add any necessary indexes
-- CREATE INDEX idx_{}_{} ON {} ({});
"#, table_name, table_name, column_part, table_name, column_part, table_name, column_part);

            let down_content = format!(r#"-- Remove column(s) from {} table
ALTER TABLE {} DROP COLUMN IF EXISTS {};

-- Drop any indexes
-- DROP INDEX IF EXISTS idx_{}_{};
"#, table_name, table_name, column_part, table_name, column_part);

            (up_content, down_content)
        } else {
            generate_generic_migration(name)
        }
    } else if name.starts_with("drop_") && name.ends_with("_table") {
        let table_name = name
            .strip_prefix("drop_")
            .unwrap()
            .strip_suffix("_table")
            .unwrap();

        let up_content = format!(r#"-- Drop {} table
DROP TABLE IF EXISTS {};
"#, table_name, table_name);

        let down_content = format!(r#"-- Recreate {} table (you may need to customize this)
CREATE TABLE {} (
    id TEXT PRIMARY KEY,
    -- Add columns here
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
"#, table_name, table_name);

        (up_content, down_content)
    } else {
        generate_generic_migration(name)
    }
}

fn generate_generic_migration(name: &str) -> (String, String) {
    let up_content = format!(r#"-- {}
-- Add your SQL statements here

-- Example:
-- CREATE TABLE example (
--     id TEXT PRIMARY KEY,
--     name VARCHAR NOT NULL,
--     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
-- );

-- ALTER TABLE example ADD COLUMN new_column VARCHAR;

-- DROP TABLE IF EXISTS old_table;
"#, name.replace('_', " ").to_uppercase());

    let down_content = format!(r#"-- Rollback {}
-- Add rollback SQL statements here

-- Example:
-- DROP TABLE IF EXISTS example;

-- ALTER TABLE example DROP COLUMN IF EXISTS new_column;

-- CREATE TABLE old_table (
--     id TEXT PRIMARY KEY,
--     name VARCHAR NOT NULL
-- );
"#, name.replace('_', " ").to_uppercase());

    (up_content, down_content)
}