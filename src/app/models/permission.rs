use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: Ulid,
    pub name: String,
    pub guard_name: String,
    pub resource: Option<String>,
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePermission {
    pub name: String,
    pub guard_name: Option<String>,
    pub resource: Option<String>,
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePermission {
    pub name: Option<String>,
    pub guard_name: Option<String>,
    pub resource: Option<String>,
    pub action: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PermissionResponse {
    pub id: String,
    pub name: String,
    pub guard_name: String,
    pub resource: Option<String>,
    pub action: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Permission {
    pub fn new(name: String, guard_name: Option<String>, resource: Option<String>, action: String) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            name,
            guard_name: guard_name.unwrap_or_else(|| "api".to_string()),
            resource,
            action,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> PermissionResponse {
        PermissionResponse {
            id: self.id.to_string(),
            name: self.name.clone(),
            guard_name: self.guard_name.clone(),
            resource: self.resource.clone(),
            action: self.action.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
