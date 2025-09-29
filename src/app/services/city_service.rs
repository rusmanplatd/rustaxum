use anyhow::Result;
use crate::database::DbPool;
use rust_decimal::Decimal;
use diesel::prelude::*;
use diesel::sql_types::Numeric;
use crate::schema::ref_geo_cities;

use crate::app::models::city::{City, CreateCity, UpdateCity};

pub struct CityService;

impl CityService {
    pub fn create(pool: &DbPool, data: CreateCity) -> Result<City> {
        let city = City::new(data.province_id, data.name, data.code, data.latitude, data.longitude);
        let mut conn = pool.get()?;

        diesel::insert_into(ref_geo_cities::table)
            .values((
                ref_geo_cities::id.eq(&city.id),
                ref_geo_cities::province_id.eq(&city.province_id),
                ref_geo_cities::name.eq(&city.name),
                ref_geo_cities::code.eq(&city.code),
                ref_geo_cities::latitude.eq(city.latitude),
                ref_geo_cities::longitude.eq(city.longitude),
                ref_geo_cities::created_at.eq(city.created_at),
                ref_geo_cities::updated_at.eq(city.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(city)
    }

    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<City>> {
        let mut conn = pool.get()?;

        let result = ref_geo_cities::table
            .filter(ref_geo_cities::id.eq(id.to_string()))
            .first::<City>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_province_id(pool: &DbPool, province_id: String) -> Result<Vec<City>> {
        let mut conn = pool.get()?;

        let result = ref_geo_cities::table
            .filter(ref_geo_cities::province_id.eq(province_id.to_string()))
            .order(ref_geo_cities::name.asc())
            .load::<City>(&mut conn)?;

        Ok(result)
    }

    pub fn find_by_coordinates(pool: &DbPool, lat: Decimal, lng: Decimal, radius_km: Decimal) -> Result<Vec<City>> {
        let mut conn = pool.get()?;

        // Define the distance calculation as a custom SQL function
        diesel::define_sql_function!(fn haversine_distance(lat1: Numeric, lng1: Numeric, lat2: Numeric, lng2: Numeric) -> Numeric);

        let result = ref_geo_cities::table
            .filter(ref_geo_cities::latitude.is_not_null())
            .filter(ref_geo_cities::longitude.is_not_null())
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

        let result = ref_geo_cities::table
            .order(ref_geo_cities::name.asc())
            .load::<City>(&mut conn)?;

        Ok(result)
    }

    pub fn update(pool: &DbPool, id: String, data: UpdateCity) -> Result<City> {
        let mut conn = pool.get()?;

        // Get the current city
        let mut current = Self::find_by_id(pool, id.clone())?
            .ok_or_else(|| anyhow::anyhow!("City not found"))?;

        // Update fields if provided
        if let Some(province_id) = data.province_id {
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

        diesel::update(ref_geo_cities::table.filter(ref_geo_cities::id.eq(&id)))
            .set((
                ref_geo_cities::province_id.eq(&current.province_id),
                ref_geo_cities::name.eq(&current.name),
                ref_geo_cities::code.eq(&current.code),
                ref_geo_cities::latitude.eq(current.latitude),
                ref_geo_cities::longitude.eq(current.longitude),
                ref_geo_cities::updated_at.eq(current.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(current)
    }

    pub fn delete(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(ref_geo_cities::table.filter(ref_geo_cities::id.eq(id.to_string())))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn count(pool: &DbPool) -> Result<i64> {
        let mut conn = pool.get()?;

        let result = ref_geo_cities::table
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(result)
    }

    pub fn count_by_province(pool: &DbPool, province_id: String) -> Result<i64> {
        let mut conn = pool.get()?;

        let result = ref_geo_cities::table
            .filter(ref_geo_cities::province_id.eq(province_id.to_string()))
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(result)
    }
}