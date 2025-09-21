use anyhow::Result;
use crate::database::DbPool;
use ulid::Ulid;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use diesel::prelude::*;
use crate::schema::{oauth_clients, oauth_personal_access_clients};

use crate::app::models::oauth::{
    Client, CreateClient, UpdateClient, ClientResponse,
    PersonalAccessClient
};
use crate::app::query_builder::QueryBuilder;

pub struct ClientService;

impl ClientService {
    pub fn create_client(pool: &DbPool, data: CreateClient) -> Result<ClientResponse> {
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

        let created_client = Self::create_client_record(pool, client)?;

        // If this is a personal access client, create the personal access client record
        if data.personal_access_client {
            let pac = PersonalAccessClient::new(created_client.id);
            Self::create_personal_access_client_record(pool, pac)?;
        }

        Ok(created_client.to_response())
    }

    pub fn create_client_record(pool: &DbPool, client: Client) -> Result<Client> {
        let mut conn = pool.get()?;

        diesel::insert_into(oauth_clients::table)
            .values((
                oauth_clients::id.eq(client.id.to_string()),
                oauth_clients::user_id.eq(client.user_id.map(|id| id.to_string())),
                oauth_clients::name.eq(&client.name),
                oauth_clients::secret.eq(&client.secret),
                oauth_clients::provider.eq(&client.provider),
                oauth_clients::redirect_uris.eq(&client.redirect_uris),
                oauth_clients::personal_access_client.eq(client.personal_access_client),
                oauth_clients::password_client.eq(client.password_client),
                oauth_clients::revoked.eq(client.revoked),
                oauth_clients::created_at.eq(client.created_at),
                oauth_clients::updated_at.eq(client.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(client)
    }

    pub fn create_personal_access_client_record(
        pool: &DbPool,
        pac: PersonalAccessClient
    ) -> Result<PersonalAccessClient> {
        let mut conn = pool.get()?;

        diesel::insert_into(oauth_personal_access_clients::table)
            .values((
                oauth_personal_access_clients::id.eq(pac.id.to_string()),
                oauth_personal_access_clients::client_id.eq(pac.client_id.to_string()),
                oauth_personal_access_clients::created_at.eq(pac.created_at),
                oauth_personal_access_clients::updated_at.eq(pac.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(pac)
    }

    pub fn find_by_id(pool: &DbPool, id: Ulid) -> Result<Option<Client>> {
        let mut conn = pool.get()?;

        let row = oauth_clients::table
            .filter(oauth_clients::id.eq(id.to_string()))
            .filter(oauth_clients::revoked.eq(false))
            .first::<Client>(&mut conn)
            .optional()?;

        Ok(row)
    }

    pub fn find_by_id_and_secret(pool: &DbPool, id: Ulid, secret: &str) -> Result<Option<Client>> {
        let mut conn = pool.get()?;

        let row = oauth_clients::table
            .filter(oauth_clients::id.eq(id.to_string()))
            .filter(oauth_clients::secret.eq(secret))
            .filter(oauth_clients::revoked.eq(false))
            .first::<Client>(&mut conn)
            .optional()?;

        Ok(row)
    }

    pub fn find_personal_access_client(pool: &DbPool) -> Result<Option<Client>> {
        let mut conn = pool.get()?;

        let row = oauth_clients::table
            .inner_join(oauth_personal_access_clients::table)
            .filter(oauth_clients::revoked.eq(false))
            .order(oauth_clients::created_at.asc())
            .select(oauth_clients::all_columns)
            .first::<Client>(&mut conn)
            .optional()?;

        Ok(row)
    }

    pub fn find_password_client(pool: &DbPool) -> Result<Option<Client>> {
        let mut conn = pool.get()?;

        let row = oauth_clients::table
            .filter(oauth_clients::password_client.eq(true))
            .filter(oauth_clients::revoked.eq(false))
            .order(oauth_clients::created_at.asc())
            .first::<Client>(&mut conn)
            .optional()?;

        Ok(row)
    }

    pub fn list_clients(pool: &DbPool, user_id: Option<Ulid>) -> Result<Vec<ClientResponse>> {
        let mut request = crate::app::query_builder::QueryParams::default();

        if let Some(user_id) = user_id {
            request.filter.insert("user_id".to_string(), serde_json::Value::String(user_id.to_string()));
        }

        request.filter.insert("revoked".to_string(), serde_json::Value::String("false".to_string()));

        let query_builder = QueryBuilder::<Client>::new(pool.clone(), request);
        let clients = query_builder.get()?;
        Ok(clients.into_iter().map(|c| c.to_response_without_secret()).collect())
    }

    pub fn update_client(pool: &DbPool, id: Ulid, data: UpdateClient) -> Result<ClientResponse> {
        let mut client = Self::find_by_id(pool, id)?
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

        let mut conn = pool.get()?;

        diesel::update(oauth_clients::table.filter(oauth_clients::id.eq(id.to_string())))
            .set((
                oauth_clients::name.eq(&client.name),
                oauth_clients::redirect_uris.eq(&client.redirect_uris),
                oauth_clients::revoked.eq(client.revoked),
                oauth_clients::updated_at.eq(client.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(client.to_response())
    }

    pub fn revoke_client(pool: &DbPool, id: Ulid) -> Result<()> {
        let mut conn = pool.get()?;
        let now = chrono::Utc::now();

        diesel::update(oauth_clients::table.filter(oauth_clients::id.eq(id.to_string())))
            .set((
                oauth_clients::revoked.eq(true),
                oauth_clients::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        // Also revoke all access tokens for this client
        use crate::schema::oauth_access_tokens;
        diesel::update(oauth_access_tokens::table.filter(oauth_access_tokens::client_id.eq(id.to_string())))
            .set((
                oauth_access_tokens::revoked.eq(true),
                oauth_access_tokens::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn delete_client(pool: &DbPool, id: Ulid) -> Result<()> {
        let mut conn = pool.get()?;

        // Delete personal access client record first if it exists
        diesel::delete(oauth_personal_access_clients::table
            .filter(oauth_personal_access_clients::client_id.eq(id.to_string())))
            .execute(&mut conn)?;

        // Delete the client (this will cascade to tokens due to foreign key constraints)
        diesel::delete(oauth_clients::table
            .filter(oauth_clients::id.eq(id.to_string())))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn regenerate_secret(pool: &DbPool, id: Ulid) -> Result<String> {
        let client = Self::find_by_id(pool, id)?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        if client.personal_access_client {
            return Err(anyhow::anyhow!("Personal access clients do not have secrets"));
        }

        let new_secret = Self::generate_client_secret();

        let mut conn = pool.get()?;

        diesel::update(oauth_clients::table.filter(oauth_clients::id.eq(id.to_string())))
            .set((
                oauth_clients::secret.eq(&new_secret),
                oauth_clients::updated_at.eq(chrono::Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(new_secret)
    }

    fn generate_client_secret() -> String {
        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(40)
            .map(char::from)
            .collect()
    }

    pub fn is_valid_redirect_uri(pool: &DbPool, client_id: Ulid, redirect_uri: &str) -> Result<bool> {
        if let Some(client) = Self::find_by_id(pool, client_id)? {
            Ok(client.is_valid_redirect_uri(redirect_uri))
        } else {
            Ok(false)
        }
    }
}

