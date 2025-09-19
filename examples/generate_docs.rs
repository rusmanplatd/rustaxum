use rustaxum::app::docs::ApiDoc;
use utoipa::OpenApi;

fn main() {
    println!("Generating OpenAPI documentation...\n");

    // Generate the OpenAPI specification
    let openapi = ApiDoc::openapi();

    // Convert to JSON
    let json_spec = serde_json::to_string_pretty(&openapi).unwrap();

    println!("OpenAPI 3.0 Specification (JSON):");
    println!("{}", json_spec);

    // Also show basic info
    println!("\n=== API Documentation Summary ===");
    println!("Title: {}", openapi.info.title);
    println!("Version: {}", openapi.info.version);
    println!("Description: {}", openapi.info.description.as_ref().unwrap_or(&"No description".to_string()));

    let paths = &openapi.paths;
    println!("Number of documented endpoints: {}", paths.paths.len());

    println!("\nAvailable endpoints:");
    for (path, _) in paths.paths.iter() {
        println!("  {}", path);
    }

    if let Some(components) = &openapi.components {
        let schemas = &components.schemas;
        println!("\nDocumented schemas: {}", schemas.len());
        for (name, _) in schemas.iter() {
            println!("  {}", name);
        }
    }

    println!("\n=== Documentation URLs ===");
    println!("Once the server is running, you can access:");
    println!("• OpenAPI JSON: http://localhost:3000/api/docs/openapi.json");
    println!("• OpenAPI YAML: http://localhost:3000/api/docs/openapi.yaml");
    println!("• Swagger UI: http://localhost:3000/docs/swagger");
    println!("• ReDoc: http://localhost:3000/docs/redoc");
    println!("• RapiDoc: http://localhost:3000/docs/rapidoc");

    println!("\n=== Usage Examples ===");
    println!("1. Start the server: cargo run --bin rustaxum");
    println!("2. View interactive docs: Open http://localhost:3000/docs/swagger in your browser");
    println!("3. Get raw OpenAPI spec: curl http://localhost:3000/api/docs/openapi.json");

    println!("\nThe documentation is automatically generated from:");
    println!("• Function comments with utoipa::path macros");
    println!("• Struct field documentation with ToSchema derives");
    println!("• Request/Response model definitions");
}