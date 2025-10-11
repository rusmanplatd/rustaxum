use anyhow::Result;
use diesel::prelude::*;
use serde_json::json;
use rust_decimal::Decimal;
use crate::database::DbPool;
use crate::schema::ref_geo_villages;
use crate::app::models::village::{Village, CreateVillage, UpdateVillage};
use crate::app::traits::ServiceActivityLogger;

pub struct VillageService;

impl ServiceActivityLogger for VillageService {}

impl VillageService {
    pub async fn create(pool: &DbPool, data: CreateVillage, created_by: &str) -> Result<Village> {
        let mut conn = pool.get()?;
        let new_village = Village::new(
            data.district_id.clone(),
            data.name.clone(),
            data.code.clone(),
            data.latitude,
            data.longitude,
            created_by,
        );

        let result = diesel::insert_into(ref_geo_villages::table)
            .values(&new_village)
            .returning(Village::as_select())
            .get_result(&mut conn)?;

        // Log the village creation activity
        let service = VillageService;
        let properties = json!({
            "village_name": result.name,
            "district_id": result.district_id,
            "code": result.code,
            "latitude": result.latitude,
            "longitude": result.longitude,
            "created_by": created_by
        });

        if let Err(e) = service.log_created(
            &result,
            Some(created_by),
            Some(properties)
        ).await {
            eprintln!("Failed to log village creation activity: {}", e);
        }

        Ok(result)
    }

    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<Village>> {
        let mut conn = pool.get()?;

        let result = ref_geo_villages::table
            .filter(ref_geo_villages::id.eq(id.to_string()))
            .select(Village::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn update(pool: &DbPool, id: String, data: UpdateVillage) -> Result<Village> {
        let mut conn = pool.get()?;

        let result = diesel::update(ref_geo_villages::table.filter(ref_geo_villages::id.eq(id.to_string())))
            .set((
                data.district_id.map(|d| ref_geo_villages::district_id.eq(d)),
                data.name.map(|n| ref_geo_villages::name.eq(n)),
                data.code.map(|c| ref_geo_villages::code.eq(c)),
                data.latitude.map(|l| ref_geo_villages::latitude.eq(l)),
                data.longitude.map(|l| ref_geo_villages::longitude.eq(l)),
                ref_geo_villages::updated_at.eq(diesel::dsl::now),
            ))
            .returning(Village::as_select())
            .get_result(&mut conn)?;

        Ok(result)
    }

    pub fn delete(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(ref_geo_villages::table.filter(ref_geo_villages::id.eq(id.to_string())))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn find_by_district_id(pool: &DbPool, district_id: String) -> Result<Vec<Village>> {
        let mut conn = pool.get()?;

        let result = ref_geo_villages::table
            .filter(ref_geo_villages::district_id.eq(district_id))
            .filter(ref_geo_villages::deleted_at.is_null())
            .order(ref_geo_villages::name.asc())
            .select(Village::as_select())
            .load(&mut conn)?;

        Ok(result)
    }

    pub fn list_all(pool: &DbPool) -> Result<Vec<Village>> {
        let mut conn = pool.get()?;

        let result = ref_geo_villages::table
            .filter(ref_geo_villages::deleted_at.is_null())
            .order(ref_geo_villages::name.asc())
            .select(Village::as_select())
            .load(&mut conn)?;

        Ok(result)
    }

    pub fn find_by_coordinates(pool: &DbPool, min_lat: f64, max_lat: f64, min_lng: f64, max_lng: f64) -> Result<Vec<Village>> {
        let mut conn = pool.get()?;

        let min_lat_decimal = Decimal::from_f64_retain(min_lat).unwrap_or_default();
        let max_lat_decimal = Decimal::from_f64_retain(max_lat).unwrap_or_default();
        let min_lng_decimal = Decimal::from_f64_retain(min_lng).unwrap_or_default();
        let max_lng_decimal = Decimal::from_f64_retain(max_lng).unwrap_or_default();

        let result = ref_geo_villages::table
            .filter(ref_geo_villages::deleted_at.is_null())
            .filter(ref_geo_villages::latitude.is_not_null())
            .filter(ref_geo_villages::longitude.is_not_null())
            .filter(ref_geo_villages::latitude.ge(min_lat_decimal))
            .filter(ref_geo_villages::latitude.le(max_lat_decimal))
            .filter(ref_geo_villages::longitude.ge(min_lng_decimal))
            .filter(ref_geo_villages::longitude.le(max_lng_decimal))
            .order(ref_geo_villages::name.asc())
            .select(Village::as_select())
            .load(&mut conn)?;

        Ok(result)
    }
}