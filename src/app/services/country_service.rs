use anyhow::Result;
use ulid::Ulid;
use sqlx::PgPool;
use crate::query_builder::QueryBuilder;

use crate::app::models::country::{Country, CreateCountry, UpdateCountry};

pub struct CountryService;

impl CountryService {
    pub async fn create(pool: &PgPool, data: CreateCountry) -> Result<Country> {
        let country = Country::new(data.name, data.iso_code, data.phone_code);

        let query = r#"
            INSERT INTO countries (id, name, iso_code, phone_code, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
        "#;

        let result = sqlx::query_as::<_, Country>(query)
            .bind(country.id.to_string())
            .bind(&country.name)
            .bind(&country.iso_code)
            .bind(&country.phone_code)
            .bind(country.created_at)
            .bind(country.updated_at)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn find_by_id(pool: &PgPool, id: Ulid) -> Result<Option<Country>> {
        let query = "SELECT * FROM countries WHERE id = $1";

        let result = sqlx::query_as::<_, Country>(query)
            .bind(id.to_string())
            .fetch_optional(pool)
            .await?;

        Ok(result)
    }

    pub async fn find_by_iso_code(pool: &PgPool, iso_code: &str) -> Result<Option<Country>> {
        let query = "SELECT * FROM countries WHERE iso_code = $1";

        let result = sqlx::query_as::<_, Country>(query)
            .bind(iso_code)
            .fetch_optional(pool)
            .await?;

        Ok(result)
    }

    pub async fn list(pool: &PgPool, _query_params: std::collections::HashMap<String, String>) -> Result<Vec<Country>> {
        // For now, use a simple query without the query builder to avoid SQL syntax issues
        let query = "SELECT * FROM countries ORDER BY name ASC";
        let result = sqlx::query_as::<_, Country>(query)
            .fetch_all(pool)
            .await?;
        Ok(result)
    }

    pub async fn update(pool: &PgPool, id: Ulid, data: UpdateCountry) -> Result<Country> {
        let query = r#"
            UPDATE countries
            SET name = COALESCE($2, name),
                iso_code = COALESCE($3, iso_code),
                phone_code = COALESCE($4, phone_code),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#;

        let result = sqlx::query_as::<_, Country>(query)
            .bind(id.to_string())
            .bind(data.name)
            .bind(data.iso_code)
            .bind(data.phone_code)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn delete(pool: &PgPool, id: Ulid) -> Result<()> {
        let query = "DELETE FROM countries WHERE id = $1";

        sqlx::query(query)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn count(pool: &PgPool) -> Result<i64> {
        let query = "SELECT COUNT(*) FROM countries";

        let result: (i64,) = sqlx::query_as(query)
            .fetch_one(pool)
            .await?;

        Ok(result.0)
    }
}