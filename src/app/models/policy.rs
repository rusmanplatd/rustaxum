use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};
use ulid::Ulid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyEffect {
    Permit,
    Deny,
}

impl PolicyEffect {
    pub fn as_str(&self) -> &'static str {
        match self {
            PolicyEffect::Permit => "permit",
            PolicyEffect::Deny => "deny",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "permit" => Some(PolicyEffect::Permit),
            "deny" => Some(PolicyEffect::Deny),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: Ulid,
    pub name: String,
    pub description: Option<String>,
    pub effect: PolicyEffect,
    pub target: String,
    pub condition: Option<String>,
    pub priority: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePolicy {
    pub name: String,
    pub description: Option<String>,
    pub effect: PolicyEffect,
    pub target: String,
    pub condition: Option<String>,
    pub priority: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePolicy {
    pub name: Option<String>,
    pub description: Option<String>,
    pub effect: Option<PolicyEffect>,
    pub target: Option<String>,
    pub condition: Option<String>,
    pub priority: Option<i32>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct PolicyResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub effect: PolicyEffect,
    pub target: String,
    pub condition: Option<String>,
    pub priority: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Policy {
    pub fn new(
        name: String,
        description: Option<String>,
        effect: PolicyEffect,
        target: String,
        condition: Option<String>,
        priority: Option<i32>,
        is_active: Option<bool>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            name,
            description,
            effect,
            target,
            condition,
            priority: priority.unwrap_or(0),
            is_active: is_active.unwrap_or(true),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> PolicyResponse {
        PolicyResponse {
            id: self.id.to_string(),
            name: self.name.clone(),
            description: self.description.clone(),
            effect: self.effect.clone(),
            target: self.target.clone(),
            condition: self.condition.clone(),
            priority: self.priority,
            is_active: self.is_active,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

impl FromRow<'_, PgRow> for Policy {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        let effect_str: String = row.try_get("effect")?;
        let effect = PolicyEffect::from_str(&effect_str)
            .ok_or_else(|| sqlx::Error::ColumnDecode {
                index: "effect".to_string(),
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid policy effect",
                )),
            })?;

        Ok(Policy {
            id,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            effect,
            target: row.try_get("target")?,
            condition: row.try_get("condition")?,
            priority: row.try_get("priority")?,
            is_active: row.try_get("is_active")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
