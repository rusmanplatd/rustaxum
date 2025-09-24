use anyhow::Result;
use diesel::prelude::*;
use serde_json::json;
use crate::database::DbPool;
use crate::schema::districts;
use crate::app::models::district::{District, CreateDistrict, UpdateDistrict, NewDistrict};
use crate::app::traits::ServiceActivityLogger;

pub struct DistrictService;

impl ServiceActivityLogger for DistrictService {}

impl DistrictService {
    pub async fn create(pool: &DbPool, data: CreateDistrict, created_by: Option<&str>) -> Result<District> {
        let mut conn = pool.get()?;
        let new_district = NewDistrict::new(data.city_id.clone(), data.name.clone(), data.code.clone());

        let result = diesel::insert_into(districts::table)
            .values(&new_district)
            .get_result::<District>(&mut conn)?;

        // Log the district creation activity
        let service = DistrictService;
        let properties = json!({
            "district_name": result.name,
            "city_id": result.city_id,
            "code": result.code,
            "created_by": created_by
        });

        if let Err(e) = service.log_created(
            &result,
            created_by,
            Some(properties)
        ).await {
            eprintln!("Failed to log district creation activity: {}", e);
        }

        Ok(result)
    }

    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<District>> {
        let mut conn = pool.get()?;

        let result = districts::table
            .filter(districts::id.eq(id.to_string()))
            .first::<District>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn update(pool: &DbPool, id: String, data: UpdateDistrict) -> Result<District> {
        let mut conn = pool.get()?;

        let result = diesel::update(districts::table.filter(districts::id.eq(id.to_string())))
            .set((
                data.city_id.map(|c| districts::city_id.eq(c)),
                data.name.map(|n| districts::name.eq(n)),
                data.code.map(|c| districts::code.eq(c)),
                districts::updated_at.eq(diesel::dsl::now),
            ))
            .get_result::<District>(&mut conn)?;

        Ok(result)
    }

    pub fn delete(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(districts::table.filter(districts::id.eq(id.to_string())))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn find_by_city_id(pool: &DbPool, city_id: String) -> Result<Vec<District>> {
        let mut conn = pool.get()?;

        let result = districts::table
            .filter(districts::city_id.eq(city_id))
            .filter(districts::deleted_at.is_null())
            .order(districts::name.asc())
            .load::<District>(&mut conn)?;

        Ok(result)
    }

    pub fn list_all(pool: &DbPool) -> Result<Vec<District>> {
        let mut conn = pool.get()?;

        let result = districts::table
            .filter(districts::deleted_at.is_null())
            .order(districts::name.asc())
            .load::<District>(&mut conn)?;

        Ok(result)
    }
}