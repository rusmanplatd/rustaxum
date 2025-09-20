use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow, PgPool};
use ulid::Ulid;
use chrono::{DateTime, Utc};
use anyhow::Result;
use std::collections::HashMap;
use serde_json::{json, Value};
use utoipa::ToSchema;
use crate::query_builder::{Queryable, SortDirection};

/// User organization model representing the relationship between users and organizations
/// Contains employment information, job position, and temporal data
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserOrganization {
    /// Unique identifier for the user-organization relationship
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub id: Ulid,
    /// ID of the user in this relationship
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub user_id: Ulid,
    /// ID of the organization in this relationship
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_id: Ulid,
    /// ID of the job position held by the user
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub job_position_id: Ulid,
    /// Whether this employment relationship is currently active
    #[schema(example = true)]
    pub is_active: bool,
    /// When the employment started
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub started_at: DateTime<Utc>,
    /// When the employment ended (if applicable)
    #[schema(example = "2024-01-01T00:00:00Z")]
    pub ended_at: Option<DateTime<Utc>>,
    /// Creation timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    #[schema(example = "2023-01-01T00:00:00Z")]
    pub updated_at: DateTime<Utc>,
}

/// Create user organization payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateUserOrganization {
    pub user_id: String,
    pub organization_id: String,
    pub job_position_id: String,
    pub started_at: Option<DateTime<Utc>>,
}

/// Update user organization payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateUserOrganization {
    pub organization_id: Option<String>,
    pub job_position_id: Option<String>,
    pub is_active: Option<bool>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
}

/// User organization response payload for API endpoints
#[derive(Debug, Serialize, ToSchema)]
pub struct UserOrganizationResponse {
    pub id: String,
    pub user_id: String,
    pub organization_id: String,
    pub job_position_id: String,
    pub is_active: bool,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserOrganization {
    pub fn new(user_id: Ulid, organization_id: Ulid, job_position_id: Ulid, started_at: Option<DateTime<Utc>>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            user_id,
            organization_id,
            job_position_id,
            is_active: true,
            started_at: started_at.unwrap_or(now),
            ended_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn to_response(&self) -> UserOrganizationResponse {
        UserOrganizationResponse {
            id: self.id.to_string(),
            user_id: self.user_id.to_string(),
            organization_id: self.organization_id.to_string(),
            job_position_id: self.job_position_id.to_string(),
            is_active: self.is_active,
            started_at: self.started_at,
            ended_at: self.ended_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    // RBAC Methods for UserOrganization

    /// Check if user has a specific role in an organization
    pub async fn user_has_role_in_organization(
        pool: &PgPool,
        user_id: Ulid,
        organization_id: Ulid,
        role_name: &str,
    ) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM roles r
            INNER JOIN user_organization_roles uor ON r.id = uor.role_id
            INNER JOIN user_organizations uo ON uor.user_organization_id = uo.id
            WHERE uo.user_id = $1 AND uo.organization_id = $2 AND r.name = $3 AND uo.is_active = true
            "#
        )
        .bind(user_id.to_string())
        .bind(organization_id.to_string())
        .bind(role_name)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        Ok(count > 0)
    }

    /// Check if user has a specific permission in an organization
    pub async fn user_has_permission_in_organization(
        pool: &PgPool,
        user_id: Ulid,
        organization_id: Ulid,
        permission_name: &str,
    ) -> Result<bool> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM permissions p
            INNER JOIN role_permissions rp ON p.id = rp.permission_id
            INNER JOIN roles r ON rp.role_id = r.id
            INNER JOIN user_organization_roles uor ON r.id = uor.role_id
            INNER JOIN user_organizations uo ON uor.user_organization_id = uo.id
            WHERE uo.user_id = $1 AND uo.organization_id = $2 AND p.name = $3 AND uo.is_active = true
            "#
        )
        .bind(user_id.to_string())
        .bind(organization_id.to_string())
        .bind(permission_name)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        Ok(count > 0)
    }

    /// Assign a role to user in organization
    pub async fn assign_role_to_user_organization(
        pool: &PgPool,
        user_organization_id: Ulid,
        role_id: Ulid,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO user_organization_roles (id, user_organization_id, role_id, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            ON CONFLICT (user_organization_id, role_id) DO NOTHING
            "#
        )
        .bind(Ulid::new().to_string())
        .bind(user_organization_id.to_string())
        .bind(role_id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Remove a role from user in organization
    pub async fn remove_role_from_user_organization(
        pool: &PgPool,
        user_organization_id: Ulid,
        role_id: Ulid,
    ) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM user_organization_roles
            WHERE user_organization_id = $1 AND role_id = $2
            "#
        )
        .bind(user_organization_id.to_string())
        .bind(role_id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    // ABAC Methods for UserOrganization

    /// Get ABAC attributes for this user organization relationship
    pub fn get_abac_attributes(&self) -> HashMap<String, Value> {
        let mut attributes = HashMap::new();

        attributes.insert("user_organization_id".to_string(), json!(self.id.to_string()));
        attributes.insert("user_id".to_string(), json!(self.user_id.to_string()));
        attributes.insert("organization_id".to_string(), json!(self.organization_id.to_string()));
        attributes.insert("job_position_id".to_string(), json!(self.job_position_id.to_string()));
        attributes.insert("is_active".to_string(), json!(self.is_active));
        attributes.insert("started_at".to_string(), json!(self.started_at.to_rfc3339()));

        if let Some(ended_at) = self.ended_at {
            attributes.insert("ended_at".to_string(), json!(ended_at.to_rfc3339()));
        }

        // Add temporal attributes
        let now = Utc::now();
        let duration_days = (now - self.started_at).num_days();
        attributes.insert("tenure_days".to_string(), json!(duration_days));
        attributes.insert("is_current".to_string(), json!(self.ended_at.is_none()));

        // Add derived attributes
        attributes.insert("employment_status".to_string(), json!(
            if self.is_active && self.ended_at.is_none() {
                "active"
            } else if !self.is_active {
                "inactive"
            } else {
                "terminated"
            }
        ));

        attributes
    }

    /// Check if user can access resource based on organization hierarchy
    pub async fn can_access_in_hierarchy(
        &self,
        pool: &PgPool,
        target_organization_id: Ulid,
        access_level: u8, // 1 = same org, 2 = child orgs, 3 = parent orgs
    ) -> Result<bool> {
        match access_level {
            1 => Ok(self.organization_id == target_organization_id),
            2 => {
                // Check if target is a child organization
                let count: i64 = sqlx::query_scalar(
                    r#"
                    WITH RECURSIVE org_hierarchy AS (
                        SELECT id, parent_id FROM organizations WHERE id = $1
                        UNION ALL
                        SELECT o.id, o.parent_id
                        FROM organizations o
                        INNER JOIN org_hierarchy oh ON o.parent_id = oh.id
                    )
                    SELECT COUNT(*) FROM org_hierarchy WHERE id = $2
                    "#
                )
                .bind(self.organization_id.to_string())
                .bind(target_organization_id.to_string())
                .fetch_one(pool)
                .await
                .unwrap_or(0);

                Ok(count > 0)
            }
            3 => {
                // Check if target is a parent organization
                let count: i64 = sqlx::query_scalar(
                    r#"
                    WITH RECURSIVE parent_hierarchy AS (
                        SELECT id, parent_id FROM organizations WHERE id = $1
                        UNION ALL
                        SELECT o.id, o.parent_id
                        FROM organizations o
                        INNER JOIN parent_hierarchy ph ON ph.parent_id = o.id
                    )
                    SELECT COUNT(*) FROM parent_hierarchy WHERE id = $2
                    "#
                )
                .bind(self.organization_id.to_string())
                .bind(target_organization_id.to_string())
                .fetch_one(pool)
                .await
                .unwrap_or(0);

                Ok(count > 0)
            }
            _ => Ok(false),
        }
    }

    /// Activate user organization relationship
    pub async fn activate(&mut self, pool: &PgPool) -> Result<()> {
        self.is_active = true;
        self.updated_at = Utc::now();

        sqlx::query(
            "UPDATE user_organizations SET is_active = true, updated_at = NOW() WHERE id = $1"
        )
        .bind(self.id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Deactivate user organization relationship
    pub async fn deactivate(&mut self, pool: &PgPool) -> Result<()> {
        self.is_active = false;
        self.ended_at = Some(Utc::now());
        self.updated_at = Utc::now();

        sqlx::query(
            "UPDATE user_organizations SET is_active = false, ended_at = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind(self.id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Transfer user to different organization
    pub async fn transfer_to_organization(
        &mut self,
        pool: &PgPool,
        new_organization_id: Ulid,
        new_job_position_id: Ulid,
    ) -> Result<()> {
        // End current relationship
        self.deactivate(pool).await?;

        // Create new relationship
        let new_user_org = UserOrganization::new(
            self.user_id,
            new_organization_id,
            new_job_position_id,
            Some(Utc::now()),
        );

        sqlx::query(
            r#"
            INSERT INTO user_organizations (id, user_id, organization_id, job_position_id, is_active, started_at, ended_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(new_user_org.id.to_string())
        .bind(new_user_org.user_id.to_string())
        .bind(new_user_org.organization_id.to_string())
        .bind(new_user_org.job_position_id.to_string())
        .bind(new_user_org.is_active)
        .bind(new_user_org.started_at)
        .bind(new_user_org.ended_at)
        .bind(new_user_org.created_at)
        .bind(new_user_org.updated_at)
        .execute(pool)
        .await?;

        Ok(())
    }
}

impl FromRow<'_, PgRow> for UserOrganization {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id_str: String = row.try_get("id")?;
        let id = Ulid::from_string(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".to_string(),
            source: Box::new(e),
        })?;

        let user_id_str: String = row.try_get("user_id")?;
        let user_id = Ulid::from_string(&user_id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "user_id".to_string(),
            source: Box::new(e),
        })?;

        let organization_id_str: String = row.try_get("organization_id")?;
        let organization_id = Ulid::from_string(&organization_id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "organization_id".to_string(),
            source: Box::new(e),
        })?;

        let job_position_id_str: String = row.try_get("job_position_id")?;
        let job_position_id = Ulid::from_string(&job_position_id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "job_position_id".to_string(),
            source: Box::new(e),
        })?;

        Ok(UserOrganization {
            id,
            user_id,
            organization_id,
            job_position_id,
            is_active: row.try_get("is_active")?,
            started_at: row.try_get("started_at")?,
            ended_at: row.try_get("ended_at")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl Queryable for UserOrganization {
    fn table_name() -> &'static str {
        "user_organizations"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "organization_id",
            "job_position_id",
            "is_active",
            "started_at",
            "ended_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "organization_id",
            "job_position_id",
            "is_active",
            "started_at",
            "ended_at",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "organization_id",
            "job_position_id",
            "is_active",
            "started_at",
            "ended_at",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, SortDirection)> {
        Some(("created_at", SortDirection::Desc))
    }
}