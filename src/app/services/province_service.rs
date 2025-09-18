use anyhow::Result;
use ulid::Ulid;
use sqlx::PgPool;
use crate::query_builder::QueryBuilder;

use crate::app::models::province::{Province, CreateProvince, UpdateProvince};

pub struct ProvinceService;

impl ProvinceService {
    pub async fn create(pool: &PgPool, data: CreateProvince) -> Result<Province> {
        let country_id = Ulid::from_string(&data.country_id)?;
        let province = Province::new(country_id, data.name, data.code);

        let query = r#"
            INSERT INTO provinces (id, country_id, name, code, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
        "#;

        let result = sqlx::query_as::<_, Province>(query)
            .bind(province.id.to_string())
            .bind(province.country_id.to_string())
            .bind(&province.name)
            .bind(&province.code)
            .bind(province.created_at)
            .bind(province.updated_at)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn find_by_id(pool: &PgPool, id: Ulid) -> Result<Option<Province>> {
        let query = "SELECT * FROM provinces WHERE id = $1";

        let result = sqlx::query_as::<_, Province>(query)
            .bind(id.to_string())
            .fetch_optional(pool)
            .await?;

        Ok(result)
    }

    pub async fn find_by_country_id(pool: &PgPool, country_id: Ulid) -> Result<Vec<Province>> {
        let query = "SELECT * FROM provinces WHERE country_id = $1 ORDER BY name ASC";

        let result = sqlx::query_as::<_, Province>(query)
            .bind(country_id.to_string())
            .fetch_all(pool)
            .await?;

        Ok(result)
    }

    pub async fn list(pool: &PgPool, _query_params: std::collections::HashMap<String, String>) -> Result<Vec<Province>> {
        // For now, use a simple query without the query builder to avoid SQL syntax issues
        let query = "SELECT * FROM provinces ORDER BY name ASC";
        let result = sqlx::query_as::<_, Province>(query)
            .fetch_all(pool)
            .await?;
        Ok(result)
    }

    pub async fn update(pool: &PgPool, id: Ulid, data: UpdateProvince) -> Result<Province> {
        let country_id = if let Some(country_id_str) = &data.country_id {
            Some(Ulid::from_string(country_id_str)?.to_string())
        } else {
            None
        };

        let query = r#"
            UPDATE provinces
            SET country_id = COALESCE($2, country_id),
                name = COALESCE($3, name),
                code = COALESCE($4, code),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#;

        let result = sqlx::query_as::<_, Province>(query)
            .bind(id.to_string())
            .bind(country_id)
            .bind(data.name)
            .bind(data.code)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn delete(pool: &PgPool, id: Ulid) -> Result<()> {
        let query = "DELETE FROM provinces WHERE id = $1";

        sqlx::query(query)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn count(pool: &PgPool) -> Result<i64> {
        let query = "SELECT COUNT(*) FROM provinces";

        let result: (i64,) = sqlx::query_as(query)
            .fetch_one(pool)
            .await?;

        Ok(result.0)
    }

    pub async fn count_by_country(pool: &PgPool, country_id: Ulid) -> Result<i64> {
        let query = "SELECT COUNT(*) FROM provinces WHERE country_id = $1";

        let result: (i64,) = sqlx::query_as(query)
            .bind(country_id.to_string())
            .fetch_one(pool)
            .await?;

        Ok(result.0)
    }
}