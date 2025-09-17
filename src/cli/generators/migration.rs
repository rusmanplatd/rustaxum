use anyhow::Result;
use std::fs;
use std::path::Path;
use chrono::Utc;

pub async fn generate_migration(name: &str) -> Result<()> {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let migration_name = format!("{}_{}.sql", timestamp, name);
    let migrations_dir = "src/database/migrations";
    let file_path = format!("{}/{}", migrations_dir, migration_name);

    // Ensure migrations directory exists
    fs::create_dir_all(migrations_dir)?;

    if Path::new(&file_path).exists() {
        println!("Migration {} already exists", migration_name);
        return Ok(());
    }

    let content = generate_migration_content(name);

    fs::write(&file_path, content)?;
    println!("Migration created: {}", file_path);

    Ok(())
}

fn generate_migration_content(name: &str) -> String {
    if name.starts_with("create_") && name.ends_with("_table") {
        let table_name = name
            .strip_prefix("create_")
            .unwrap()
            .strip_suffix("_table")
            .unwrap();

        format!(r#"-- Create {} table
CREATE TABLE {} (
    id TEXT PRIMARY KEY,
    name VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add indexes
CREATE INDEX idx_{}_name ON {} (name);
CREATE INDEX idx_{}_created_at ON {} (created_at);
"#, table_name, table_name, table_name, table_name, table_name, table_name)
    } else if name.starts_with("add_") && name.contains("_to_") {
        let parts: Vec<&str> = name.split("_to_").collect();
        if parts.len() == 2 {
            let column_part = parts[0].strip_prefix("add_").unwrap_or("");
            let table_name = parts[1];

            format!(r#"-- Add column(s) to {} table
ALTER TABLE {} ADD COLUMN {} VARCHAR;

-- Add any necessary indexes
-- CREATE INDEX idx_{}_{} ON {} ({});
"#, table_name, table_name, column_part, table_name, column_part, table_name, column_part)
        } else {
            generate_generic_migration(name)
        }
    } else if name.starts_with("drop_") && name.ends_with("_table") {
        let table_name = name
            .strip_prefix("drop_")
            .unwrap()
            .strip_suffix("_table")
            .unwrap();

        format!(r#"-- Drop {} table
DROP TABLE IF EXISTS {};
"#, table_name, table_name)
    } else {
        generate_generic_migration(name)
    }
}

fn generate_generic_migration(name: &str) -> String {
    format!(r#"-- {}
-- Add your SQL statements here

-- Example:
-- CREATE TABLE example (
--     id TEXT PRIMARY KEY,
--     name VARCHAR NOT NULL,
--     created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
-- );

-- ALTER TABLE example ADD COLUMN new_column VARCHAR;

-- DROP TABLE IF EXISTS old_table;
"#, name.replace('_', " ").to_uppercase())
}