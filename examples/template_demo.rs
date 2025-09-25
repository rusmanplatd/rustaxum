use rustaxum::app::services::template_service::TemplateService;
use serde_json::json;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the template service
    let template_service = TemplateService::new()?;

    println!("🎨 Handlebars Template Engine Demo");
    println!("====================================\n");

    // Test simple template rendering
    println!("1. Simple Template Rendering:");
    let simple_template = "Hello {{name}}! Welcome to {{app_name}}.";
    let simple_data = json!({
        "name": "Rust Developer",
        "app_name": "RustAxum"
    });

    let result = template_service.render_string(simple_template, &simple_data).await?;
    println!("   Template: {}", simple_template);
    println!("   Result: {}\n", result);

    // Test template with conditionals
    println!("2. Conditional Template:");
    let conditional_template = r#"
{{#if user}}
Hello {{user.name}}! You are logged in.
{{else}}
Please log in to continue.
{{/if}}"#;

    let with_user = json!({
        "user": {
            "name": "John Doe"
        }
    });

    let without_user = json!({});

    let result_with = template_service.render_string(conditional_template, &with_user).await?;
    let result_without = template_service.render_string(conditional_template, &without_user).await?;

    println!("   With user: {}", result_with.trim());
    println!("   Without user: {}\n", result_without.trim());

    // Test template with loops
    println!("3. Loop Template:");
    let loop_template = r#"
Features:
{{#each features}}
- {{name}}: {{description}}
{{/each}}"#;

    let loop_data = json!({
        "features": [
            {
                "name": "Authentication",
                "description": "Secure JWT-based auth"
            },
            {
                "name": "Templates",
                "description": "Handlebars template engine"
            },
            {
                "name": "Database",
                "description": "Diesel ORM with PostgreSQL"
            }
        ]
    });

    let result_loop = template_service.render_string(loop_template, &loop_data).await?;
    println!("{}", result_loop.trim());

    println!("\n✅ Handlebars integration is working correctly!");
    println!("📁 Templates are stored in: resources/views/");
    println!("🔧 Use TemplateResponse in controllers for easy rendering");

    Ok(())
}