use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::database::DbPool;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use anyhow::Result;
use std::collections::HashMap;
use serde_json::{json, Value};
use utoipa::ToSchema;
use crate::app::query_builder::{Queryable, SortDirection};
use crate::app::models::{HasModelType, HasRoles};
use crate::schema::{user_organizations, sys_roles, sys_model_has_roles, sys_permissions, sys_model_has_permissions};

/// User organization model representing the relationship between users and organizations
/// Contains employment information, organization position, and temporal data
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, diesel::Queryable, Identifiable)]
#[diesel(table_name = user_organizations)]
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
    /// ID of the organization position held by the user
    #[schema(example = "01ARZ3NDEKTSV4RRFFQ69G5FAV")]
    pub organization_position_id: Ulid,
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
    pub organization_position_id: String,
    pub started_at: Option<DateTime<Utc>>,
}

/// Update user organization payload for service layer
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateUserOrganization {
    pub organization_id: Option<String>,
    pub organization_position_id: Option<String>,
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
    pub organization_position_id: String,
    pub is_active: bool,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserOrganization {
    pub fn new(user_id: Ulid, organization_id: Ulid, organization_position_id: Ulid, started_at: Option<DateTime<Utc>>) -> Self {
        let now = Utc::now();
        Self {
            id: Ulid::new(),
            user_id,
            organization_id,
            organization_position_id,
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
            organization_position_id: self.organization_position_id.to_string(),
            is_active: self.is_active,
            started_at: self.started_at,
            ended_at: self.ended_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }

    // RBAC Methods for UserOrganization

    /// Check if user has a specific role in an organization
    pub fn user_has_role_in_organization(
        pool: &DbPool,
        user_id: Ulid,
        organization_id: Ulid,
        role_name: &str,
    ) -> Result<bool> {
        let mut conn = pool.get()?;

        let count = user_organizations::table
            .inner_join(sys_model_has_roles::table.on(
                sys_model_has_roles::model_id.eq(user_organizations::id)
                    .and(sys_model_has_roles::model_type.eq("UserOrganization"))
            ))
            .inner_join(sys_roles::table.on(sys_roles::id.eq(sys_model_has_roles::role_id)))
            .filter(user_organizations::user_id.eq(user_id.to_string()))
            .filter(user_organizations::organization_id.eq(organization_id.to_string()))
            .filter(sys_roles::name.eq(role_name))
            .filter(user_organizations::is_active.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .unwrap_or(0);

        Ok(count > 0)
    }

    /// Check if user has a specific permission in an organization
    pub fn user_has_permission_in_organization(
        pool: &DbPool,
        user_id: Ulid,
        organization_id: Ulid,
        permission_name: &str,
    ) -> Result<bool> {
        let mut conn = pool.get()?;

        let count = user_organizations::table
            .inner_join(sys_model_has_roles::table.on(
                sys_model_has_roles::model_id.eq(user_organizations::id)
                    .and(sys_model_has_roles::model_type.eq("UserOrganization"))
            ))
            .inner_join(sys_roles::table.on(sys_roles::id.eq(sys_model_has_roles::role_id)))
            .inner_join(sys_model_has_permissions::table.on(
                sys_model_has_permissions::model_id.eq(sys_roles::id)
                    .and(sys_model_has_permissions::model_type.eq("Role"))
            ))
            .inner_join(sys_permissions::table.on(sys_permissions::id.eq(sys_model_has_permissions::permission_id)))
            .filter(user_organizations::user_id.eq(user_id.to_string()))
            .filter(user_organizations::organization_id.eq(organization_id.to_string()))
            .filter(sys_permissions::name.eq(permission_name))
            .filter(user_organizations::is_active.eq(true))
            .count()
            .get_result::<i64>(&mut conn)
            .unwrap_or(0);

        Ok(count > 0)
    }

    /// Assign a role to user in organization
    pub fn assign_role_to_user_organization(
        pool: &DbPool,
        user_organization_id: Ulid,
        role_id: Ulid,
    ) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::sql_query(
            r#"
            INSERT INTO sys_model_has_roles (id, model_type, model_id, role_id, scope_type, scope_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (model_type, model_id, role_id) DO NOTHING
            "#
        )
        .bind::<diesel::sql_types::Text, _>(Ulid::new().to_string())
        .bind::<diesel::sql_types::Text, _>("UserOrganization")
        .bind::<diesel::sql_types::Text, _>(user_organization_id.to_string())
        .bind::<diesel::sql_types::Text, _>(role_id.to_string())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Option::<String>::None)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(Option::<String>::None)
        .execute(&mut conn)?;

        Ok(())
    }

    /// Remove a role from user in organization
    pub fn remove_role_from_user_organization(
        pool: &DbPool,
        user_organization_id: Ulid,
        role_id: Ulid,
    ) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::sql_query(
            r#"
            DELETE FROM sys_model_has_roles
            WHERE model_type = 'UserOrganization' AND model_id = $1 AND role_id = $2
            "#
        )
        .bind::<diesel::sql_types::Text, _>(user_organization_id.to_string())
        .bind::<diesel::sql_types::Text, _>(role_id.to_string())
        .execute(&mut conn)?;

        Ok(())
    }

    // ABAC Methods for UserOrganization

    /// Get ABAC attributes for this user organization relationship
    pub fn get_abac_attributes(&self) -> HashMap<String, Value> {
        let mut attributes = HashMap::new();

        attributes.insert("user_organization_id".to_string(), json!(self.id.to_string()));
        attributes.insert("user_id".to_string(), json!(self.user_id.to_string()));
        attributes.insert("organization_id".to_string(), json!(self.organization_id.to_string()));
        attributes.insert("organization_position_id".to_string(), json!(self.organization_position_id.to_string()));
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
    pub fn can_access_in_hierarchy(
        &self,
        pool: &DbPool,
        target_organization_id: Ulid,
        access_level: u8, // 1 = same org, 2 = child orgs, 3 = parent orgs
    ) -> Result<bool> {
        match access_level {
            1 => Ok(self.organization_id == target_organization_id),
            2 => {
                let mut conn = pool.get()?;
                // Check if target is a child organization
                let count: i64 = diesel::sql_query(
                    r#"
                    WITH RECURSIVE org_hierarchy AS (
                        SELECT id, parent_id FROM organizations WHERE id = $1
                        UNION ALL
                        SELECT o.id, o.parent_id
                        FROM organizations o
                        INNER JOIN org_hierarchy oh ON o.parent_id = oh.id
                    )
                    SELECT COUNT(*) as count FROM org_hierarchy WHERE id = $2
                    "#
                )
                .bind::<diesel::sql_types::Text, _>(self.organization_id.to_string())
                .bind::<diesel::sql_types::Text, _>(target_organization_id.to_string())
                .get_result::<(i64,)>(&mut conn)
                .map(|(count,)| count)
                .unwrap_or(0);

                Ok(count > 0)
            }
            3 => {
                let mut conn = pool.get()?;
                // Check if target is a parent organization
                let count: i64 = diesel::sql_query(
                    r#"
                    WITH RECURSIVE parent_hierarchy AS (
                        SELECT id, parent_id FROM organizations WHERE id = $1
                        UNION ALL
                        SELECT o.id, o.parent_id
                        FROM organizations o
                        INNER JOIN parent_hierarchy ph ON ph.parent_id = o.id
                    )
                    SELECT COUNT(*) as count FROM parent_hierarchy WHERE id = $2
                    "#
                )
                .bind::<diesel::sql_types::Text, _>(self.organization_id.to_string())
                .bind::<diesel::sql_types::Text, _>(target_organization_id.to_string())
                .get_result::<(i64,)>(&mut conn)
                .map(|(count,)| count)
                .unwrap_or(0);

                Ok(count > 0)
            }
            _ => Ok(false),
        }
    }

    /// Activate user organization relationship
    pub fn activate(&mut self, pool: &DbPool) -> Result<()> {
        let mut conn = pool.get()?;
        self.is_active = true;
        self.updated_at = Utc::now();

        diesel::sql_query(
            "UPDATE user_organizations SET is_active = true, updated_at = NOW() WHERE id = $1"
        )
        .bind::<diesel::sql_types::Text, _>(self.id.to_string())
        .execute(&mut conn)?;

        Ok(())
    }

    /// Deactivate user organization relationship
    pub fn deactivate(&mut self, pool: &DbPool) -> Result<()> {
        let mut conn = pool.get()?;
        self.is_active = false;
        self.ended_at = Some(Utc::now());
        self.updated_at = Utc::now();

        diesel::sql_query(
            "UPDATE user_organizations SET is_active = false, ended_at = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind::<diesel::sql_types::Text, _>(self.id.to_string())
        .execute(&mut conn)?;

        Ok(())
    }

    /// Transfer user to different organization
    pub fn transfer_to_organization(
        &mut self,
        pool: &DbPool,
        new_organization_id: Ulid,
        new_organization_position_id: Ulid,
    ) -> Result<()> {
        // End current relationship
        self.deactivate(pool)?;

        // Create new relationship
        let new_user_org = UserOrganization::new(
            self.user_id,
            new_organization_id,
            new_organization_position_id,
            Some(Utc::now()),
        );

        let mut conn = pool.get()?;
        diesel::sql_query(
            r#"
            INSERT INTO user_organizations (id, user_id, organization_id, organization_position_id, is_active, started_at, ended_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind::<diesel::sql_types::Text, _>(new_user_org.id.to_string())
        .bind::<diesel::sql_types::Text, _>(new_user_org.user_id.to_string())
        .bind::<diesel::sql_types::Text, _>(new_user_org.organization_id.to_string())
        .bind::<diesel::sql_types::Text, _>(new_user_org.organization_position_id.to_string())
        .bind::<diesel::sql_types::Bool, _>(new_user_org.is_active)
        .bind::<diesel::sql_types::Timestamptz, _>(new_user_org.started_at)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(new_user_org.ended_at)
        .bind::<diesel::sql_types::Timestamptz, _>(new_user_org.created_at)
        .bind::<diesel::sql_types::Timestamptz, _>(new_user_org.updated_at)
        .execute(&mut conn)?;

        Ok(())
    }
}

impl crate::app::query_builder::Queryable for UserOrganization {
    fn table_name() -> &'static str {
        "user_organizations"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "user_id",
            "organization_id",
            "organization_position_id",
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
            "organization_position_id",
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
            "organization_position_id",
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

    fn allowed_includes() -> Vec<&'static str> {
        vec![
            "user",
            "organization",
            "position",
        ]
    }
}

// Implement the query builder service for UserOrganization
crate::impl_query_builder_service!(UserOrganization);

impl HasModelType for UserOrganization {
    fn model_type() -> &'static str {
        "UserOrganization"
    }
}

impl HasRoles for UserOrganization {
    fn model_id(&self) -> String {
        self.id.to_string()
    }
}