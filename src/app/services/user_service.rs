use anyhow::Result;
use ulid::Ulid;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::app::models::user::{User, CreateUser, UpdateUser};
use crate::app::models::token_blacklist::TokenBlacklist;

pub struct UserService;

impl UserService {
    pub async fn create_user(_pool: &PgPool, data: CreateUser) -> Result<User> {
        let user = User::new(data.name, data.email, data.password);
        Ok(user)
    }

    pub async fn create_user_record(pool: &PgPool, user: User) -> Result<User> {
        let query = r#"
            INSERT INTO users (id, name, email, password, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
        "#;

        let result = sqlx::query_as::<_, User>(query)
            .bind(user.id.to_string())
            .bind(&user.name)
            .bind(&user.email)
            .bind(&user.password)
            .bind(user.created_at)
            .bind(user.updated_at)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn find_by_id(pool: &PgPool, id: Ulid) -> Result<Option<User>> {
        let query = "SELECT * FROM users WHERE id = $1";

        let result = sqlx::query_as::<_, User>(query)
            .bind(id.to_string())
            .fetch_optional(pool)
            .await?;

        Ok(result)
    }

    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>> {
        let query = "SELECT * FROM users WHERE email = $1";

        let result = sqlx::query_as::<_, User>(query)
            .bind(email)
            .fetch_optional(pool)
            .await?;

        Ok(result)
    }

    pub async fn find_by_reset_token(pool: &PgPool, token: &str) -> Result<Option<User>> {
        let query = "SELECT * FROM users WHERE password_reset_token = $1 AND password_reset_expires_at > NOW()";

        let result = sqlx::query_as::<_, User>(query)
            .bind(token)
            .fetch_optional(pool)
            .await?;

        Ok(result)
    }

    pub async fn update_user(pool: &PgPool, id: Ulid, data: UpdateUser) -> Result<User> {
        let query = r#"
            UPDATE users
            SET name = COALESCE($2, name),
                email = COALESCE($3, email),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#;

        let result = sqlx::query_as::<_, User>(query)
            .bind(id.to_string())
            .bind(data.name)
            .bind(data.email)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn update_password(pool: &PgPool, id: Ulid, new_password: String) -> Result<()> {
        let query = "UPDATE users SET password = $1, updated_at = NOW() WHERE id = $2";

        sqlx::query(query)
            .bind(new_password)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn update_last_login(pool: &PgPool, id: Ulid) -> Result<()> {
        let query = "UPDATE users SET last_login_at = NOW(), updated_at = NOW() WHERE id = $1";

        sqlx::query(query)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn update_failed_attempts(pool: &PgPool, id: Ulid, attempts: i32, locked_until: Option<DateTime<Utc>>) -> Result<()> {
        let query = "UPDATE users SET failed_login_attempts = $1, locked_until = $2, updated_at = NOW() WHERE id = $3";

        sqlx::query(query)
            .bind(attempts)
            .bind(locked_until)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn reset_failed_attempts(pool: &PgPool, id: Ulid) -> Result<()> {
        let query = "UPDATE users SET failed_login_attempts = 0, locked_until = NULL, updated_at = NOW() WHERE id = $1";

        sqlx::query(query)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn update_password_reset_token(pool: &PgPool, id: Ulid, token: Option<String>, expires_at: Option<DateTime<Utc>>) -> Result<()> {
        let query = "UPDATE users SET password_reset_token = $1, password_reset_expires_at = $2, updated_at = NOW() WHERE id = $3";

        sqlx::query(query)
            .bind(token)
            .bind(expires_at)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn update_refresh_token(pool: &PgPool, id: Ulid, token: Option<String>, expires_at: Option<DateTime<Utc>>) -> Result<()> {
        let query = "UPDATE users SET refresh_token = $1, refresh_token_expires_at = $2, updated_at = NOW() WHERE id = $3";

        sqlx::query(query)
            .bind(token)
            .bind(expires_at)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn find_by_refresh_token(pool: &PgPool, token: &str) -> Result<Option<User>> {
        let query = "SELECT * FROM users WHERE refresh_token = $1 AND refresh_token_expires_at > NOW()";

        let result = sqlx::query_as::<_, User>(query)
            .bind(token)
            .fetch_optional(pool)
            .await?;

        Ok(result)
    }

    pub async fn delete_user(pool: &PgPool, id: Ulid) -> Result<()> {
        let query = "DELETE FROM users WHERE id = $1";

        sqlx::query(query)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn blacklist_token(pool: &PgPool, token: TokenBlacklist) -> Result<()> {
        let query = r#"
            INSERT INTO token_blacklist (id, token_hash, user_id, expires_at, revoked_at, reason)
            VALUES ($1, $2, $3, $4, $5, $6)
        "#;

        sqlx::query(query)
            .bind(token.id.to_string())
            .bind(&token.token_hash)
            .bind(token.user_id.to_string())
            .bind(token.expires_at)
            .bind(token.revoked_at)
            .bind(token.reason)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn is_token_blacklisted(pool: &PgPool, token_hash: &str) -> Result<bool> {
        let query = "SELECT EXISTS(SELECT 1 FROM token_blacklist WHERE token_hash = $1 AND expires_at > NOW())";

        let result: (bool,) = sqlx::query_as(query)
            .bind(token_hash)
            .fetch_one(pool)
            .await?;

        Ok(result.0)
    }

    pub async fn cleanup_expired_tokens(pool: &PgPool) -> Result<u64> {
        let query = "DELETE FROM token_blacklist WHERE expires_at <= NOW()";

        let result = sqlx::query(query)
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }
}