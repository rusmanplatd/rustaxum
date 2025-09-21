use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = crate::schema::sys_roles)]
pub struct Role {
    pub id: Ulid,
    pub name: String,
    pub description: Option<String>,
    pub guard_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRole {
    pub name: String,
    pub description: Option<String>,
    pub guard_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRole {
    pub name: Option<String>,
    pub description: Option<String>,
    pub guard_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub guard_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Role {
    pub fn new(name: String, description: Option<String>, guard_name: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            name,
            description,
            guard_name: guard_name.unwrap_or_else(|| "api".to_string()),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> RoleResponse {
        RoleResponse {
            id: self.id.to_string(),
            name: self.name.clone(),
            description: self.description.clone(),
            guard_name: self.guard_name.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

