use anyhow::Result;
use ulid::Ulid;
use sqlx::PgPool;
use crate::app::models::role::{Role, CreateRole, UpdateRole};

pub struct RoleService;

impl RoleService {
    pub async fn create(pool: &PgPool, data: CreateRole) -> Result<Role> {
        let role = Role::new(data.name, data.description, data.guard_name);

        sqlx::query(
            r#"
            INSERT INTO sys_roles (id, name, description, guard_name, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(role.id.to_string())
        .bind(role.name.clone())
        .bind(role.description.clone())
        .bind(role.guard_name.clone())
        .bind(role.created_at)
        .bind(role.updated_at)
        .execute(pool)
        .await?;

        Ok(role)
    }

    pub async fn find_by_id(pool: &PgPool, id: Ulid) -> Result<Option<Role>> {
        let role = sqlx::query_as::<_, Role>(
            "SELECT id, name, description, guard_name, created_at, updated_at FROM sys_roles WHERE id = $1"
        )
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?;

        Ok(role)
    }

    pub async fn find_by_name(pool: &PgPool, name: &str, guard_name: Option<&str>) -> Result<Option<Role>> {
        let guard = guard_name.unwrap_or("api");

        let role = sqlx::query_as::<_, Role>(
            "SELECT id, name, description, guard_name, created_at, updated_at FROM sys_roles WHERE name = $1 AND guard_name = $2"
        )
        .bind(name)
        .bind(guard)
        .fetch_optional(pool)
        .await?;

        Ok(role)
    }

    pub async fn update(pool: &PgPool, id: Ulid, data: UpdateRole) -> Result<Role> {
        let mut role = Self::find_by_id(pool, id).await?
            .ok_or_else(|| anyhow::anyhow!("Role not found"))?;

        if let Some(name) = data.name {
            role.name = name;
        }
        if data.description.is_some() {
            role.description = data.description;
        }
        if let Some(guard_name) = data.guard_name {
            role.guard_name = guard_name;
        }
        role.updated_at = chrono::Utc::now();

        sqlx::query(
            r#"
            UPDATE sys_roles
            SET name = $1, description = $2, guard_name = $3, updated_at = $4
            WHERE id = $5
            "#
        )
        .bind(role.name.clone())
        .bind(role.description.clone())
        .bind(role.guard_name.clone())
        .bind(role.updated_at)
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Ok(role)
    }

    pub async fn delete(pool: &PgPool, id: Ulid) -> Result<()> {
        sqlx::query(
            "DELETE FROM sys_roles WHERE id = $1"
        )
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn list(pool: &PgPool, limit: i64, offset: i64) -> Result<Vec<Role>> {
        let roles = sqlx::query_as::<_, Role>(
            "SELECT id, name, description, guard_name, created_at, updated_at FROM sys_roles ORDER BY created_at DESC LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(roles)
    }

    pub async fn assign_to_user(pool: &PgPool, user_id: Ulid, role_id: Ulid) -> Result<()> {
        let user_role_id = Ulid::new();
        let now = chrono::Utc::now();

        sqlx::query(
            r#"
            INSERT INTO user_roles (id, user_id, role_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id, role_id) DO NOTHING
            "#
        )
        .bind(user_role_id.to_string())
        .bind(user_id.to_string())
        .bind(role_id.to_string())
        .bind(now)
        .bind(now)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn remove_from_user(pool: &PgPool, user_id: Ulid, role_id: Ulid) -> Result<()> {
        sqlx::query(
            "DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2"
        )
        .bind(user_id.to_string())
        .bind(role_id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn user_has_role(pool: &PgPool, user_id: Ulid, role_name: &str, guard_name: Option<&str>) -> Result<bool> {
        let guard = guard_name.unwrap_or("api");

        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) as count
            FROM user_roles ur
            JOIN sys_roles r ON ur.role_id = r.id
            WHERE ur.user_id = $1 AND r.name = $2 AND r.guard_name = $3
            "#
        )
        .bind(user_id.to_string())
        .bind(role_name)
        .bind(guard)
        .fetch_one(pool)
        .await?;

        Ok(count.unwrap_or(0) > 0)
    }

    pub async fn get_user_roles(pool: &PgPool, user_id: Ulid, guard_name: Option<&str>) -> Result<Vec<Role>> {
        let query = if let Some(guard) = guard_name {
            sqlx::query_as::<_, Role>(
                r#"
                SELECT r.id, r.name, r.description, r.guard_name, r.created_at, r.updated_at
                FROM sys_roles r
                JOIN user_roles ur ON r.id = ur.role_id
                WHERE ur.user_id = $1 AND r.guard_name = $2
                ORDER BY r.name
                "#
            )
            .bind(user_id.to_string())
            .bind(guard)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, Role>(
                r#"
                SELECT r.id, r.name, r.description, r.guard_name, r.created_at, r.updated_at
                FROM sys_roles r
                JOIN user_roles ur ON r.id = ur.role_id
                WHERE ur.user_id = $1
                ORDER BY r.name
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
            "SELECT COUNT(*) as count FROM roles"
        )
        .fetch_one(pool)
        .await?;

        Ok(count.unwrap_or(0))
    }
}