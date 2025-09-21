use anyhow::Result;
use crate::database::DbPool;
use ulid::Ulid;
use chrono::Utc;
use serde_json::{json, Value};
use diesel::prelude::*;
use crate::schema::organization_positions;

use crate::app::models::jobposition::JobPosition;
use crate::app::http::requests::organization_position_requests::{
    CreateJobPositionRequest, UpdateJobPositionRequest, IndexJobPositionRequest, JobPositionsByLevelRequest
};

pub struct JobPositionService;

impl JobPositionService {
    pub fn index(pool: &DbPool, request: &IndexJobPositionRequest) -> Result<Value> {
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
            .load::<JobPosition>(&mut conn)?;

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

    pub fn show(pool: &DbPool, id: &str) -> Result<JobPosition> {
        let mut conn = pool.get()?;

        let organization_position = organization_positions::table
            .filter(organization_positions::id.eq(id))
            .first::<JobPosition>(&mut conn)
            .optional()?
            .ok_or_else(|| anyhow::anyhow!("Organization position not found"))?;

        Ok(organization_position)
    }

    pub fn create(pool: &DbPool, request: &CreateJobPositionRequest) -> Result<JobPosition> {
        let mut conn = pool.get()?;
        let id = Ulid::new();
        let now = Utc::now();

        let organization_position = diesel::insert_into(organization_positions::table)
            .values((
                organization_positions::id.eq(id.to_string()),
                organization_positions::name.eq(&request.name),
                organization_positions::code.eq(&request.code),
                organization_positions::organization_position_level_id.eq(&request.organization_position_level_id),
                organization_positions::description.eq(&request.description),
                organization_positions::is_active.eq(true),
                organization_positions::created_at.eq(now),
                organization_positions::updated_at.eq(now),
            ))
            .get_result::<JobPosition>(&mut conn)?;

        Ok(organization_position)
    }

    pub fn update(pool: &DbPool, id: &str, request: &UpdateJobPositionRequest) -> Result<JobPosition> {
        let mut conn = pool.get()?;
        let now = Utc::now();

        let mut changeset = organization_positions::table.filter(organization_positions::id.eq(id));

        let organization_position = diesel::update(changeset)
            .set((
                organization_positions::name.eq(request.name.as_ref().unwrap_or(&"".to_string())),
                organization_positions::code.eq(request.code.as_ref().unwrap_or(&"".to_string())),
                organization_positions::organization_position_level_id.eq(request.organization_position_level_id.as_ref().unwrap_or(&"".to_string())),
                organization_positions::description.eq(request.description.as_ref().unwrap_or(&"".to_string())),
                organization_positions::is_active.eq(request.is_active.unwrap_or(true)),
                organization_positions::updated_at.eq(now),
            ))
            .get_result::<JobPosition>(&mut conn)
            .optional()?
            .ok_or_else(|| anyhow::anyhow!("Organization position not found"))?;

        Ok(organization_position)
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

    pub fn activate(pool: &DbPool, id: &str) -> Result<JobPosition> {
        let mut conn = pool.get()?;
        let now = Utc::now();

        let organization_position = diesel::update(organization_positions::table.filter(organization_positions::id.eq(id)))
            .set((
                organization_positions::is_active.eq(true),
                organization_positions::updated_at.eq(now),
            ))
            .get_result::<JobPosition>(&mut conn)
            .optional()?
            .ok_or_else(|| anyhow::anyhow!("Organization position not found"))?;

        Ok(organization_position)
    }

    pub fn deactivate(pool: &DbPool, id: &str) -> Result<JobPosition> {
        let mut conn = pool.get()?;
        let now = Utc::now();

        let organization_position = diesel::update(organization_positions::table.filter(organization_positions::id.eq(id)))
            .set((
                organization_positions::is_active.eq(false),
                organization_positions::updated_at.eq(now),
            ))
            .get_result::<JobPosition>(&mut conn)
            .optional()?
            .ok_or_else(|| anyhow::anyhow!("Organization position not found"))?;

        Ok(organization_position)
    }

    pub fn by_level(pool: &DbPool, request: &JobPositionsByLevelRequest) -> Result<Value> {
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
            .load::<JobPosition>(&mut conn)?;

        Ok(json!({
            "data": organization_positions.into_iter().map(|jp| jp.to_response()).collect::<Vec<_>>(),
        }))
    }
}