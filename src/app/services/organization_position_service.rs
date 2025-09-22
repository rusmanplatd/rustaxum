use anyhow::Result;
use crate::database::DbPool;
use chrono::Utc;
use serde_json::{json, Value};
use diesel::prelude::*;
use crate::schema::organization_positions;

use crate::app::models::organization_position::{OrganizationPosition, NewOrganizationPosition, CreateOrganizationPosition};
use crate::app::http::requests::organization_position_requests::{
    CreateOrganizationPositionRequest, UpdateOrganizationPositionRequest, IndexOrganizationPositionRequest, OrganizationPositionsByLevelRequest
};

pub struct OrganizationPositionService;

impl OrganizationPositionService {
    pub fn index(pool: &DbPool, request: &IndexOrganizationPositionRequest) -> Result<Value> {
        let mut conn = pool.get()?;

        let mut query = organization_positions::table.into_boxed();

        // Add filters
        if let Some(is_active) = request.is_active {
            query = query.filter(organization_positions::is_active.eq(is_active));
        }

        if let Some(organization_position_level_id) = &request.organization_position_level_id {
            query = query.filter(organization_positions::organization_position_level_id.eq(organization_position_level_id));
        }

        if let Some(name_search) = &request.name_search {
            query = query.filter(organization_positions::name.ilike(format!("%{}%", name_search)));
        }

        // Add sorting
        let sort_by = request.sort_by.as_deref().unwrap_or("created_at");
        let sort_direction = request.sort_direction.as_deref().unwrap_or("desc");

        query = match (sort_by, sort_direction) {
            ("created_at", "asc") => query.order(organization_positions::created_at.asc()),
            ("created_at", "desc") => query.order(organization_positions::created_at.desc()),
            ("name", "asc") => query.order(organization_positions::name.asc()),
            ("name", "desc") => query.order(organization_positions::name.desc()),
            _ => query.order(organization_positions::created_at.desc()),
        };

        // Add pagination
        let page = request.page.unwrap_or(1);
        let per_page = request.per_page.unwrap_or(15).min(100);
        let offset = (page - 1) * per_page;

        let organization_positions = query
            .limit(per_page as i64)
            .offset(offset as i64)
            .load::<OrganizationPosition>(&mut conn)?;

        let total = organization_positions::table.count().get_result::<i64>(&mut conn)?;

        Ok(json!({
            "data": organization_positions,
            "meta": {
                "total": total,
                "page": page,
                "per_page": per_page,
                "last_page": (total as f64 / per_page as f64).ceil() as u32
            }
        }))
    }

    pub fn show(pool: &DbPool, id: &str) -> Result<OrganizationPosition> {
        let mut conn = pool.get()?;

        let organization_position = organization_positions::table
            .filter(organization_positions::id.eq(id))
            .first::<OrganizationPosition>(&mut conn)
            .optional()?
            .ok_or_else(|| anyhow::anyhow!("Organization position not found"))?;

        Ok(organization_position)
    }

    pub fn create(pool: &DbPool, request: &CreateOrganizationPositionRequest) -> Result<OrganizationPosition> {
        let mut conn = pool.get()?;

        // Convert request to CreateOrganizationPosition model
        let create_data = CreateOrganizationPosition {
            organization_id: request.organization_id,
            organization_position_level_id: request.organization_position_level_id,
            code: request.code.clone(),
            name: request.name.clone(),
            description: request.description.clone(),
            min_salary: request.min_salary.clone(),
            max_salary: request.max_salary.clone(),
            max_incumbents: request.max_incumbents,
            qualifications: request.qualifications.clone(),
            responsibilities: request.responsibilities.clone(),
        };

        let new_position = NewOrganizationPosition::new(create_data, None); // TODO: Add created_by from auth context

        let result = diesel::insert_into(organization_positions::table)
            .values(&new_position)
            .get_result::<OrganizationPosition>(&mut conn)?;

        Ok(result)
    }

    pub fn update(pool: &DbPool, id: &str, request: &UpdateOrganizationPositionRequest) -> Result<OrganizationPosition> {
        let mut conn = pool.get()?;
        let now = Utc::now();

        // First get the current position
        let mut current = Self::show(pool, id)?;

        // Update fields if provided
        if let Some(organization_id) = request.organization_id {
            current.organization_id = organization_id;
        }
        if let Some(organization_position_level_id) = request.organization_position_level_id {
            current.organization_position_level_id = organization_position_level_id;
        }
        if let Some(code) = &request.code {
            current.code = code.clone();
        }
        if let Some(name) = &request.name {
            current.name = name.clone();
        }
        if let Some(description) = &request.description {
            current.description = description.clone();
        }
        if let Some(is_active) = request.is_active {
            current.is_active = is_active;
        }
        if let Some(min_salary) = &request.min_salary {
            current.min_salary = min_salary.clone();
        }
        if let Some(max_salary) = &request.max_salary {
            current.max_salary = max_salary.clone();
        }
        if let Some(max_incumbents) = request.max_incumbents {
            current.max_incumbents = max_incumbents;
        }
        if let Some(qualifications) = &request.qualifications {
            current.qualifications = qualifications.clone();
        }
        if let Some(responsibilities) = &request.responsibilities {
            current.responsibilities = responsibilities.clone();
        }
        current.updated_at = now;
        // TODO: Set updated_by from auth context

        let result = diesel::update(organization_positions::table.filter(organization_positions::id.eq(id)))
            .set((
                organization_positions::organization_id.eq(current.organization_id),
                organization_positions::organization_position_level_id.eq(current.organization_position_level_id),
                organization_positions::code.eq(&current.code),
                organization_positions::name.eq(&current.name),
                organization_positions::description.eq(&current.description),
                organization_positions::is_active.eq(current.is_active),
                organization_positions::min_salary.eq(&current.min_salary),
                organization_positions::max_salary.eq(&current.max_salary),
                organization_positions::max_incumbents.eq(current.max_incumbents),
                organization_positions::qualifications.eq(&current.qualifications),
                organization_positions::responsibilities.eq(&current.responsibilities),
                organization_positions::updated_at.eq(current.updated_at),
            ))
            .get_result::<OrganizationPosition>(&mut conn)?;

        Ok(result)
    }

    pub fn delete(pool: &DbPool, id: &str) -> Result<()> {
        let mut conn = pool.get()?;

        let rows_affected = diesel::delete(organization_positions::table.filter(organization_positions::id.eq(id)))
            .execute(&mut conn)?;

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("Organization position not found"));
        }

        Ok(())
    }

    pub fn activate(pool: &DbPool, id: &str) -> Result<OrganizationPosition> {
        let mut conn = pool.get()?;
        let now = Utc::now();

        let organization_position = diesel::update(organization_positions::table.filter(organization_positions::id.eq(id)))
            .set((
                organization_positions::is_active.eq(true),
                organization_positions::updated_at.eq(now),
            ))
            .get_result::<OrganizationPosition>(&mut conn)
            .optional()?
            .ok_or_else(|| anyhow::anyhow!("Organization position not found"))?;

        Ok(organization_position)
    }

    pub fn deactivate(pool: &DbPool, id: &str) -> Result<OrganizationPosition> {
        let mut conn = pool.get()?;
        let now = Utc::now();

        let organization_position = diesel::update(organization_positions::table.filter(organization_positions::id.eq(id)))
            .set((
                organization_positions::is_active.eq(false),
                organization_positions::updated_at.eq(now),
            ))
            .get_result::<OrganizationPosition>(&mut conn)
            .optional()?
            .ok_or_else(|| anyhow::anyhow!("Organization position not found"))?;

        Ok(organization_position)
    }

    pub fn by_level(pool: &DbPool, request: &OrganizationPositionsByLevelRequest) -> Result<Value> {
        let mut conn = pool.get()?;
        let include_inactive = request.include_inactive.unwrap_or(false);

        let mut query = organization_positions::table
            .filter(organization_positions::organization_position_level_id.eq(&request.organization_position_level_id))
            .into_boxed();

        if !include_inactive {
            query = query.filter(organization_positions::is_active.eq(true));
        }

        let organization_positions = query
            .order(organization_positions::created_at.desc())
            .load::<OrganizationPosition>(&mut conn)?;

        Ok(json!({
            "data": organization_positions.into_iter().map(|jp| jp.to_response()).collect::<Vec<_>>(),
        }))
    }
}