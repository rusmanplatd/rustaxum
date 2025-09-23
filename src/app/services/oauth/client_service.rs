use anyhow::Result;
use crate::database::DbPool;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;
use diesel::prelude::*;
use crate::schema::{oauth_clients, oauth_personal_access_clients, user_organizations};

use crate::app::models::oauth::{
    Client, CreateClient, UpdateClient, ClientResponse,
    PersonalAccessClient
};
use crate::app::models::DieselUlid;

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
            data.organization_id,
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
            let pac = PersonalAccessClient::new(created_client.id.clone());
            Self::create_personal_access_client_record(pool, pac)?;
        }

        Ok(created_client.to_response())
    }

    pub fn create_client_record(pool: &DbPool, client: Client) -> Result<Client> {
        let mut conn = pool.get()?;

        diesel::insert_into(oauth_clients::table)
            .values((
                oauth_clients::id.eq(client.id.clone()),
                oauth_clients::organization_id.eq(client.organization_id.clone()),
                oauth_clients::user_id.eq(client.user_id.clone()),
                oauth_clients::name.eq(&client.name),
                oauth_clients::secret.eq(&client.secret),
                oauth_clients::provider.eq(&client.provider),
                oauth_clients::redirect_uris.eq(&client.redirect_uris),
                oauth_clients::personal_access_client.eq(client.personal_access_client),
                oauth_clients::password_client.eq(client.password_client),
                oauth_clients::revoked.eq(client.revoked),
                oauth_clients::created_at.eq(client.created_at),
                oauth_clients::updated_at.eq(client.updated_at),
                oauth_clients::deleted_at.eq(client.deleted_at),
                oauth_clients::created_by.eq(client.created_by.clone()),
                oauth_clients::updated_by.eq(client.updated_by.clone()),
                oauth_clients::deleted_by.eq(client.deleted_by.clone()),
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
                oauth_personal_access_clients::id.eq(pac.id.clone()),
                oauth_personal_access_clients::client_id.eq(pac.client_id.clone()),
                oauth_personal_access_clients::created_at.eq(pac.created_at),
                oauth_personal_access_clients::updated_at.eq(pac.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(pac)
    }

    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<Client>> {
        let mut conn = pool.get()?;

        let row = oauth_clients::table
            .filter(oauth_clients::id.eq(id))
            .filter(oauth_clients::revoked.eq(false))
            .select(Client::as_select())
            .first::<Client>(&mut conn)
            .optional()?;

        Ok(row)
    }

    pub fn find_by_id_and_secret(pool: &DbPool, id: String, secret: &str) -> Result<Option<Client>> {
        let mut conn = pool.get()?;

        let row = oauth_clients::table
            .filter(oauth_clients::id.eq(id))
            .filter(oauth_clients::secret.eq(secret))
            .filter(oauth_clients::revoked.eq(false))
            .select(Client::as_select())
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
            .select(Client::as_select())
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
            .select(Client::as_select())
            .first::<Client>(&mut conn)
            .optional()?;

        Ok(row)
    }

    pub fn list_clients(pool: &DbPool, user_id: Option<String>) -> Result<Vec<ClientResponse>> {
        let mut conn = pool.get()?;

        let mut query = oauth_clients::table.into_boxed();

        if let Some(user_id) = user_id {
            query = query.filter(oauth_clients::user_id.eq(user_id));
        }

        query = query.filter(oauth_clients::revoked.eq(false));

        let clients = query
            .select(Client::as_select())
            .load::<Client>(&mut conn)?;

        Ok(clients.into_iter().map(|c| c.to_response_without_secret()).collect())
    }

    pub fn update_client(pool: &DbPool, id: String, data: UpdateClient) -> Result<ClientResponse> {
        let mut client = Self::find_by_id(pool, id.clone())?
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

        diesel::update(oauth_clients::table.filter(oauth_clients::id.eq(id)))
            .set((
                oauth_clients::name.eq(&client.name),
                oauth_clients::redirect_uris.eq(&client.redirect_uris),
                oauth_clients::revoked.eq(client.revoked),
                oauth_clients::updated_at.eq(client.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(client.to_response())
    }

    pub fn revoke_client(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;
        let now = chrono::Utc::now();

        diesel::update(oauth_clients::table.filter(oauth_clients::id.eq(&id)))
            .set((
                oauth_clients::revoked.eq(true),
                oauth_clients::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        // Also revoke all access tokens for this client
        use crate::schema::oauth_access_tokens;
        diesel::update(oauth_access_tokens::table.filter(oauth_access_tokens::client_id.eq(&id)))
            .set((
                oauth_access_tokens::revoked.eq(true),
                oauth_access_tokens::updated_at.eq(now),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn delete_client(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;

        // Delete personal access client record first if it exists
        diesel::delete(oauth_personal_access_clients::table
            .filter(oauth_personal_access_clients::client_id.eq(id.clone())))
            .execute(&mut conn)?;

        // Delete the client (this will cascade to tokens due to foreign key constraints)
        diesel::delete(oauth_clients::table
            .filter(oauth_clients::id.eq(id)))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn regenerate_secret(pool: &DbPool, id: String) -> Result<String> {
        let client = Self::find_by_id(pool, id.clone())?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        if client.personal_access_client {
            return Err(anyhow::anyhow!("Personal access clients do not have secrets"));
        }

        let new_secret = Self::generate_client_secret();

        let mut conn = pool.get()?;

        diesel::update(oauth_clients::table.filter(oauth_clients::id.eq(id)))
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

    pub fn is_valid_redirect_uri(pool: &DbPool, client_id: String, redirect_uri: &str) -> Result<bool> {
        if let Some(client) = Self::find_by_id(pool, client_id)? {
            Ok(client.is_valid_redirect_uri(redirect_uri))
        } else {
            Ok(false)
        }
    }

    /// Validates if a user belongs to the OAuth client's organization
    /// Returns true if the client has no organization_id (global client)
    /// or if the user is an active member of the client's organization
    pub fn validate_user_organization_access(pool: &DbPool, client_id: String, user_id: DieselUlid) -> Result<bool> {
        let client = Self::find_by_id(pool, client_id)?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        // If client has no organization_id, allow access (global client)
        if client.organization_id.is_none() {
            return Ok(true);
        }

        let organization_id = client.organization_id.unwrap();

        // Check if user is an active member of the client's organization
        let mut conn = pool.get()?;

        let count = user_organizations::table
            .filter(user_organizations::user_id.eq(user_id))
            .filter(user_organizations::organization_id.eq(organization_id))
            .filter(user_organizations::is_active.eq(true))
            .filter(user_organizations::deleted_at.is_null())
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(count > 0)
    }

    /// Find client by ID and validate user's organization access
    pub fn find_by_id_with_user_validation(pool: &DbPool, client_id: String, user_id: DieselUlid) -> Result<Option<Client>> {
        let client = Self::find_by_id(pool, client_id.clone())?;

        if let Some(ref _client) = client {
            if !Self::validate_user_organization_access(pool, client_id, user_id)? {
                return Ok(None); // User doesn't have access to this client
            }
        }

        Ok(client)
    }

    /// Find client by ID and secret with user organization validation
    pub fn find_by_id_and_secret_with_user_validation(
        pool: &DbPool,
        client_id: String,
        secret: &str,
        user_id: DieselUlid
    ) -> Result<Option<Client>> {
        let client = Self::find_by_id_and_secret(pool, client_id.clone(), secret)?;

        if let Some(ref _client) = client {
            if !Self::validate_user_organization_access(pool, client_id, user_id)? {
                return Ok(None); // User doesn't have access to this client
            }
        }

        Ok(client)
    }
}

