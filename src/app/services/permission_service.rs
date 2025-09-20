use anyhow::Result;
use ulid::Ulid;
use sqlx::PgPool;
use crate::app::models::permission::{Permission, CreatePermission, UpdatePermission};

pub struct PermissionService;

impl PermissionService {
    pub async fn create(pool: &PgPool, data: CreatePermission) -> Result<Permission> {
        let permission = Permission::new(
            data.name,
            data.guard_name,
            data.resource,
            data.action,
        );

        sqlx::query(
            r#"
            INSERT INTO sys_permissions (id, name, guard_name, resource, action, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(permission.id.to_string())
        .bind(permission.name.clone())
        .bind(permission.guard_name.clone())
        .bind(permission.resource.clone())
        .bind(permission.action.clone())
        .bind(permission.created_at)
        .bind(permission.updated_at)
        .execute(pool)
        .await?;

        Ok(permission)
    }

    pub async fn find_by_id(pool: &PgPool, id: Ulid) -> Result<Option<Permission>> {
        let permission = sqlx::query_as::<_, Permission>(
            "SELECT id, name, guard_name, resource, action, created_at, updated_at FROM sys_permissions WHERE id = $1"
        )
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?;

        Ok(permission)
    }

    pub async fn find_by_name(pool: &PgPool, name: &str, guard_name: Option<&str>) -> Result<Option<Permission>> {
        let guard = guard_name.unwrap_or("api");

        let permission = sqlx::query_as::<_, Permission>(
            "SELECT id, name, guard_name, resource, action, created_at, updated_at FROM sys_permissions WHERE name = $1 AND guard_name = $2"
        )
        .bind(name)
        .bind(guard)
        .fetch_optional(pool)
        .await?;

        Ok(permission)
    }

    pub async fn update(pool: &PgPool, id: Ulid, data: UpdatePermission) -> Result<Permission> {
        let mut permission = Self::find_by_id(pool, id).await?
            .ok_or_else(|| anyhow::anyhow!("Permission not found"))?;

        if let Some(name) = data.name {
            permission.name = name;
        }
        if let Some(guard_name) = data.guard_name {
            permission.guard_name = guard_name;
        }
        if data.resource.is_some() {
            permission.resource = data.resource;
        }
        if let Some(action) = data.action {
            permission.action = action;
        }
        permission.updated_at = chrono::Utc::now();

        sqlx::query(
            r#"
            UPDATE sys_permissions
            SET name = $1, guard_name = $2, resource = $3, action = $4, updated_at = $5
            WHERE id = $6
            "#
        )
        .bind(permission.name.clone())
        .bind(permission.guard_name.clone())
        .bind(permission.resource.clone())
        .bind(permission.action.clone())
        .bind(permission.updated_at)
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Ok(permission)
    }

    pub async fn delete(pool: &PgPool, id: Ulid) -> Result<()> {
        sqlx::query(
            "DELETE FROM sys_permissions WHERE id = $1"
        )
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn list(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<Permission>> {
        let permissions = sqlx::query_as::<_, Permission>(
            "SELECT id, name, guard_name, resource, action, created_at, updated_at FROM sys_permissions ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(permissions)
    }

    pub async fn assign_to_role(pool: &PgPool, role_id: Ulid, permission_id: Ulid) -> Result<()> {
        let role_permission_id = Ulid::new();
        let now = chrono::Utc::now();

        sqlx::query(
            r#"
            INSERT INTO role_permissions (id, role_id, permission_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (role_id, permission_id) DO NOTHING
            "#
        )
        .bind(role_permission_id.to_string())
        .bind(role_id.to_string())
        .bind(permission_id.to_string())
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn remove_from_role(pool: &PgPool, role_id: Ulid, permission_id: Ulid) -> Result<()> {
        sqlx::query(
            "DELETE FROM role_permissions WHERE role_id = $1 AND permission_id = $2"
        )
        .bind(role_id.to_string())
        .bind(permission_id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn role_has_permission(pool: &PgPool, role_id: Ulid, permission_name: &str, guard_name: Option<&str>) -> Result<bool> {
        let guard = guard_name.unwrap_or("api");

        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) as count
            FROM role_permissions rp
            JOIN sys_permissions p ON rp.permission_id = p.id
            WHERE rp.role_id = $1 AND p.name = $2 AND p.guard_name = $3
            "#
        )
        .bind(role_id.to_string())
        .bind(permission_name)
        .bind(guard)
        .fetch_one(pool)
        .await?;

        Ok(count.unwrap_or(0) > 0)
    }

    pub async fn user_has_permission(pool: &PgPool, user_id: Ulid, permission_name: &str, guard_name: Option<&str>) -> Result<bool> {
        let guard = guard_name.unwrap_or("api");

        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) as count
            FROM user_roles ur
            JOIN role_permissions rp ON ur.role_id = rp.role_id
            JOIN sys_permissions p ON rp.permission_id = p.id
            WHERE ur.user_id = $1 AND p.name = $2 AND p.guard_name = $3
            "#
        )
        .bind(user_id.to_string())
        .bind(permission_name)
        .bind(guard)
        .fetch_one(pool)
        .await?;

        Ok(count.unwrap_or(0) > 0)
    }

    pub async fn get_role_permissions(pool: &PgPool, role_id: Ulid, guard_name: Option<&str>) -> Result<Vec<Permission>> {
        let query = if let Some(guard) = guard_name {
            sqlx::query_as::<_, Permission>(
                r#"
                SELECT p.id, p.name, p.guard_name, p.resource, p.action, p.created_at, p.updated_at
                FROM sys_permissions p
                JOIN role_permissions rp ON p.id = rp.permission_id
                WHERE rp.role_id = $1 AND p.guard_name = $2
                ORDER BY p.name
                "#
            )
            .bind(role_id.to_string())
            .bind(guard)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, Permission>(
                r#"
                SELECT p.id, p.name, p.guard_name, p.resource, p.action, p.created_at, p.updated_at
                FROM sys_permissions p
                JOIN role_permissions rp ON p.id = rp.permission_id
                WHERE rp.role_id = $1
                ORDER BY p.name
                "#
            )
            .bind(role_id.to_string())
            .fetch_all(pool)
            .await?
        };

        Ok(query)
    }

    pub async fn get_user_permissions(pool: &PgPool, user_id: Ulid, guard_name: Option<&str>) -> Result<Vec<Permission>> {
        let query = if let Some(guard) = guard_name {
            sqlx::query_as::<_, Permission>(
                r#"
                SELECT DISTINCT p.id, p.name, p.guard_name, p.resource, p.action, p.created_at, p.updated_at
                FROM sys_permissions p
                JOIN role_permissions rp ON p.id = rp.permission_id
                JOIN user_roles ur ON rp.role_id = ur.role_id
                WHERE ur.user_id = $1 AND p.guard_name = $2
                ORDER BY p.name
                "#
            )
            .bind(user_id.to_string())
            .bind(guard)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, Permission>(
                r#"
                SELECT DISTINCT p.id, p.name, p.guard_name, p.resource, p.action, p.created_at, p.updated_at
                FROM sys_permissions p
                JOIN role_permissions rp ON p.id = rp.permission_id
                JOIN user_roles ur ON rp.role_id = ur.role_id
                WHERE ur.user_id = $1
                ORDER BY p.name
                "#
            )
            .bind(user_id.to_string())
            .fetch_all(pool)
            .await?
        };

        Ok(query)
    }

    pub async fn count(pool: &PgPool) -> Result<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) as count FROM permissions"
        )
        .fetch_one(pool)
        .await?;

        Ok(count.unwrap_or(0))
    }
}