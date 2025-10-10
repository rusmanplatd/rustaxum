use anyhow::Result;
use crate::database::DbPool;
use std::collections::HashMap;
use diesel::prelude::*;
use serde_json::json;
use crate::schema::organization_position_levels;
use crate::app::traits::ServiceActivityLogger;

use crate::app::models::organization_position_level::{OrganizationPositionLevel, CreateOrganizationPositionLevel, UpdateOrganizationPositionLevel};
use crate::app::models::DieselUlid;

pub struct OrganizationPositionLevelService;

impl ServiceActivityLogger for OrganizationPositionLevelService {}

impl OrganizationPositionLevelService {
    pub async fn create(pool: &DbPool, data: CreateOrganizationPositionLevel, created_by: Option<&str>) -> Result<OrganizationPositionLevel> {
        let mut conn = pool.get()?;
        let new_level = OrganizationPositionLevel::new(data, created_by.and_then(|s| DieselUlid::from_string(s).ok()));

        let result = diesel::insert_into(organization_position_levels::table)
            .values(&new_level)
            .get_result::<OrganizationPositionLevel>(&mut conn)?;

        // Log activity
        let service = Self;
        let properties = json!({
            "level_name": result.name,
            "level_code": result.code,
            "level_number": result.level,
            "organization_id": result.organization_id.to_string()
        });

        if let Err(e) = service.log_created(
            &result,
            created_by,
            Some(properties)
        ).await {
            eprintln!("Failed to log organization position level creation activity: {}", e);
        }

        Ok(result)
    }

    pub async fn find_by_id(pool: &DbPool, id: &str, user_id: Option<&str>) -> Result<Option<OrganizationPositionLevel>> {
        let mut conn = pool.get()?;

        let result = organization_position_levels::table
            .filter(organization_position_levels::id.eq(id))
            .first::<OrganizationPositionLevel>(&mut conn)
            .optional()?;

        // Log activity if found
        if let Some(level) = &result {
            let service = Self;
            let _properties = json!({
                "level_id": id,
                "level_name": level.name,
                "level_number": level.level
            });

            if let Err(e) = service.log_viewed(
                level,
                user_id
            ).await {
                eprintln!("Failed to log organization position level view activity: {}", e);
            }
        }

        Ok(result)
    }

    pub fn find_by_level(pool: &DbPool, level: i32) -> Result<Option<OrganizationPositionLevel>> {
        let mut conn = pool.get()?;

        let result = organization_position_levels::table
            .filter(organization_position_levels::level.eq(level))
            .first::<OrganizationPositionLevel>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub async fn list(pool: &DbPool, _query_params: HashMap<String, String>, _user_id: Option<&str>) -> Result<Vec<OrganizationPositionLevel>> {
        let mut conn = pool.get()?;

        let result = organization_position_levels::table
            .order(organization_position_levels::level.asc())
            .load::<OrganizationPositionLevel>(&mut conn)?;

        // Log activity
        let service = Self;
        let properties = json!({
            "level_count": result.len()
        });

        if let Err(e) = service.log_system_event(
            "organization_position_levels_listed",
            &format!("Listed {} organization position levels", result.len()),
            Some(properties)
        ).await {
            eprintln!("Failed to log organization position levels list activity: {}", e);
        }

        Ok(result)
    }

    pub async fn update(pool: &DbPool, id: String, data: UpdateOrganizationPositionLevel, updated_by: Option<&str>) -> Result<OrganizationPositionLevel> {
        let mut conn = pool.get()?;
        let mut current = Self::find_by_id(pool, &id, updated_by).await?
            .ok_or_else(|| anyhow::anyhow!("Position level not found"))?;
        let original = current.clone();

        // Update fields if provided
        if let Some(organization_id) = data.organization_id {
            current.organization_id = organization_id;
        }
        if let Some(code) = data.code {
            current.code = code;
        }
        if let Some(name) = data.name {
            current.name = name;
        }
        if let Some(description) = data.description {
            current.description = description;
        }
        if let Some(level) = data.level {
            current.level = level;
        }
        if let Some(is_active) = data.is_active {
            current.is_active = is_active;
        }
        current.updated_at = chrono::Utc::now();

        let result = diesel::update(organization_position_levels::table.filter(organization_position_levels::id.eq(&id)))
            .set((
                organization_position_levels::organization_id.eq(current.organization_id),
                organization_position_levels::code.eq(&current.code),
                organization_position_levels::name.eq(&current.name),
                organization_position_levels::description.eq(&current.description),
                organization_position_levels::level.eq(current.level),
                organization_position_levels::is_active.eq(current.is_active),
                organization_position_levels::updated_at.eq(current.updated_at),
            ))
            .get_result::<OrganizationPositionLevel>(&mut conn)?;

        // Log activity with changes
        let service = Self;
        let mut changes = json!({});

        if original.name != result.name {
            changes["name"] = json!({"from": original.name, "to": result.name});
        }
        if original.code != result.code {
            changes["code"] = json!({"from": original.code, "to": result.code});
        }
        if original.level != result.level {
            changes["level"] = json!({"from": original.level, "to": result.level});
        }
        if original.is_active != result.is_active {
            changes["is_active"] = json!({"from": original.is_active, "to": result.is_active});
        }

        let properties = json!({
            "level_id": id,
            "level_name": result.name,
            "changes": changes
        });

        if let Err(e) = service.log_updated(
            &result,
            properties,
            updated_by
        ).await {
            eprintln!("Failed to log organization position level update activity: {}", e);
        }

        Ok(result)
    }

    pub async fn delete(pool: &DbPool, id: String, deleted_by: Option<&str>) -> Result<()> {
        let mut conn = pool.get()?;

        // First get the level to log it
        let level = Self::find_by_id(pool, &id, deleted_by).await?
            .ok_or_else(|| anyhow::anyhow!("Position level not found"))?;

        let rows_affected = diesel::delete(organization_position_levels::table.filter(organization_position_levels::id.eq(id.to_string())))
            .execute(&mut conn)?;

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("Position level not found"));
        }

        // Log activity
        let service = Self;
        let _properties = json!({
            "level_id": id,
            "level_name": level.name,
            "level_code": level.code,
            "level_number": level.level
        });

        if let Err(e) = service.log_deleted(
            &level,
            deleted_by
        ).await {
            eprintln!("Failed to log organization position level deletion activity: {}", e);
        }

        Ok(())
    }

    pub async fn find_active_levels(pool: &DbPool, _user_id: Option<&str>) -> Result<Vec<OrganizationPositionLevel>> {
        let mut conn = pool.get()?;

        let results = organization_position_levels::table
            .filter(organization_position_levels::is_active.eq(true))
            .order(organization_position_levels::level.asc())
            .load::<OrganizationPositionLevel>(&mut conn)?;

        // Log activity
        let service = Self;
        let properties = json!({
            "active_level_count": results.len()
        });

        if let Err(e) = service.log_system_event(
            "active_organization_position_levels_retrieved",
            &format!("Retrieved {} active organization position levels", results.len()),
            Some(properties)
        ).await {
            eprintln!("Failed to log active organization position levels activity: {}", e);
        }

        Ok(results)
    }

    pub fn find_by_level_range(pool: &DbPool, min_level: i32, max_level: i32) -> Result<Vec<OrganizationPositionLevel>> {
        let mut conn = pool.get()?;

        let results = organization_position_levels::table
            .filter(organization_position_levels::level.ge(min_level))
            .filter(organization_position_levels::level.le(max_level))
            .order(organization_position_levels::level.asc())
            .load::<OrganizationPositionLevel>(&mut conn)?;

        Ok(results)
    }

    pub fn find_by_organization(pool: &DbPool, organization_id: &str) -> Result<Vec<OrganizationPositionLevel>> {
        let mut conn = pool.get()?;

        let results = organization_position_levels::table
            .filter(organization_position_levels::organization_id.eq(organization_id))
            .order(organization_position_levels::level.asc())
            .load::<OrganizationPositionLevel>(&mut conn)?;

        Ok(results)
    }

    pub fn find_active_by_organization(pool: &DbPool, organization_id: &str) -> Result<Vec<OrganizationPositionLevel>> {
        let mut conn = pool.get()?;

        let results = organization_position_levels::table
            .filter(organization_position_levels::organization_id.eq(organization_id))
            .filter(organization_position_levels::is_active.eq(true))
            .order(organization_position_levels::level.asc())
            .load::<OrganizationPositionLevel>(&mut conn)?;

        Ok(results)
    }
}