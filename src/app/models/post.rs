use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: Ulid,
    pub title: String,
    pub content: String,
    pub author_id: Ulid,
    pub published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePost {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub content: Option<String>,
    pub published: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct PostResponse {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
    pub published: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Post {
    pub fn new(title: String, content: String, author_id: Ulid) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            title,
            content,
            author_id,
            published: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> PostResponse {
        PostResponse {
            id: self.id.to_string(),
            title: self.title.clone(),
            content: self.content.clone(),
            author_id: self.author_id.to_string(),
            published: self.published,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl FromRow<'_, PgRow> for Post {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        let author_id_str: String = row.try_get("author_id")?;
        let author_id = Ulid::from_string(&author_id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "author_id".to_string(),
            source: Box::new(e),
        })?;

        Ok(Post {
            id,
            title: row.try_get("title")?,
            content: row.try_get("content")?,
            author_id,
            published: row.try_get("published")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}