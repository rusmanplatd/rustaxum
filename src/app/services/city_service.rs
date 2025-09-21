use anyhow::Result;
use ulid::Ulid;
use crate::database::DbPool;
use rust_decimal::Decimal;
use diesel::prelude::*;
use diesel::sql_types::Numeric;
use crate::schema::cities;

use crate::app::models::city::{City, CreateCity, UpdateCity};

pub struct CityService;

impl CityService {
    pub fn create(pool: &DbPool, data: CreateCity) -> Result<City> {
        let province_id = Ulid::from_string(&data.province_id)?;
        let city = City::new(province_id, data.name, data.code, data.latitude, data.longitude);
        let mut conn = pool.get()?;

        diesel::insert_into(cities::table)
            .values((
                cities::id.eq(city.id.to_string()),
                cities::province_id.eq(city.province_id.to_string()),
                cities::name.eq(&city.name),
                cities::code.eq(&city.code),
                cities::latitude.eq(city.latitude),
                cities::longitude.eq(city.longitude),
                cities::created_at.eq(city.created_at),
                cities::updated_at.eq(city.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(city)
    }

    pub fn find_by_id(pool: &DbPool, id: Ulid) -> Result<Option<City>> {
        let mut conn = pool.get()?;

        let result = cities::table
            .filter(cities::id.eq(id.to_string()))
            .first::<City>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_province_id(pool: &DbPool, province_id: Ulid) -> Result<Vec<City>> {
        let mut conn = pool.get()?;

        let result = cities::table
            .filter(cities::province_id.eq(province_id.to_string()))
            .order(cities::name.asc())
            .load::<City>(&mut conn)?;

        Ok(result)
    }

    pub fn find_by_coordinates(pool: &DbPool, lat: Decimal, lng: Decimal, radius_km: Decimal) -> Result<Vec<City>> {
        let mut conn = pool.get()?;

        // Define the distance calculation as a custom SQL function
        diesel::define_sql_function!(fn haversine_distance(lat1: Numeric, lng1: Numeric, lat2: Numeric, lng2: Numeric) -> Numeric);

        let result = cities::table
            .filter(cities::latitude.is_not_null())
            .filter(cities::longitude.is_not_null())
            .filter(
                diesel::dsl::sql::<diesel::sql_types::Numeric>(
                    "6371 * acos(cos(radians("
                )
                .bind::<diesel::sql_types::Numeric, _>(lat)
                .sql(")) * cos(radians(latitude)) * cos(radians(longitude) - radians(")
                .bind::<diesel::sql_types::Numeric, _>(lng)
                .sql(")) + sin(radians(")
                .bind::<diesel::sql_types::Numeric, _>(lat)
                .sql(")) * sin(radians(latitude)))")
                .le(radius_km)
            )
            .order_by(
                diesel::dsl::sql::<diesel::sql_types::Numeric>(
                    "6371 * acos(cos(radians("
                )
                .bind::<diesel::sql_types::Numeric, _>(lat)
                .sql(")) * cos(radians(latitude)) * cos(radians(longitude) - radians(")
                .bind::<diesel::sql_types::Numeric, _>(lng)
                .sql(")) + sin(radians(")
                .bind::<diesel::sql_types::Numeric, _>(lat)
                .sql(")) * sin(radians(latitude)))")
            )
            .load::<City>(&mut conn)?;

        Ok(result)
    }

    pub fn list(pool: &DbPool, _query_params: std::collections::HashMap<String, String>) -> Result<Vec<City>> {
        let mut conn = pool.get()?;

        let result = cities::table
            .order(cities::name.asc())
            .load::<City>(&mut conn)?;

        Ok(result)
    }

    pub fn update(pool: &DbPool, id: Ulid, data: UpdateCity) -> Result<City> {
        let mut conn = pool.get()?;

        // Get the current city
        let mut current = Self::find_by_id(pool, id)?
            .ok_or_else(|| anyhow::anyhow!("City not found"))?;

        // Update fields if provided
        if let Some(province_id_str) = data.province_id {
            let province_id = Ulid::from_string(&province_id_str)?;
            current.province_id = province_id;
        }
        if let Some(name) = data.name {
            current.name = name;
        }
        if let Some(code) = data.code {
            current.code = Some(code);
        }
        if let Some(latitude) = data.latitude {
            current.latitude = Some(latitude);
        }
        if let Some(longitude) = data.longitude {
            current.longitude = Some(longitude);
        }
        current.updated_at = chrono::Utc::now();

        diesel::update(cities::table.filter(cities::id.eq(id.to_string())))
            .set((
                cities::province_id.eq(current.province_id.to_string()),
                cities::name.eq(&current.name),
                cities::code.eq(&current.code),
                cities::latitude.eq(current.latitude),
                cities::longitude.eq(current.longitude),
                cities::updated_at.eq(current.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(current)
    }

    pub fn delete(pool: &DbPool, id: Ulid) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(cities::table.filter(cities::id.eq(id.to_string())))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn count(pool: &DbPool) -> Result<i64> {
        let mut conn = pool.get()?;

        let result = cities::table
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(result)
    }

    pub fn count_by_province(pool: &DbPool, province_id: Ulid) -> Result<i64> {
        let mut conn = pool.get()?;

        let result = cities::table
            .filter(cities::province_id.eq(province_id.to_string()))
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(result)
    }
}