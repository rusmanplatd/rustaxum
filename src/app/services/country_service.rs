use anyhow::Result;
use diesel::prelude::*;
use serde_json::json;
use crate::database::DbPool;
use crate::schema::ref_geo_countries;
use crate::app::models::country::{Country, CreateCountry, UpdateCountry, NewCountry};
use crate::app::traits::ServiceActivityLogger;

pub struct CountryService;

impl ServiceActivityLogger for CountryService {}

impl CountryService {
    pub async fn create(pool: &DbPool, data: CreateCountry, created_by: Option<&str>) -> Result<Country> {
        let mut conn = pool.get()?;
        let new_country = NewCountry::new(data.name.clone(), data.iso_code.clone(), data.phone_code.clone());

        let result = diesel::insert_into(ref_geo_countries::table)
            .values(&new_country)
            .get_result::<Country>(&mut conn)?;

        // Log the country creation activity
        let service = CountryService;
        let properties = json!({
            "country_name": result.name,
            "iso_code": result.iso_code,
            "phone_code": result.phone_code,
            "created_by": created_by
        });

        if let Err(e) = service.log_created(
            &result,
            created_by,
            Some(properties)
        ).await {
            eprintln!("Failed to log country creation activity: {}", e);
        }

        Ok(result)
    }

    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<Country>> {
        let mut conn = pool.get()?;

        let result = ref_geo_countries::table
            .filter(ref_geo_countries::id.eq(id.to_string()))
            .first::<Country>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_iso_code(pool: &DbPool, iso_code: &str) -> Result<Option<Country>> {
        let mut conn = pool.get()?;

        let result = ref_geo_countries::table
            .filter(ref_geo_countries::iso_code.eq(iso_code))
            .first::<Country>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn list(pool: &DbPool, _query_params: std::collections::HashMap<String, String>) -> Result<Vec<Country>> {
        let mut conn = pool.get()?;

        let result = ref_geo_countries::table
            .order(ref_geo_countries::name.asc())
            .load::<Country>(&mut conn)?;
        Ok(result)
    }

    pub fn update(pool: &DbPool, id: String, data: UpdateCountry) -> Result<Country> {
        let mut conn = pool.get()?;

        let result = diesel::update(ref_geo_countries::table.filter(ref_geo_countries::id.eq(id.to_string())))
            .set((
                data.name.map(|n| ref_geo_countries::name.eq(n)),
                data.iso_code.map(|c| ref_geo_countries::iso_code.eq(c)),
                data.phone_code.map(|p| ref_geo_countries::phone_code.eq(p)),
                ref_geo_countries::updated_at.eq(chrono::Utc::now()),
            ))
            .get_result::<Country>(&mut conn)?;

        Ok(result)
    }

    pub fn delete(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(ref_geo_countries::table.filter(ref_geo_countries::id.eq(id.to_string())))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn count(pool: &DbPool) -> Result<i64> {
        let mut conn = pool.get()?;

        let result = ref_geo_countries::table.count().get_result::<i64>(&mut conn)?;

        Ok(result)
    }
}