use anyhow::Result;
use ulid::Ulid;
use crate::database::DbPool;

use crate::app::models::province::{Province, CreateProvince, UpdateProvince};

pub struct ProvinceService;

impl ProvinceService {
    pub fn create(pool: &DbPool, data: CreateProvince) -> Result<Province> {
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
            ?;

        Ok(result)
    }

    pub fn find_by_id(pool: &DbPool, id: Ulid) -> Result<Option<Province>> {
        let query = "SELECT * FROM provinces WHERE id = $1";

        let result = sqlx::query_as::<_, Province>(query)
            .bind(id.to_string())
            .fetch_optional(pool)
            ?;

        Ok(result)
    }

    pub fn find_by_country_id(pool: &DbPool, country_id: Ulid) -> Result<Vec<Province>> {
        let query = "SELECT * FROM provinces WHERE country_id = $1 ORDER BY name ASC";

        let result = sqlx::query_as::<_, Province>(query)
            .bind(country_id.to_string())
            .fetch_all(pool)
            ?;

        Ok(result)
    }

    pub fn list(pool: &DbPool, _query_params: std::collections::HashMap<String, String>) -> Result<Vec<Province>> {
        // For now, use a simple query without the query builder to avoid SQL syntax issues
        let query = "SELECT * FROM provinces ORDER BY name ASC";
        let result = sqlx::query_as::<_, Province>(query)
            .fetch_all(pool)
            ?;
        Ok(result)
    }

    pub fn update(pool: &DbPool, id: Ulid, data: UpdateProvince) -> Result<Province> {
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
            ?;

        Ok(result)
    }

    pub fn delete(pool: &DbPool, id: Ulid) -> Result<()> {
        let query = "DELETE FROM provinces WHERE id = $1";

        sqlx::query(query)
            .bind(id.to_string())
            .execute(pool)
            ?;

        Ok(())
    }

    pub fn count(pool: &DbPool) -> Result<i64> {
        let query = "SELECT COUNT(*) FROM provinces";

        let result: (i64,) = sqlx::query_as(query)
            .fetch_one(pool)
            ?;

        Ok(result.0)
    }

    pub fn count_by_country(pool: &DbPool, country_id: Ulid) -> Result<i64> {
        let query = "SELECT COUNT(*) FROM provinces WHERE country_id = $1";

        let result: (i64,) = sqlx::query_as(query)
            .bind(country_id.to_string())
            .fetch_one(pool)
            ?;

        Ok(result.0)
    }
}