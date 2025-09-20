use anyhow::Result;
use ulid::Ulid;
use sqlx::PgPool;

use crate::app::models::sys_model_has_permission::{SysModelHasPermission, CreateSysModelHasPermission, UpdateSysModelHasPermission};

pub struct SysModelHasPermissionService;

impl SysModelHasPermissionService {
    pub async fn create(pool: &PgPool, data: CreateSysModelHasPermission) -> Result<SysModelHasPermission> {
        let permission = SysModelHasPermission::new(
            data.model_type,
            data.model_id,
            data.permission_id,
            data.scope_type,
            data.scope_id,
        );

        let query = r#"
            INSERT INTO sys_model_has_permissions (id, model_type, model_id, permission_id, scope_type, scope_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
        "#;

        let result = sqlx::query_as::<_, SysModelHasPermission>(query)
            .bind(permission.id.to_string())
            .bind(&permission.model_type)
            .bind(permission.model_id.to_string())
            .bind(permission.permission_id.to_string())
            .bind(&permission.scope_type)
            .bind(permission.scope_id.map(|id| id.to_string()))
            .bind(permission.created_at)
            .bind(permission.updated_at)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn find_by_id(pool: &PgPool, id: Ulid) -> Result<Option<SysModelHasPermission>> {
        let query = "SELECT * FROM sys_model_has_permissions WHERE id = $1";

        let result = sqlx::query_as::<_, SysModelHasPermission>(query)
            .bind(id.to_string())
            .fetch_optional(pool)
            .await?;

        Ok(result)
    }

    pub async fn find_by_model(pool: &PgPool, model_type: &str, model_id: Ulid) -> Result<Vec<SysModelHasPermission>> {
        let query = "SELECT * FROM sys_model_has_permissions WHERE model_type = $1 AND model_id = $2 ORDER BY created_at DESC";

        let result = sqlx::query_as::<_, SysModelHasPermission>(query)
            .bind(model_type)
            .bind(model_id.to_string())
            .fetch_all(pool)
            .await?;

        Ok(result)
    }

    pub async fn list(pool: &PgPool, _query_params: std::collections::HashMap<String, String>) -> Result<Vec<SysModelHasPermission>> {
        let query = "SELECT * FROM sys_model_has_permissions ORDER BY created_at DESC";
        let result = sqlx::query_as::<_, SysModelHasPermission>(query)
            .fetch_all(pool)
            .await?;
        Ok(result)
    }

    pub async fn update(pool: &PgPool, id: Ulid, data: UpdateSysModelHasPermission) -> Result<SysModelHasPermission> {
        let query = r#"
            UPDATE sys_model_has_permissions
            SET model_type = COALESCE($2, model_type),
                model_id = COALESCE($3, model_id),
                permission_id = COALESCE($4, permission_id),
                scope_type = COALESCE($5, scope_type),
                scope_id = COALESCE($6, scope_id),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#;

        let result = sqlx::query_as::<_, SysModelHasPermission>(query)
            .bind(id.to_string())
            .bind(data.model_type)
            .bind(data.model_id.map(|id| id.to_string()))
            .bind(data.permission_id.map(|id| id.to_string()))
            .bind(data.scope_type)
            .bind(data.scope_id.map(|id| id.to_string()))
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn delete(pool: &PgPool, id: Ulid) -> Result<()> {
        let query = "DELETE FROM sys_model_has_permissions WHERE id = $1";

        sqlx::query(query)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn assign_permission_to_model(
        pool: &PgPool,
        model_type: &str,
        model_id: Ulid,
        permission_id: Ulid,
        scope_type: Option<String>,
        scope_id: Option<Ulid>,
    ) -> Result<SysModelHasPermission> {
        let data = CreateSysModelHasPermission {
            model_type: model_type.to_string(),
            model_id,
            permission_id,
            scope_type,
            scope_id,
        };
        Self::create(pool, data).await
    }

    pub async fn remove_permission_from_model(
        pool: &PgPool,
        model_type: &str,
        model_id: Ulid,
        permission_id: Ulid,
    ) -> Result<()> {
        let query = "DELETE FROM sys_model_has_permissions WHERE model_type = $1 AND model_id = $2 AND permission_id = $3";

        sqlx::query(query)
            .bind(model_type)
            .bind(model_id.to_string())
            .bind(permission_id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_model_permissions(
        pool: &PgPool,
        model_type: &str,
        model_id: Ulid,
        guard_name: Option<&str>,
    ) -> Result<Vec<crate::app::models::permission::Permission>> {
        use crate::app::models::permission::Permission;

        let query = if let Some(guard) = guard_name {
            sqlx::query_as::<_, Permission>(
                r#"
                SELECT p.id, p.name, p.description, p.guard_name, p.created_at, p.updated_at
                FROM sys_permissions p
                JOIN sys_model_has_permissions smhp ON p.id = smhp.permission_id
                WHERE smhp.model_type = $1 AND smhp.model_id = $2 AND p.guard_name = $3
                ORDER BY p.name
                "#
            )
            .bind(model_type)
            .bind(model_id.to_string())
            .bind(guard)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, Permission>(
                r#"
                SELECT p.id, p.name, p.description, p.guard_name, p.created_at, p.updated_at
                FROM sys_permissions p
                JOIN sys_model_has_permissions smhp ON p.id = smhp.permission_id
                WHERE smhp.model_type = $1 AND smhp.model_id = $2
                ORDER BY p.name
                "#
            )
            .bind(model_type)
            .bind(model_id.to_string())
            .fetch_all(pool)
            .await?
        };

        Ok(query)
    }
}