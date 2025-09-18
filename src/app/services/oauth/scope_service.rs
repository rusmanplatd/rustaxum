use anyhow::Result;
use sqlx::PgPool;
use ulid::Ulid;

use crate::app::models::oauth::{Scope, CreateScope, UpdateScope, ScopeResponse};
use crate::query_builder::QueryBuilder;

pub struct ScopeService;

impl ScopeService {
    pub async fn create_scope(pool: &PgPool, data: CreateScope) -> Result<ScopeResponse> {
        // Check if scope already exists
        if Self::find_by_name(pool, &data.name).await?.is_some() {
            return Err(anyhow::anyhow!("Scope with this name already exists"));
        }

        let scope = Scope::new(data.name, data.description, data.is_default);

        let created_scope = Self::create_scope_record(pool, scope).await?;
        Ok(created_scope.to_response())
    }

    pub async fn create_scope_record(pool: &PgPool, scope: Scope) -> Result<Scope> {
        sqlx::query(
            r#"
            INSERT INTO oauth_scopes (
                id, name, description, is_default, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(scope.id.to_string())
        .bind(&scope.name)
        .bind(&scope.description)
        .bind(scope.is_default)
        .bind(scope.created_at)
        .bind(scope.updated_at)
        .execute(pool)
        .await?;

        Ok(scope)
    }

    pub async fn find_by_id(pool: &PgPool, id: Ulid) -> Result<Option<Scope>> {
        let row = sqlx::query_as::<_, Scope>(
            "SELECT * FROM oauth_scopes WHERE id = $1"
        )
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    pub async fn find_by_name(pool: &PgPool, name: &str) -> Result<Option<Scope>> {
        let row = sqlx::query_as::<_, Scope>(
            "SELECT * FROM oauth_scopes WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(row)
    }

    pub async fn list_scopes(pool: &PgPool) -> Result<Vec<ScopeResponse>> {
        let request = crate::query_builder::QueryBuilderRequest::default();
        let query_builder = QueryBuilder::<Scope>::new(pool.clone(), request);
        let scopes = query_builder.get().await?;
        Ok(scopes.into_iter().map(|s| s.to_response()).collect())
    }

    pub async fn list_default_scopes(pool: &PgPool) -> Result<Vec<Scope>> {
        let mut request = crate::query_builder::QueryBuilderRequest::default();
        request.filters.insert("is_default".to_string(), "true".to_string());
        let query_builder = QueryBuilder::<Scope>::new(pool.clone(), request);
        query_builder.get().await
    }

    pub async fn update_scope(pool: &PgPool, id: Ulid, data: UpdateScope) -> Result<ScopeResponse> {
        let mut scope = Self::find_by_id(pool, id).await?
            .ok_or_else(|| anyhow::anyhow!("Scope not found"))?;

        if let Some(name) = data.name {
            // Check if new name conflicts with existing scope
            if let Some(existing) = Self::find_by_name(pool, &name).await? {
                if existing.id != scope.id {
                    return Err(anyhow::anyhow!("Scope with this name already exists"));
                }
            }
            scope.name = name;
        }

        if let Some(description) = data.description {
            scope.description = Some(description);
        }

        if let Some(is_default) = data.is_default {
            scope.is_default = is_default;
        }

        scope.updated_at = chrono::Utc::now();

        sqlx::query(
            r#"
            UPDATE oauth_scopes
            SET name = $1, description = $2, is_default = $3, updated_at = $4
            WHERE id = $5
            "#
        )
        .bind(&scope.name)
        .bind(&scope.description)
        .bind(scope.is_default)
        .bind(scope.updated_at)
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Ok(scope.to_response())
    }

    pub async fn delete_scope(pool: &PgPool, id: Ulid) -> Result<()> {
        let scope = Self::find_by_id(pool, id).await?
            .ok_or_else(|| anyhow::anyhow!("Scope not found"))?;

        // Prevent deletion of wildcard scope
        if scope.is_wildcard() {
            return Err(anyhow::anyhow!("Cannot delete wildcard scope"));
        }

        sqlx::query(
            "DELETE FROM oauth_scopes WHERE id = $1"
        )
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn validate_scopes(pool: &PgPool, scope_names: &[String]) -> Result<Vec<Scope>> {
        if scope_names.is_empty() {
            // Return default scopes if none specified
            return Self::list_default_scopes(pool).await;
        }

        let mut valid_scopes = Vec::new();

        for scope_name in scope_names {
            if let Some(scope) = Self::find_by_name(pool, scope_name).await? {
                valid_scopes.push(scope);
            } else {
                return Err(anyhow::anyhow!("Invalid scope: {}", scope_name));
            }
        }

        Ok(valid_scopes)
    }

    pub async fn get_scope_names(scopes: &[Scope]) -> Vec<String> {
        scopes.iter().map(|s| s.name.clone()).collect()
    }

    pub fn scope_includes(granted_scopes: &[String], required_scope: &str) -> bool {
        granted_scopes.contains(&"*".to_string()) || granted_scopes.contains(&required_scope.to_string())
    }
}

