use anyhow::Result;
use ulid::Ulid;
use crate::database::DbPool;
use std::collections::HashMap;
use diesel::prelude::*;
use crate::schema::job_levels;

use crate::app::models::joblevel::{JobLevel, CreateJobLevel, UpdateJobLevel};

pub struct JobLevelService;

impl JobLevelService {
    pub fn create(pool: &DbPool, data: CreateJobLevel) -> Result<JobLevel> {
        let mut conn = pool.get()?;
        let job_level = JobLevel::new(data.name, data.code, data.level, data.description);

        diesel::insert_into(job_levels::table)
            .values((
                job_levels::id.eq(job_level.id.to_string()),
                job_levels::name.eq(&job_level.name),
                job_levels::code.eq(&job_level.code),
                job_levels::level.eq(job_level.level),
                job_levels::description.eq(&job_level.description),
                job_levels::is_active.eq(job_level.is_active),
                job_levels::created_at.eq(job_level.created_at),
                job_levels::updated_at.eq(job_level.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(job_level)
    }

    pub fn find_by_id(pool: &DbPool, id: Ulid) -> Result<Option<JobLevel>> {
        let mut conn = pool.get()?;

        let result = job_levels::table
            .filter(job_levels::id.eq(id.to_string()))
            .first::<JobLevel>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_level(pool: &DbPool, level: i32) -> Result<Option<JobLevel>> {
        let mut conn = pool.get()?;

        let result = job_levels::table
            .filter(job_levels::level.eq(level))
            .first::<JobLevel>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn list(pool: &DbPool, _query_params: HashMap<String, String>) -> Result<Vec<JobLevel>> {
        let mut conn = pool.get()?;

        let result = job_levels::table
            .order(job_levels::level.asc())
            .load::<JobLevel>(&mut conn)?;

        Ok(result)
    }

    pub fn update(pool: &DbPool, id: Ulid, data: UpdateJobLevel) -> Result<JobLevel> {
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

        diesel::update(job_levels::table.filter(job_levels::id.eq(id.to_string())))
            .set((
                job_levels::name.eq(&current.name),
                job_levels::code.eq(&current.code),
                job_levels::level.eq(current.level),
                job_levels::description.eq(&current.description),
                job_levels::is_active.eq(current.is_active),
                job_levels::updated_at.eq(current.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(current)
    }

    pub fn delete(pool: &DbPool, id: Ulid) -> Result<()> {
        let mut conn = pool.get()?;

        let rows_affected = diesel::delete(job_levels::table.filter(job_levels::id.eq(id.to_string())))
            .execute(&mut conn)?;

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("Job level not found"));
        }

        Ok(())
    }

    pub fn find_active_levels(pool: &DbPool) -> Result<Vec<JobLevel>> {
        let query = "SELECT * FROM job_levels WHERE is_active = true ORDER BY level ASC";

        let results = sqlx::query_as::<_, JobLevel>(query)
            .fetch_all(pool)
            ?;

        Ok(results)
    }

    pub fn find_by_level_range(pool: &DbPool, min_level: i32, max_level: i32) -> Result<Vec<JobLevel>> {
        let query = "SELECT * FROM job_levels WHERE level >= $1 AND level <= $2 ORDER BY level ASC";

        let results = sqlx::query_as::<_, JobLevel>(query)
            .bind(min_level)
            .bind(max_level)
            .fetch_all(pool)
            ?;

        Ok(results)
    }
}