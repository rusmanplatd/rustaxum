use anyhow::Result;
use ulid::Ulid;
use crate::database::DbPool;
use std::collections::HashMap;
use diesel::prelude::*;
use crate::schema::OrganizationPositionLevel;

use crate::app::models::joblevel::{OrganizationPositionLevel, CreateOrganizationPositionLevel, UpdateOrganizationPositionLevel};

pub struct OrganizationPositionLevelService;

impl OrganizationPositionLevelService {
    pub fn create(pool: &DbPool, data: CreateOrganizationPositionLevel) -> Result<OrganizationPositionLevel> {
        let mut conn = pool.get()?;
        let organization_position_level = OrganizationPositionLevel::new(data.name, data.code, data.level, data.description);

        diesel::insert_into(OrganizationPositionLevel::table)
            .values((
                OrganizationPositionLevel::id.eq(organization_position_level.id.to_string()),
                OrganizationPositionLevel::name.eq(&organization_position_level.name),
                OrganizationPositionLevel::code.eq(&organization_position_level.code),
                OrganizationPositionLevel::level.eq(organization_position_level.level),
                OrganizationPositionLevel::description.eq(&organization_position_level.description),
                OrganizationPositionLevel::is_active.eq(organization_position_level.is_active),
                OrganizationPositionLevel::created_at.eq(organization_position_level.created_at),
                OrganizationPositionLevel::updated_at.eq(organization_position_level.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(organization_position_level)
    }

    pub fn find_by_id(pool: &DbPool, id: Ulid) -> Result<Option<OrganizationPositionLevel>> {
        let mut conn = pool.get()?;

        let result = OrganizationPositionLevel::table
            .filter(OrganizationPositionLevel::id.eq(id.to_string()))
            .first::<OrganizationPositionLevel>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_level(pool: &DbPool, level: i32) -> Result<Option<OrganizationPositionLevel>> {
        let mut conn = pool.get()?;

        let result = OrganizationPositionLevel::table
            .filter(OrganizationPositionLevel::level.eq(level))
            .first::<OrganizationPositionLevel>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn list(pool: &DbPool, _query_params: HashMap<String, String>) -> Result<Vec<OrganizationPositionLevel>> {
        let mut conn = pool.get()?;

        let result = OrganizationPositionLevel::table
            .order(OrganizationPositionLevel::level.asc())
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

        diesel::update(OrganizationPositionLevel::table.filter(OrganizationPositionLevel::id.eq(id.to_string())))
            .set((
                OrganizationPositionLevel::name.eq(&current.name),
                OrganizationPositionLevel::code.eq(&current.code),
                OrganizationPositionLevel::level.eq(current.level),
                OrganizationPositionLevel::description.eq(&current.description),
                OrganizationPositionLevel::is_active.eq(current.is_active),
                OrganizationPositionLevel::updated_at.eq(current.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(current)
    }

    pub fn delete(pool: &DbPool, id: Ulid) -> Result<()> {
        let mut conn = pool.get()?;

        let rows_affected = diesel::delete(OrganizationPositionLevel::table.filter(OrganizationPositionLevel::id.eq(id.to_string())))
            .execute(&mut conn)?;

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("Job level not found"));
        }

        Ok(())
    }

    pub fn find_active_levels(pool: &DbPool) -> Result<Vec<OrganizationPositionLevel>> {
        let mut conn = pool.get()?;

        let results = OrganizationPositionLevel::table
            .filter(OrganizationPositionLevel::is_active.eq(true))
            .order(OrganizationPositionLevel::level.asc())
            .load::<OrganizationPositionLevel>(&mut conn)?;

        Ok(results)
    }

    pub fn find_by_level_range(pool: &DbPool, min_level: i32, max_level: i32) -> Result<Vec<OrganizationPositionLevel>> {
        let mut conn = pool.get()?;

        let results = OrganizationPositionLevel::table
            .filter(OrganizationPositionLevel::level.ge(min_level))
            .filter(OrganizationPositionLevel::level.le(max_level))
            .order(OrganizationPositionLevel::level.asc())
            .load::<OrganizationPositionLevel>(&mut conn)?;

        Ok(results)
    }
}