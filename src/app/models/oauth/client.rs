use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};
use crate::query_builder::{Queryable, SortDirection};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: Ulid,
    pub user_id: Option<Ulid>,
    pub name: String,
    pub secret: Option<String>,
    pub provider: Option<String>,
    pub redirect_uris: String,
    pub personal_access_client: bool,
    pub password_client: bool,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateClient {
    pub user_id: Option<Ulid>,
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub personal_access_client: bool,
    pub password_client: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateClient {
    pub name: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    pub revoked: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ClientResponse {
    pub id: String,
    pub name: String,
    pub secret: Option<String>,
    pub redirect_uris: Vec<String>,
    pub personal_access_client: bool,
    pub password_client: bool,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Client {
    pub fn new(
        user_id: Option<Ulid>,
        name: String,
        secret: Option<String>,
        redirect_uris: String,
        personal_access_client: bool,
        password_client: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            user_id,
            name,
            secret,
            provider: None,
            redirect_uris,
            personal_access_client,
            password_client,
            revoked: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> ClientResponse {
        ClientResponse {
            id: self.id.to_string(),
            name: self.name.clone(),
            secret: self.secret.clone(),
            redirect_uris: self.redirect_uris
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            personal_access_client: self.personal_access_client,
            password_client: self.password_client,
            revoked: self.revoked,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    pub fn to_response_without_secret(&self) -> ClientResponse {
        let mut response = self.to_response();
        response.secret = None;
        response
    }

    pub fn get_redirect_uris(&self) -> Vec<String> {
        self.redirect_uris
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    }

    pub fn is_valid_redirect_uri(&self, uri: &str) -> bool {
        self.get_redirect_uris().contains(&uri.to_string())
    }

    pub fn has_secret(&self) -> bool {
        self.secret.is_some()
    }

    pub fn verify_secret(&self, secret: &str) -> bool {
        match &self.secret {
            Some(client_secret) => client_secret == secret,
            None => false,
        }
    }
}

impl FromRow<'_, PgRow> for Client {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        let user_id = match row.try_get::<Option<String>, _>("user_id")? {
            Some(user_id_str) => Some(Ulid::from_string(&user_id_str).map_err(|e| sqlx::Error::ColumnDecode {
                index: "user_id".to_string(),
                source: Box::new(e),
            })?),
            None => None,
        };

        Ok(Client {
            id,
            user_id,
            name: row.try_get("name")?,
            secret: row.try_get("secret")?,
            provider: row.try_get("provider")?,
            redirect_uris: row.try_get("redirect_uris")?,
            personal_access_client: row.try_get("personal_access_client")?,
            password_client: row.try_get("password_client")?,
            revoked: row.try_get("revoked")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl Queryable for Client {
    fn table_name() -> &'static str {
        "oauth_clients"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "name",
            "personal_access_client",
            "password_client",
            "revoked",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "name",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "name",
            "redirect_uris",
            "personal_access_client",
            "password_client",
            "revoked",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }
}