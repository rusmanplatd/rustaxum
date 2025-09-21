use anyhow::Result;
use ulid::Ulid;
use crate::database::DbPool;
use std::collections::HashMap;
use diesel::prelude::*;
use crate::schema::organization_position_levels;

use crate::app::models::organization_position_level::{OrganizationPositionLevel, CreateOrganizationPositionLevel, UpdateOrganizationPositionLevel};

pub struct OrganizationPositionLevelService;

impl OrganizationPositionLevelService {
    pub fn create(pool: &DbPool, data: CreateOrganizationPositionLevel) -> Result<OrganizationPositionLevel> {
        let mut conn = pool.get()?;
        let organization_position_level = OrganizationPositionLevel::new(data.name, data.code, data.level, data.description);

        diesel::insert_into(organization_position_levels::table)
            .values((
                organization_position_levels::id.eq(organization_position_level.id.to_string()),
                organization_position_levels::name.eq(&organization_position_level.name),
                organization_position_levels::code.eq(&organization_position_level.code),
                organization_position_levels::level.eq(organization_position_level.level),
                organization_position_levels::description.eq(&organization_position_level.description),
                organization_position_levels::is_active.eq(organization_position_level.is_active),
                organization_position_levels::created_at.eq(organization_position_level.created_at),
                organization_position_levels::updated_at.eq(organization_position_level.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(organization_position_level)
    }

    pub fn find_by_id(pool: &DbPool, id: Ulid) -> Result<Option<OrganizationPositionLevel>> {
        let mut conn = pool.get()?;

        let result = organization_position_levels::table
            .filter(organization_position_levels::id.eq(id.to_string()))
            .first::<OrganizationPositionLevel>(&mut conn)
            .optional()?;

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

    pub fn list(pool: &DbPool, _query_params: HashMap<String, String>) -> Result<Vec<OrganizationPositionLevel>> {
        let mut conn = pool.get()?;

        let result = organization_position_levels::table
            .order(organization_position_levels::level.asc())
            .load::<OrganizationPositionLevel>(&mut conn)?;

        Ok(result)
    }

    pub fn update(pool: &DbPool, id: Ulid, data: UpdateOrganizationPositionLevel) -> Result<OrganizationPositionLevel> {
        let mut conn = pool.get()?;
        let mut current = Self::find_by_id(pool, id)?
            .ok_or_else(|| anyhow::anyhow!("Job level not found"))?;

        // Update fields if provided
        if let Some(name) = data.name {
            current.name = name;
        }
        if let Some(code) = data.code {
            current.code = Some(code);
        }
        if let Some(level) = data.level {
            current.level = level;
        }
        if let Some(description) = data.description {
            current.description = Some(description);
        }
        if let Some(is_active) = data.is_active {
            current.is_active = is_active;
        }
        current.updated_at = chrono::Utc::now();

        diesel::update(organization_position_levels::table.filter(organization_position_levels::id.eq(id.to_string())))
            .set((
                organization_position_levels::name.eq(&current.name),
                organization_position_levels::code.eq(&current.code),
                organization_position_levels::level.eq(current.level),
                organization_position_levels::description.eq(&current.description),
                organization_position_levels::is_active.eq(current.is_active),
                organization_position_levels::updated_at.eq(current.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(current)
    }

    pub fn delete(pool: &DbPool, id: Ulid) -> Result<()> {
        let mut conn = pool.get()?;

        let rows_affected = diesel::delete(organization_position_levels::table.filter(organization_position_levels::id.eq(id.to_string())))
            .execute(&mut conn)?;

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("Job level not found"));
        }

        Ok(())
    }

    pub fn find_active_levels(pool: &DbPool) -> Result<Vec<OrganizationPositionLevel>> {
        let mut conn = pool.get()?;

        let results = organization_position_levels::table
            .filter(organization_position_levels::is_active.eq(true))
            .order(organization_position_levels::level.asc())
            .load::<OrganizationPositionLevel>(&mut conn)?;

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
}