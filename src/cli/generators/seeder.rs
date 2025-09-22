use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;

pub fn generate_seeder(name: &str) -> Result<()> {
    let seeder_name = to_pascal_case(name);
    let file_name = to_snake_case(&seeder_name);
    let file_path = format!("src/database/seeders/{}.rs", file_name);

    if Path::new(&file_path).exists() {
        return Err(anyhow!("Seeder {} already exists", seeder_name));
    }

    let content = generate_seeder_content(&seeder_name);

    fs::write(&file_path, content)?;
    println!("Seeder created: {}", file_path);

    // Update the seeders mod.rs file
    update_seeders_mod(&file_name)?;

    Ok(())
}

fn to_pascal_case(name: &str) -> String {
    name.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
            }
        })
        .collect()
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

fn generate_seeder_content(seeder_name: &str) -> String {
    format!(r#"use crate::database::DbPool;
use anyhow::Result;
use crate::database::seeder::Seeder;

pub struct {};

impl Seeder for {} {{
    fn name(&self) -> &'static str {{
        "{}"
    }}

    fn description(&self) -> Option<&'static str> {{
        Some("Seed data for {} table")
    }}

    fn run(&self, pool: &DbPool) -> Result<()> {{
        tracing::info!("Running {} seeder", self.name());

        // Add your seeding logic here
        // Example using Diesel:
        // use diesel::prelude::*;
        // use crate::schema::table_name;
        //
        // let mut conn = pool.get()?;
        // diesel::insert_into(table_name::table)
        //     .values((
        //         table_name::column1.eq("value1"),
        //         table_name::column2.eq("value2"),
        //     ))
        //     .execute(&mut conn)?;

        println!("{} seeder executed successfully");
        Ok(())
    }}
}}
"#, seeder_name, seeder_name, seeder_name, seeder_name, seeder_name, seeder_name)
}

fn update_seeders_mod(file_name: &str) -> Result<()> {
    let mod_path = "src/database/seeders/mod.rs";
    let module_declaration = format!("pub mod {};", file_name);

    if let Ok(current_content) = fs::read_to_string(mod_path) {
        if !current_content.contains(&module_declaration) {
            let new_content = format!("{}\n{}", current_content.trim(), module_declaration);
            fs::write(mod_path, new_content)?;
            println!("Updated seeders/mod.rs");
        }
    } else {
        // Create the mod.rs file if it doesn't exist
        fs::write(mod_path, module_declaration)?;
        println!("Created seeders/mod.rs");
    }

    Ok(())
}