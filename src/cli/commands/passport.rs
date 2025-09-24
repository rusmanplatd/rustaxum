use anyhow::Result;
use crate::database::DbPool;
use ulid::Ulid;

use crate::cli::PassportCommands;
use crate::app::services::oauth::{ClientService, ScopeService, TokenService};
use crate::app::models::oauth::{CreateClient, CreateScope};

pub async fn handle_passport_command(cmd: PassportCommands) -> Result<()> {
    let config = crate::config::Config::load()?;
    let pool = crate::database::create_pool(&config)?;

    match cmd {
        PassportCommands::Install => handle_install(&pool).await,
        PassportCommands::CreateClient { name, redirect_uris, personal, password } => {
            handle_create_client(&pool, name, redirect_uris, personal, password).await
        },
        PassportCommands::ListClients => handle_list_clients(&pool).await,
        PassportCommands::RevokeClient { client_id } => handle_revoke_client(&pool, client_id).await,
        PassportCommands::DeleteClient { client_id } => handle_delete_client(&pool, client_id).await,
        PassportCommands::RegenerateSecret { client_id } => handle_regenerate_secret(&pool, client_id).await,
        PassportCommands::CreateScope { name, description, default } => {
            handle_create_scope(&pool, name, description, default).await
        },
        PassportCommands::ListScopes => handle_list_scopes(&pool).await,
        PassportCommands::DeleteScope { scope } => handle_delete_scope(&pool, scope).await,
        PassportCommands::ListTokens { user_id } => handle_list_tokens(&pool, user_id).await,
        PassportCommands::RevokeToken { token_id } => handle_revoke_token(&pool, token_id).await,
        PassportCommands::RevokeAllUserTokens { user_id } => handle_revoke_all_user_tokens(&pool, user_id).await,
    }
}

async fn handle_install(pool: &DbPool) -> Result<()> {
    use diesel::prelude::*;
    use crate::schema::{oauth_clients, oauth_scopes};

    println!("📦 Installing OAuth2/Passport...");

    // Check if tables already exist by trying to count records
    let mut conn = pool.get()?;
    let client_count = oauth_clients::table.count().get_result::<i64>(&mut conn);

    match client_count {
        Ok(_) => {
            println!("✅ OAuth2 tables already exist!");
        },
        Err(_) => {
            println!("❌ OAuth2 tables do not exist. Please run migrations first:");
            println!("   cargo run --bin artisan migrate");
            return Ok(());
        }
    }

    // Create default personal access client if none exists
    match ClientService::find_personal_access_client(pool)? {
        Some(_) => {
            println!("✅ Personal access client already exists!");
        },
        None => {
            let create_client = CreateClient {
                organization_id: None,
                user_id: None,
                name: "Personal Access Client".to_string(),
                redirect_uris: vec!["http://localhost".to_string()],
                personal_access_client: true,
                password_client: false,
            };

            let client = ClientService::create_client(pool, create_client, None).await?;
            println!("✅ Created personal access client: {}", client.id);
        }
    }

    // Check if default scopes exist
    let scope_count = oauth_scopes::table.count().get_result::<i64>(&mut conn)?;

    if scope_count == 0 {
        println!("⚠️  No scopes found. Default scopes should be created via migrations.");
        println!("   Please ensure migration 008_create_oauth_scopes_table.sql has been run.");
    } else {
        println!("✅ Found {} existing scopes", scope_count);
    }

    println!("\n🎉 OAuth2/Passport installation complete!");
    println!("\nYou can now:");
    println!("  • Create OAuth2 clients with: cargo run --bin artisan passport client --name 'My App' --redirect-uris 'http://localhost:3000/callback'");
    println!("  • Create personal access tokens programmatically");
    println!("  • Use OAuth2 authorization code flow");
    println!("  • Protect routes with scope-based middleware");

    Ok(())
}

async fn handle_create_client(pool: &DbPool, name: String, redirect_uris: String, personal: bool, password: bool) -> Result<()> {
    println!("🔧 Creating OAuth2 client...");

    let redirect_uri_list: Vec<String> = redirect_uris
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let create_client = CreateClient {
        organization_id: None,
        user_id: None, // System client
        name: name.clone(),
        redirect_uris: redirect_uri_list,
        personal_access_client: personal,
        password_client: password,
    };

    let client = ClientService::create_client(pool, create_client, None).await?;

    println!("✅ Client created successfully!");
    println!("   Client ID: {}", client.id);
    println!("   Client Name: {}", client.name);

    if let Some(secret) = &client.secret {
        println!("   Client Secret: {}", secret);
        println!("   ⚠️  Store the secret securely - it won't be shown again!");
    }

    println!("   Personal Access Client: {}", client.personal_access_client);
    println!("   Password Client: {}", client.password_client);
    println!("   Redirect URIs: {}", client.redirect_uris.join(", "));

    Ok(())
}

async fn handle_list_clients(pool: &DbPool) -> Result<()> {
    println!("📋 Listing OAuth2 clients...");

    let clients = ClientService::list_clients(pool, None)?;

    if clients.is_empty() {
        println!("No OAuth2 clients found.");
        return Ok(());
    }

    println!("\n{:<26} {:<30} {:<8} {:<8} {:<8}", "ID", "Name", "Personal", "Password", "Revoked");
    println!("{}", "-".repeat(90));

    for client in clients {
        println!(
            "{:<26} {:<30} {:<8} {:<8} {:<8}",
            client.id,
            if client.name.len() > 28 { format!("{}...", &client.name[..25]) } else { client.name },
            client.personal_access_client,
            client.password_client,
            client.revoked
        );
    }

    Ok(())
}

async fn handle_revoke_client(pool: &DbPool, client_id: String) -> Result<()> {
    println!("🔒 Revoking OAuth2 client...");

    ClientService::revoke_client(pool, client_id.clone())?;

    println!("✅ Client {} has been revoked!", client_id);
    println!("   All associated access tokens have also been revoked.");

    Ok(())
}

async fn handle_delete_client(pool: &DbPool, client_id: String) -> Result<()> {
    println!("🗑️  Deleting OAuth2 client...");

    // Check if client exists first
    let client = ClientService::find_by_id(pool, client_id.clone())?;
    match client {
        Some(client) => {
            println!("   Found client: {}", client.name);
        },
        None => {
            println!("❌ Client not found: {}", client_id);
            return Ok(());
        }
    }

    ClientService::delete_client(pool, client_id.clone())?;

    println!("✅ Client {} has been deleted!", client_id);
    println!("   ⚠️  This action cannot be undone!");

    Ok(())
}

async fn handle_regenerate_secret(pool: &DbPool, client_id: String) -> Result<()> {
    println!("🔄 Regenerating client secret...");

    let new_secret = ClientService::regenerate_secret(pool, client_id.clone())?;

    println!("✅ New client secret generated!");
    println!("   Client ID: {}", client_id);
    println!("   New Secret: {}", new_secret);
    println!("   ⚠️  Store the secret securely - it won't be shown again!");
    println!("   ⚠️  Update your application configuration with the new secret!");

    Ok(())
}

async fn handle_create_scope(pool: &DbPool, name: String, description: Option<String>, default: bool) -> Result<()> {
    println!("🏷️  Creating OAuth2 scope...");

    let create_scope = CreateScope {
        name: name.clone(),
        description,
        is_default: default,
    };

    let scope = ScopeService::create_scope(pool, create_scope)?;

    println!("✅ Scope created successfully!");
    println!("   Name: {}", scope.name);

    if let Some(desc) = &scope.description {
        println!("   Description: {}", desc);
    }

    println!("   Default: {}", scope.is_default);

    Ok(())
}

async fn handle_list_scopes(pool: &DbPool) -> Result<()> {
    println!("📋 Listing OAuth2 scopes...");

    let scopes = ScopeService::list_scopes(pool)?;

    if scopes.is_empty() {
        println!("No OAuth2 scopes found.");
        return Ok(());
    }

    println!("\n{:<26} {:<20} {:<50} {:<8}", "ID", "Name", "Description", "Default");
    println!("{}", "-".repeat(110));

    for scope in scopes {
        let description = scope.description
            .as_deref()
            .map(|d| if d.len() > 47 { format!("{}...", &d[..44]) } else { d.to_string() })
            .unwrap_or_else(|| "-".to_string());

        println!(
            "{:<26} {:<20} {:<50} {:<8}",
            scope.id,
            scope.name,
            description,
            scope.is_default
        );
    }

    Ok(())
}

async fn handle_delete_scope(pool: &DbPool, scope: String) -> Result<()> {
    println!("🗑️  Deleting OAuth2 scope...");

    // Try to parse as ULID first, then fallback to name lookup
    let scope_id = match Ulid::from_string(&scope) {
        Ok(id) => id,
        Err(_) => {
            // Look up by name
            match ScopeService::find_by_name(pool, &scope)? {
                Some(found_scope) => found_scope.id.inner(),
                None => {
                    println!("❌ Scope not found: {}", scope);
                    return Ok(());
                }
            }
        }
    };

    ScopeService::delete_scope(pool, scope_id.to_string())?;

    println!("✅ Scope {} has been deleted!", scope);
    println!("   ⚠️  This action cannot be undone!");

    Ok(())
}

async fn handle_list_tokens(pool: &DbPool, user_id: Option<String>) -> Result<()> {
    println!("📋 Listing access tokens...");

    let tokens = match user_id {
        Some(uid) => {
            TokenService::list_user_tokens(pool, uid).await?
        },
        None => {
            // List all tokens (this would need a new method in TokenService)
            println!("❌ Listing all tokens is not implemented yet. Please specify --user-id");
            return Ok(());
        }
    };

    if tokens.is_empty() {
        println!("No access tokens found.");
        return Ok(());
    }

    println!("\n{:<26} {:<26} {:<30} {:<8} {:<20}", "Token ID", "User ID", "Name", "Revoked", "Expires At");
    println!("{}", "-".repeat(120));

    for token in tokens {
        let token_response = token.to_response();
        let expires_str = token_response.expires_at
            .map(|exp| exp.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "Never".to_string());

        let name = token_response.name
            .as_deref()
            .unwrap_or("-");

        println!(
            "{:<26} {:<26} {:<30} {:<8} {:<20}",
            token_response.id,
            token_response.user_id.as_deref().unwrap_or("-"),
            if name.len() > 28 { format!("{}...", &name[..25]) } else { name.to_string() },
            token_response.revoked,
            expires_str
        );
    }

    Ok(())
}

async fn handle_revoke_token(pool: &DbPool, token_id: String) -> Result<()> {
    println!("🔒 Revoking access token...");

    TokenService::revoke_access_token(pool, token_id.clone())?;

    println!("✅ Token {} has been revoked!", token_id);

    Ok(())
}

async fn handle_revoke_all_user_tokens(pool: &DbPool, user_id: String) -> Result<()> {
    println!("🔒 Revoking all tokens for user...");

    TokenService::revoke_all_user_tokens(pool, user_id.clone())?;

    println!("✅ All tokens for user {} have been revoked!", user_id);

    Ok(())
}
