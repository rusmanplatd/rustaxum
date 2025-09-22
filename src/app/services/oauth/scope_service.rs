use anyhow::Result;
use crate::database::DbPool;
use ulid::Ulid;
use diesel::prelude::*;
use crate::schema::oauth_scopes;

use crate::app::models::oauth::{Scope, CreateScope, UpdateScope, ScopeResponse};
use crate::app::query_builder::QueryBuilder;

pub struct ScopeService;

impl ScopeService {
    pub fn create_scope(pool: &DbPool, data: CreateScope) -> Result<ScopeResponse> {
        // Check if scope already exists
        if Self::find_by_name(pool, &data.name)?.is_some() {
            return Err(anyhow::anyhow!("Scope with this name already exists"));
        }

        let scope = Scope::new(data.name, data.description, data.is_default);

        let created_scope = Self::create_scope_record(pool, scope)?;
        Ok(created_scope.to_response())
    }

    pub fn create_scope_record(pool: &DbPool, scope: Scope) -> Result<Scope> {
        let mut conn = pool.get()?;

        diesel::insert_into(oauth_scopes::table)
            .values((
                oauth_scopes::id.eq(scope.id.to_string()),
                oauth_scopes::name.eq(&scope.name),
                oauth_scopes::description.eq(&scope.description),
                oauth_scopes::is_default.eq(scope.is_default),
                oauth_scopes::created_at.eq(scope.created_at),
                oauth_scopes::updated_at.eq(scope.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(scope)
    }

    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<Scope>> {
        let mut conn = pool.get()?;

        let row = oauth_scopes::table
            .filter(oauth_scopes::id.eq(id.to_string()))
            .first::<Scope>(&mut conn)
            .optional()?;

        Ok(row)
    }

    pub fn find_by_name(pool: &DbPool, name: &str) -> Result<Option<Scope>> {
        let mut conn = pool.get()?;

        let row = oauth_scopes::table
            .filter(oauth_scopes::name.eq(name))
            .first::<Scope>(&mut conn)
            .optional()?;

        Ok(row)
    }

    pub fn list_scopes(pool: &DbPool) -> Result<Vec<ScopeResponse>> {
        let request = crate::app::query_builder::QueryParams::default();
        let query_builder = QueryBuilder::<Scope>::new(pool.clone(), request);
        let scopes = query_builder.get()?;
        Ok(scopes.into_iter().map(|s| s.to_response()).collect())
    }

    pub fn list_default_scopes(pool: &DbPool) -> Result<Vec<Scope>> {
        let mut conn = pool.get()?;

        let scopes = oauth_scopes::table
            .filter(oauth_scopes::is_default.eq(true))
            .load::<Scope>(&mut conn)?;

        Ok(scopes)
    }

    pub fn update_scope(pool: &DbPool, id: String, data: UpdateScope) -> Result<ScopeResponse> {
        let mut scope = Self::find_by_id(pool, id)?
            .ok_or_else(|| anyhow::anyhow!("Scope not found"))?;

        if let Some(name) = data.name {
            // Check if new name conflicts with existing scope
            if let Some(existing) = Self::find_by_name(pool, &name)? {
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

        let mut conn = pool.get()?;

        diesel::update(oauth_scopes::table.filter(oauth_scopes::id.eq(id.to_string())))
            .set((
                oauth_scopes::name.eq(&scope.name),
                oauth_scopes::description.eq(&scope.description),
                oauth_scopes::is_default.eq(scope.is_default),
                oauth_scopes::updated_at.eq(scope.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(scope.to_response())
    }

    pub fn delete_scope(pool: &DbPool, id: String) -> Result<()> {
        let scope = Self::find_by_id(pool, id)?
            .ok_or_else(|| anyhow::anyhow!("Scope not found"))?;

        // Prevent deletion of wildcard scope
        if scope.is_wildcard() {
            return Err(anyhow::anyhow!("Cannot delete wildcard scope"));
        }

        let mut conn = pool.get()?;

        diesel::delete(oauth_scopes::table.filter(oauth_scopes::id.eq(id.to_string())))
            .execute(&mut conn)?;

        Ok(())
    }

    pub async fn validate_scopes(pool: &DbPool, scope_names: &[String]) -> Result<Vec<Scope>> {
        if scope_names.is_empty() {
            // Return default scopes if none specified
            return Self::list_default_scopes(pool);
        }

        let mut valid_scopes = Vec::new();

        for scope_name in scope_names {
            if let Some(scope) = Self::find_by_name(pool, scope_name)? {
                valid_scopes.push(scope);
            } else {
                return Err(anyhow::anyhow!("Invalid scope: {}", scope_name));
            }
        }

        Ok(valid_scopes)
    }

    pub fn get_scope_names(scopes: &[Scope]) -> Vec<String> {
        scopes.iter().map(|s| s.name.clone()).collect()
    }

    pub fn scope_includes(granted_scopes: &[String], required_scope: &str) -> bool {
        granted_scopes.contains(&"*".to_string()) || granted_scopes.contains(&required_scope.to_string())
    }
}

