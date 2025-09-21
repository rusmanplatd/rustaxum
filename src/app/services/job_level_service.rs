use anyhow::Result;
use ulid::Ulid;
use crate::database::DbPool;
use std::collections::HashMap;

use crate::app::models::joblevel::{JobLevel, CreateJobLevel, UpdateJobLevel};

pub struct JobLevelService;

impl JobLevelService {
    pub fn create(pool: &DbPool, data: CreateJobLevel) -> Result<JobLevel> {
        let job_level = JobLevel::new(data.name, data.code, data.level, data.description);

        let query = r#"
            INSERT INTO job_levels (id, name, code, level, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
        "#;

        let result = sqlx::query_as::<_, JobLevel>(query)
            .bind(job_level.id.to_string())
            .bind(&job_level.name)
            .bind(&job_level.code)
            .bind(job_level.level)
            .bind(&job_level.description)
            .bind(job_level.is_active)
            .bind(job_level.created_at)
            .bind(job_level.updated_at)
            .fetch_one(pool)
            ?;

        Ok(result)
    }

    pub fn find_by_id(pool: &DbPool, id: Ulid) -> Result<Option<JobLevel>> {
        let query = "SELECT * FROM job_levels WHERE id = $1";

        let result = sqlx::query_as::<_, JobLevel>(query)
            .bind(id.to_string())
            .fetch_optional(pool)
            ?;

        Ok(result)
    }

    pub fn find_by_level(pool: &DbPool, level: i32) -> Result<Option<JobLevel>> {
        let query = "SELECT * FROM job_levels WHERE level = $1";

        let result = sqlx::query_as::<_, JobLevel>(query)
            .bind(level)
            .fetch_optional(pool)
            ?;

        Ok(result)
    }

    pub fn list(pool: &DbPool, _query_params: HashMap<String, String>) -> Result<Vec<JobLevel>> {
        // For now, use a simple query without the query builder to avoid SQL syntax issues
        let query = "SELECT * FROM job_levels ORDER BY level ASC";
        let result = sqlx::query_as::<_, JobLevel>(query)
            .fetch_all(pool)
            ?;
        Ok(result)
    }

    pub fn update(pool: &DbPool, id: Ulid, data: UpdateJobLevel) -> Result<JobLevel> {
        let current = Self::find_by_id(pool, id)?
            .ok_or_else(|| anyhow::anyhow!("Job level not found"))?;

        let query = r#"
            UPDATE job_levels
            SET name = $2, code = $3, level = $4, description = $5, is_active = $6, updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#;

        let result = sqlx::query_as::<_, JobLevel>(query)
            .bind(id.to_string())
            .bind(data.name.unwrap_or(current.name))
            .bind(data.code.or(current.code))
            .bind(data.level.unwrap_or(current.level))
            .bind(data.description.or(current.description))
            .bind(data.is_active.unwrap_or(current.is_active))
            .fetch_one(pool)
            ?;

        Ok(result)
    }

    pub fn delete(pool: &DbPool, id: Ulid) -> Result<()> {
        let query = "DELETE FROM job_levels WHERE id = $1";

        let result = sqlx::query(query)
            .bind(id.to_string())
            .execute(pool)
            ?;

        if result.rows_affected() == 0 {
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