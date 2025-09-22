use anyhow::Result;
use ulid::Ulid;
use crate::database::DbPool;
use diesel::prelude::*;
use crate::schema::provinces;

use crate::app::models::province::{Province, CreateProvince, UpdateProvince};

pub struct ProvinceService;

impl ProvinceService {
    pub fn create(pool: &DbPool, data: CreateProvince) -> Result<Province> {
        let country_id = Ulid::from_string(&data.country_id)?;
        let province = Province::new(country_id, data.name, data.code);
        let mut conn = pool.get()?;

        diesel::insert_into(provinces::table)
            .values((
                provinces::id.eq(province.id.to_string()),
                provinces::country_id.eq(province.country_id.to_string()),
                provinces::name.eq(&province.name),
                provinces::code.eq(&province.code),
                provinces::created_at.eq(province.created_at),
                provinces::updated_at.eq(province.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(province)
    }

    pub fn find_by_id(pool: &DbPool, id: Ulid) -> Result<Option<Province>> {
        let mut conn = pool.get()?;

        let result = provinces::table
            .filter(provinces::id.eq(id.to_string()))
            .first::<Province>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_country_id(pool: &DbPool, country_id: Ulid) -> Result<Vec<Province>> {
        let mut conn = pool.get()?;

        let result = provinces::table
            .filter(provinces::country_id.eq(country_id.to_string()))
            .order(provinces::name.asc())
            .load::<Province>(&mut conn)?;

        Ok(result)
    }

    pub fn list(pool: &DbPool, _query_params: std::collections::HashMap<String, String>) -> Result<Vec<Province>> {
        let mut conn = pool.get()?;

        let result = provinces::table
            .order(provinces::name.asc())
            .load::<Province>(&mut conn)?;

        Ok(result)
    }

    pub fn update(pool: &DbPool, id: String, data: UpdateProvince) -> Result<Province> {
        let mut conn = pool.get()?;

        // Get the current province
        let mut current = Self::find_by_id(pool, id)?
            .ok_or_else(|| anyhow::anyhow!("Province not found"))?;

        // Update fields if provided
        if let Some(country_id_str) = data.country_id {
            let country_id = Ulid::from_string(&country_id_str)?;
            current.country_id = country_id;
        }
        if let Some(name) = data.name {
            current.name = name;
        }
        if let Some(code) = data.code {
            current.code = Some(code);
        }
        current.updated_at = chrono::Utc::now();

        diesel::update(provinces::table.filter(provinces::id.eq(id.to_string())))
            .set((
                provinces::country_id.eq(current.country_id.to_string()),
                provinces::name.eq(&current.name),
                provinces::code.eq(&current.code),
                provinces::updated_at.eq(current.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(current)
    }

    pub fn delete(pool: &DbPool, id: Ulid) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(provinces::table.filter(provinces::id.eq(id.to_string())))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn count(pool: &DbPool) -> Result<i64> {
        let mut conn = pool.get()?;

        let result = provinces::table
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(result)
    }

    pub fn count_by_country(pool: &DbPool, country_id: Ulid) -> Result<i64> {
        let mut conn = pool.get()?;

        let result = provinces::table
            .filter(provinces::country_id.eq(country_id.to_string()))
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(result)
    }
}