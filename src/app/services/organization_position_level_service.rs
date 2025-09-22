use anyhow::Result;
use crate::database::DbPool;
use std::collections::HashMap;
use diesel::prelude::*;
use crate::schema::organization_position_levels;

use crate::app::models::organization_position_level::{OrganizationPositionLevel, CreateOrganizationPositionLevel, UpdateOrganizationPositionLevel, NewOrganizationPositionLevel};

pub struct OrganizationPositionLevelService;

impl OrganizationPositionLevelService {
    pub fn create(pool: &DbPool, data: CreateOrganizationPositionLevel) -> Result<OrganizationPositionLevel> {
        let mut conn = pool.get()?;
        let new_level = NewOrganizationPositionLevel::new(data, None); // TODO: Add created_by from auth context

        let result = diesel::insert_into(organization_position_levels::table)
            .values(&new_level)
            .get_result::<OrganizationPositionLevel>(&mut conn)?;

        Ok(result)
    }

    pub fn find_by_id(pool: &DbPool, id: &str) -> Result<Option<OrganizationPositionLevel>> {
        let mut conn = pool.get()?;

        let result = organization_position_levels::table
            .filter(organization_position_levels::id.eq(id))
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

    pub fn update(pool: &DbPool, id: String, data: UpdateOrganizationPositionLevel) -> Result<OrganizationPositionLevel> {
        let mut conn = pool.get()?;
        let mut current = Self::find_by_id(pool, &id)?
            .ok_or_else(|| anyhow::anyhow!("Position level not found"))?;

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
        // TODO: Add updated_by from auth context

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

        Ok(result)
    }

    pub fn delete(pool: &DbPool, id: String) -> Result<()> {
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