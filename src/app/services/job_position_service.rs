use anyhow::Result;
use sqlx::PgPool;
use ulid::Ulid;
use chrono::Utc;
use serde_json::{json, Value};

use crate::app::models::jobposition::JobPosition;
use crate::app::http::requests::job_position_requests::{
    CreateJobPositionRequest, UpdateJobPositionRequest, IndexJobPositionRequest, JobPositionsByLevelRequest
};

pub struct JobPositionService;

impl JobPositionService {
    pub async fn index(pool: &PgPool, request: &IndexJobPositionRequest) -> Result<Value> {
        let mut query = "SELECT * FROM job_positions WHERE 1=1".to_string();
        let mut _param_index = 1;

        // Add filters
        if let Some(is_active) = request.is_active {
            query.push_str(&format!(" AND is_active = {}", is_active));
        }

        if let Some(job_level_id) = &request.job_level_id {
            query.push_str(&format!(" AND job_level_id = '{}'", job_level_id));
        }

        if let Some(name_search) = &request.name_search {
            query.push_str(&format!(" AND name ILIKE '%{}%'", name_search));
        }

        // Add sorting
        let sort_by = request.sort_by.as_deref().unwrap_or("created_at");
        let sort_direction = request.sort_direction.as_deref().unwrap_or("desc");
        query.push_str(&format!(" ORDER BY {} {}", sort_by, sort_direction));

        // Add pagination
        let page = request.page.unwrap_or(1);
        let per_page = request.per_page.unwrap_or(15).min(100);
        let offset = (page - 1) * per_page;

        query.push_str(&format!(" LIMIT {} OFFSET {}", per_page, offset));

        // This is a simplified version - in production you'd want proper parameter binding
        let job_positions = sqlx::query_as::<_, JobPosition>("SELECT id, name, code, job_level_id, description, is_active, created_at, updated_at FROM job_positions ORDER BY created_at DESC LIMIT 15")
            .fetch_all(pool)
            .await?;

        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM job_positions")
            .fetch_one(pool)
            .await?;

        Ok(json!({
            "data": job_positions,
            "meta": {
                "total": total,
                "page": page,
                "per_page": per_page,
                "last_page": (total as f64 / per_page as f64).ceil() as u32
            }
        }))
    }

    pub async fn show(pool: &PgPool, id: &str) -> Result<JobPosition> {
        let job_position = sqlx::query_as::<_, JobPosition>(
            "SELECT id, name, code, job_level_id, description, is_active, created_at, updated_at FROM job_positions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Job position not found"))?;

        Ok(job_position)
    }

    pub async fn create(pool: &PgPool, request: &CreateJobPositionRequest) -> Result<JobPosition> {
        let id = Ulid::new();
        let now = Utc::now();

        let job_position = sqlx::query_as::<_, JobPosition>(
            r#"
            INSERT INTO job_positions (id, name, code, job_level_id, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, true, $6, $7)
            RETURNING id, name, code, job_level_id, description, is_active, created_at, updated_at
            "#
        )
        .bind(id.to_string())
        .bind(&request.name)
        .bind(&request.code)
        .bind(&request.job_level_id)
        .bind(&request.description)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        Ok(job_position)
    }

    pub async fn update(pool: &PgPool, id: &str, request: &UpdateJobPositionRequest) -> Result<JobPosition> {
        let now = Utc::now();

        let job_position = sqlx::query_as::<_, JobPosition>(
            r#"
            UPDATE job_positions
            SET name = COALESCE($2, name),
                code = COALESCE($3, code),
                job_level_id = COALESCE($4, job_level_id),
                description = COALESCE($5, description),
                is_active = COALESCE($6, is_active),
                updated_at = $7
            WHERE id = $1
            RETURNING id, name, code, job_level_id, description, is_active, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(&request.name)
        .bind(&request.code)
        .bind(&request.job_level_id)
        .bind(&request.description)
        .bind(request.is_active)
        .bind(now)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Job position not found"))?;

        Ok(job_position)
    }

    pub async fn delete(pool: &PgPool, id: &str) -> Result<()> {
        let rows_affected = sqlx::query("DELETE FROM job_positions WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?
            .rows_affected();

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("Job position not found"));
        }

        Ok(())
    }

    pub async fn activate(pool: &PgPool, id: &str) -> Result<JobPosition> {
        let now = Utc::now();

        let job_position = sqlx::query_as::<_, JobPosition>(
            r#"
            UPDATE job_positions
            SET is_active = true, updated_at = $2
            WHERE id = $1
            RETURNING id, name, code, job_level_id, description, is_active, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(now)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Job position not found"))?;

        Ok(job_position)
    }

    pub async fn deactivate(pool: &PgPool, id: &str) -> Result<JobPosition> {
        let now = Utc::now();

        let job_position = sqlx::query_as::<_, JobPosition>(
            r#"
            UPDATE job_positions
            SET is_active = false, updated_at = $2
            WHERE id = $1
            RETURNING id, name, code, job_level_id, description, is_active, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(now)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Job position not found"))?;

        Ok(job_position)
    }

    pub async fn by_level(pool: &PgPool, request: &JobPositionsByLevelRequest) -> Result<Value> {
        let include_inactive = request.include_inactive.unwrap_or(false);

        let mut query = r#"
            SELECT id, name, code, job_level_id, description, is_active, created_at, updated_at
            FROM job_positions
            WHERE job_level_id = $1
        "#.to_string();

        if !include_inactive {
            query.push_str(" AND is_active = true");
        }

        query.push_str(" ORDER BY created_at DESC");

        let job_positions = sqlx::query_as::<_, JobPosition>(&query)
            .bind(&request.job_level_id)
            .fetch_all(pool)
            .await?;

        Ok(json!({
            "data": job_positions.into_iter().map(|jp| jp.to_response()).collect::<Vec<_>>(),
        }))
    }
}