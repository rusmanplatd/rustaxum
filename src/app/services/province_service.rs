use anyhow::Result;
use ulid::Ulid;
use crate::database::DbPool;
use diesel::prelude::*;
use crate::schema::ref_geo_provinces;

use crate::app::models::province::{Province, CreateProvince, UpdateProvince};

pub struct ProvinceService;

impl ProvinceService {
    pub fn create(pool: &DbPool, data: CreateProvince, created_by: &str) -> Result<Province> {
        let country_id = Ulid::from_string(&data.country_id)?;
        let province = Province::new(country_id.to_string(), data.name, data.code, created_by);
        let mut conn = pool.get()?;

        let result = diesel::insert_into(ref_geo_provinces::table)
            .values(&province)
            .returning(Province::as_select())
            .get_result::<Province>(&mut conn)?;

        Ok(result)
    }

    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<Province>> {
        let mut conn = pool.get()?;

        let result = ref_geo_provinces::table
            .filter(ref_geo_provinces::id.eq(id))
            .select(Province::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_country_id(pool: &DbPool, country_id: String) -> Result<Vec<Province>> {
        let mut conn = pool.get()?;

        let result = ref_geo_provinces::table
            .filter(ref_geo_provinces::country_id.eq(country_id))
            .order(ref_geo_provinces::name.asc())
            .select(Province::as_select())
            .load(&mut conn)?;

        Ok(result)
    }

    pub fn list(pool: &DbPool, _query_params: std::collections::HashMap<String, String>) -> Result<Vec<Province>> {
        let mut conn = pool.get()?;

        let result = ref_geo_provinces::table
            .order(ref_geo_provinces::name.asc())
            .select(Province::as_select())
            .load(&mut conn)?;

        Ok(result)
    }

    pub fn update(pool: &DbPool, id: String, data: UpdateProvince) -> Result<Province> {
        let mut conn = pool.get()?;

        // Get the current province
        let mut current = Self::find_by_id(pool, id.clone())?
            .ok_or_else(|| anyhow::anyhow!("Province not found"))?;

        // Update fields if provided
        if let Some(country_id_str) = data.country_id {
            let country_id = Ulid::from_string(&country_id_str)?;
            current.country_id = country_id.to_string();
        }
        if let Some(name) = data.name {
            current.name = name;
        }
        if let Some(code) = data.code {
            current.code = Some(code);
        }
        current.updated_at = chrono::Utc::now();

        let result = diesel::update(ref_geo_provinces::table.filter(ref_geo_provinces::id.eq(id)))
            .set((
                ref_geo_provinces::country_id.eq(&current.country_id),
                ref_geo_provinces::name.eq(&current.name),
                ref_geo_provinces::code.eq(&current.code),
                ref_geo_provinces::updated_at.eq(current.updated_at),
            ))
            .returning(Province::as_select())
            .get_result(&mut conn)?;

        Ok(result)
    }

    pub fn delete(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(ref_geo_provinces::table.filter(ref_geo_provinces::id.eq(id.to_string())))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn count(pool: &DbPool) -> Result<i64> {
        let mut conn = pool.get()?;

        let result = ref_geo_provinces::table
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(result)
    }

    pub fn count_by_country(pool: &DbPool, country_id: String) -> Result<i64> {
        let mut conn = pool.get()?;

        let result = ref_geo_provinces::table
            .filter(ref_geo_provinces::country_id.eq(country_id))
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(result)
    }
}