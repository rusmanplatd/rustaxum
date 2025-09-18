use anyhow::Result;
use sqlx::PgPool;
use ulid::Ulid;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

use crate::app::models::oauth::{
    Client, CreateClient, UpdateClient, ClientResponse,
    PersonalAccessClient
};
use crate::query_builder::QueryBuilder;

pub struct ClientService;

impl ClientService {
    pub async fn create_client(pool: &PgPool, data: CreateClient) -> Result<ClientResponse> {
        let redirect_uris = data.redirect_uris.join(",");

        // Generate client secret if needed (not for personal access clients)
        let secret = if data.personal_access_client {
            None
        } else {
            Some(Self::generate_client_secret())
        };

        let client = Client::new(
            data.user_id,
            data.name,
            secret,
            redirect_uris,
            data.personal_access_client,
            data.password_client,
        );

        let created_client = Self::create_client_record(pool, client).await?;

        // If this is a personal access client, create the personal access client record
        if data.personal_access_client {
            let pac = PersonalAccessClient::new(created_client.id);
            Self::create_personal_access_client_record(pool, pac).await?;
        }

        Ok(created_client.to_response())
    }

    pub async fn create_client_record(pool: &PgPool, client: Client) -> Result<Client> {
        sqlx::query(
            r#"
            INSERT INTO oauth_clients (
                id, user_id, name, secret, provider, redirect_uris,
                personal_access_client, password_client, revoked,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#
        )
        .bind(client.id.to_string())
        .bind(client.user_id.map(|id| id.to_string()))
        .bind(&client.name)
        .bind(&client.secret)
        .bind(&client.provider)
        .bind(&client.redirect_uris)
        .bind(client.personal_access_client)
        .bind(client.password_client)
        .bind(client.revoked)
        .bind(client.created_at)
        .bind(client.updated_at)
        .execute(pool)
        .await?;

        Ok(client)
    }

    pub async fn create_personal_access_client_record(
        pool: &PgPool,
        pac: PersonalAccessClient
    ) -> Result<PersonalAccessClient> {
        sqlx::query(
            r#"
            INSERT INTO oauth_personal_access_clients (
                id, client_id, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4)
            "#
        )
        .bind(pac.id.to_string())
        .bind(pac.client_id.to_string())
        .bind(pac.created_at)
        .bind(pac.updated_at)
        .execute(pool)
        .await?;

        Ok(pac)
    }

    pub async fn find_by_id(pool: &PgPool, id: Ulid) -> Result<Option<Client>> {
        let row = sqlx::query_as::<_, Client>(
            "SELECT id, user_id, name, secret, provider, redirect_uris, personal_access_client, password_client, revoked, created_at, updated_at FROM oauth_clients WHERE id = $1 AND revoked = false"
        )
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    pub async fn find_by_id_and_secret(pool: &PgPool, id: Ulid, secret: &str) -> Result<Option<Client>> {
        let row = sqlx::query_as::<_, Client>(
            "SELECT id, user_id, name, secret, provider, redirect_uris, personal_access_client, password_client, revoked, created_at, updated_at FROM oauth_clients WHERE id = $1 AND secret = $2 AND revoked = false"
        )
        .bind(id.to_string())
        .bind(secret)
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    pub async fn find_personal_access_client(pool: &PgPool) -> Result<Option<Client>> {
        let row = sqlx::query_as::<_, Client>(
            r#"
            SELECT c.id, c.user_id, c.name, c.secret, c.provider, c.redirect_uris, c.personal_access_client, c.password_client, c.revoked, c.created_at, c.updated_at
            FROM oauth_clients c
            INNER JOIN oauth_personal_access_clients pac ON pac.client_id = c.id
            WHERE c.revoked = false
            ORDER BY c.created_at ASC
            LIMIT 1
            "#
        )
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    pub async fn find_password_client(pool: &PgPool) -> Result<Option<Client>> {
        let row = sqlx::query_as::<_, Client>(
            "SELECT id, user_id, name, secret, provider, redirect_uris, personal_access_client, password_client, revoked, created_at, updated_at FROM oauth_clients WHERE password_client = true AND revoked = false ORDER BY created_at ASC LIMIT 1"
        )
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    pub async fn list_clients(pool: &PgPool, user_id: Option<Ulid>) -> Result<Vec<ClientResponse>> {
        let mut request = crate::query_builder::QueryBuilderRequest::default();

        if let Some(user_id) = user_id {
            request.filters.insert("user_id".to_string(), user_id.to_string());
        }

        request.filters.insert("revoked".to_string(), "false".to_string());

        let query_builder = QueryBuilder::<Client>::new(pool.clone(), request);
        let clients = query_builder.get().await?;
        Ok(clients.into_iter().map(|c| c.to_response_without_secret()).collect())
    }

    pub async fn update_client(pool: &PgPool, id: Ulid, data: UpdateClient) -> Result<ClientResponse> {
        let mut client = Self::find_by_id(pool, id).await?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        if let Some(name) = data.name {
            client.name = name;
        }

        if let Some(redirect_uris) = data.redirect_uris {
            client.redirect_uris = redirect_uris.join(",");
        }

        if let Some(revoked) = data.revoked {
            client.revoked = revoked;
        }

        client.updated_at = chrono::Utc::now();

        sqlx::query(
            r#"
            UPDATE oauth_clients
            SET name = $1, redirect_uris = $2, revoked = $3, updated_at = $4
            WHERE id = $5
            "#
        )
        .bind(&client.name)
        .bind(&client.redirect_uris)
        .bind(client.revoked)
        .bind(client.updated_at)
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Ok(client.to_response())
    }

    pub async fn revoke_client(pool: &PgPool, id: Ulid) -> Result<()> {
        sqlx::query(
            "UPDATE oauth_clients SET revoked = true, updated_at = NOW() WHERE id = $1"
        )
        .bind(id.to_string())
        .execute(pool)
        .await?;

        // Also revoke all access tokens for this client
        sqlx::query(
            "UPDATE oauth_access_tokens SET revoked = true, updated_at = NOW() WHERE client_id = $1"
        )
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn delete_client(pool: &PgPool, id: Ulid) -> Result<()> {
        // Delete personal access client record first if it exists
        sqlx::query(
            "DELETE FROM oauth_personal_access_clients WHERE client_id = $1"
        )
        .bind(id.to_string())
        .execute(pool)
        .await?;

        // Delete the client (this will cascade to tokens due to foreign key constraints)
        sqlx::query(
            "DELETE FROM oauth_clients WHERE id = $1"
        )
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn regenerate_secret(pool: &PgPool, id: Ulid) -> Result<String> {
        let client = Self::find_by_id(pool, id).await?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        if client.personal_access_client {
            return Err(anyhow::anyhow!("Personal access clients do not have secrets"));
        }

        let new_secret = Self::generate_client_secret();

        sqlx::query(
            "UPDATE oauth_clients SET secret = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(&new_secret)
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Ok(new_secret)
    }

    fn generate_client_secret() -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(40)
            .map(char::from)
            .collect()
    }

    pub async fn is_valid_redirect_uri(pool: &PgPool, client_id: Ulid, redirect_uri: &str) -> Result<bool> {
        if let Some(client) = Self::find_by_id(pool, client_id).await? {
            Ok(client.is_valid_redirect_uri(redirect_uri))
        } else {
            Ok(false)
        }
    }
}

