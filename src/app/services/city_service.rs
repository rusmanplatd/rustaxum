use anyhow::Result;
use ulid::Ulid;
use sqlx::PgPool;
use rust_decimal::Decimal;
use crate::query_builder::QueryBuilder;

use crate::app::models::city::{City, CreateCity, UpdateCity};

pub struct CityService;

impl CityService {
    pub async fn create(pool: &PgPool, data: CreateCity) -> Result<City> {
        let province_id = Ulid::from_string(&data.province_id)?;
        let city = City::new(province_id, data.name, data.code, data.latitude, data.longitude);

        let query = r#"
            INSERT INTO cities (id, province_id, name, code, latitude, longitude, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
        "#;

        let result = sqlx::query_as::<_, City>(query)
            .bind(city.id.to_string())
            .bind(city.province_id.to_string())
            .bind(&city.name)
            .bind(&city.code)
            .bind(city.latitude)
            .bind(city.longitude)
            .bind(city.created_at)
            .bind(city.updated_at)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn find_by_id(pool: &PgPool, id: Ulid) -> Result<Option<City>> {
        let query = "SELECT * FROM cities WHERE id = $1";

        let result = sqlx::query_as::<_, City>(query)
            .bind(id.to_string())
            .fetch_optional(pool)
            .await?;

        Ok(result)
    }

    pub async fn find_by_province_id(pool: &PgPool, province_id: Ulid) -> Result<Vec<City>> {
        let query = "SELECT * FROM cities WHERE province_id = $1 ORDER BY name ASC";

        let result = sqlx::query_as::<_, City>(query)
            .bind(province_id.to_string())
            .fetch_all(pool)
            .await?;

        Ok(result)
    }

    pub async fn find_by_coordinates(pool: &PgPool, lat: Decimal, lng: Decimal, radius_km: Decimal) -> Result<Vec<City>> {
        let query = r#"
            SELECT * FROM cities
            WHERE latitude IS NOT NULL
            AND longitude IS NOT NULL
            AND (
                6371 * acos(
                    cos(radians($1)) * cos(radians(latitude)) *
                    cos(radians(longitude) - radians($2)) +
                    sin(radians($1)) * sin(radians(latitude))
                )
            ) <= $3
            ORDER BY (
                6371 * acos(
                    cos(radians($1)) * cos(radians(latitude)) *
                    cos(radians(longitude) - radians($2)) +
                    sin(radians($1)) * sin(radians(latitude))
                )
            ) ASC
        "#;

        let result = sqlx::query_as::<_, City>(query)
            .bind(lat)
            .bind(lng)
            .bind(radius_km)
            .fetch_all(pool)
            .await?;

        Ok(result)
    }

    pub async fn list(pool: &PgPool, _query_params: std::collections::HashMap<String, String>) -> Result<Vec<City>> {
        // For now, use a simple query without the query builder to avoid SQL syntax issues
        let query = "SELECT * FROM cities ORDER BY name ASC";
        let result = sqlx::query_as::<_, City>(query)
            .fetch_all(pool)
            .await?;
        Ok(result)
    }

    pub async fn update(pool: &PgPool, id: Ulid, data: UpdateCity) -> Result<City> {
        let province_id = if let Some(province_id_str) = &data.province_id {
            Some(Ulid::from_string(province_id_str)?.to_string())
        } else {
            None
        };

        let query = r#"
            UPDATE cities
            SET province_id = COALESCE($2, province_id),
                name = COALESCE($3, name),
                code = COALESCE($4, code),
                latitude = COALESCE($5, latitude),
                longitude = COALESCE($6, longitude),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
        "#;

        let result = sqlx::query_as::<_, City>(query)
            .bind(id.to_string())
            .bind(province_id)
            .bind(data.name)
            .bind(data.code)
            .bind(data.latitude)
            .bind(data.longitude)
            .fetch_one(pool)
            .await?;

        Ok(result)
    }

    pub async fn delete(pool: &PgPool, id: Ulid) -> Result<()> {
        let query = "DELETE FROM cities WHERE id = $1";

        sqlx::query(query)
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn count(pool: &PgPool) -> Result<i64> {
        let query = "SELECT COUNT(*) FROM cities";

        let result: (i64,) = sqlx::query_as(query)
            .fetch_one(pool)
            .await?;

        Ok(result.0)
    }

    pub async fn count_by_province(pool: &PgPool, province_id: Ulid) -> Result<i64> {
        let query = "SELECT COUNT(*) FROM cities WHERE province_id = $1";

        let result: (i64,) = sqlx::query_as(query)
            .bind(province_id.to_string())
            .fetch_one(pool)
            .await?;

        Ok(result.0)
    }
}