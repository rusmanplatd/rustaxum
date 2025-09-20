use anyhow::Result;
use ulid::Ulid;
use sqlx::PgPool;
use std::collections::HashMap;

use crate::app::models::organization::{Organization, CreateOrganization, UpdateOrganization};

pub struct OrganizationService;

impl OrganizationService {
    pub async fn create(pool: &PgPool, data: CreateOrganization) -> Result<Organization> {
        let parent_id = if let Some(parent_id_str) = data.parent_id {
            Some(Ulid::from_string(&parent_id_str)?)
        } else {
            None
        };

        let organization = Organization::new(
            data.name,
            data.organization_type,
            parent_id,
            data.code,
            data.description,
        );

        let query = r#"
            INSERT INTO organizations (id, name, organization_type, parent_id, code, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
        "#;

        let result = sqlx::query_as::<_, Organization>(query)
            .bind(organization.id.to_string())
            .bind(&organization.name)
            .bind(&organization.organization_type)
            .bind(organization.parent_id.map(|id| id.to_string()))
            .bind(&organization.code)
            .bind(&organization.description)
            .bind(organization.is_active)
            .bind(organization.created_at)
            .bind(organization.updated_at)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn find_by_id(pool: &PgPool, id: Ulid) -> Result<Option<Organization>> {
        let query = "SELECT * FROM organizations WHERE id = $1";

        let result = sqlx::query_as::<_, Organization>(query)
            .bind(id.to_string())
            .fetch_optional(pool)
            .await?;

        Ok(result)
    }

    pub async fn find_by_code(pool: &PgPool, code: &str) -> Result<Option<Organization>> {
        let query = "SELECT * FROM organizations WHERE code = $1";

        let result = sqlx::query_as::<_, Organization>(query)
            .bind(code)
            .fetch_optional(pool)
            .await?;

        Ok(result)
    }

    pub async fn list(pool: &PgPool, _query_params: HashMap<String, String>) -> Result<Vec<Organization>> {
        // For now, use a simple query without the query builder to avoid SQL syntax issues
        let query = "SELECT * FROM organizations ORDER BY name ASC";
        let result = sqlx::query_as::<_, Organization>(query)
            .fetch_all(pool)
            .await?;
        Ok(result)
    }

    pub async fn update(pool: &PgPool, id: Ulid, data: UpdateOrganization) -> Result<Organization> {
        let current = Self::find_by_id(pool, id).await?
            .ok_or_else(|| anyhow::anyhow!("Organization not found"))?;

        let parent_id = if let Some(parent_id_str) = data.parent_id {
            Some(Ulid::from_string(&parent_id_str)?)
        } else {
            current.parent_id
        };

        let query = r#"
            UPDATE organizations
            SET name = $2, organization_type = $3, parent_id = $4, code = $5, description = $6, is_active = $7, updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#;

        let result = sqlx::query_as::<_, Organization>(query)
            .bind(id.to_string())
            .bind(data.name.unwrap_or(current.name))
            .bind(data.organization_type.unwrap_or(current.organization_type))
            .bind(parent_id.map(|id| id.to_string()))
            .bind(data.code.or(current.code))
            .bind(data.description.or(current.description))
            .bind(data.is_active.unwrap_or(current.is_active))
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn delete(pool: &PgPool, id: Ulid) -> Result<()> {
        let query = "DELETE FROM organizations WHERE id = $1";

        let result = sqlx::query(query)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Organization not found"));
        }

        Ok(())
    }

    pub async fn find_children(pool: &PgPool, parent_id: Ulid) -> Result<Vec<Organization>> {
        let query = "SELECT * FROM organizations WHERE parent_id = $1 ORDER BY name ASC";

        let results = sqlx::query_as::<_, Organization>(query)
            .bind(parent_id.to_string())
            .fetch_all(pool)
            .await?;

        Ok(results)
    }

    pub async fn find_root_organizations(pool: &PgPool) -> Result<Vec<Organization>> {
        let query = "SELECT * FROM organizations WHERE parent_id IS NULL ORDER BY name ASC";

        let results = sqlx::query_as::<_, Organization>(query)
            .fetch_all(pool)
            .await?;

        Ok(results)
    }
}