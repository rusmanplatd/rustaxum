use anyhow::Result;
use diesel::prelude::*;
use crate::database::DbPool;
use crate::schema::countries;
use crate::app::models::country::{Country, CreateCountry, UpdateCountry};

pub struct CountryService;

impl CountryService {
    pub fn create(pool: &DbPool, data: CreateCountry) -> Result<Country> {
        let mut conn = pool.get()?;
        let country = Country::new(data.name, data.iso_code, data.phone_code);

        let result = diesel::insert_into(countries::table)
            .values((
                countries::id.eq(country.id.to_string()),
                countries::name.eq(&country.name),
                countries::iso_code.eq(&country.iso_code),
                countries::phone_code.eq(&country.phone_code),
                countries::created_at.eq(country.created_at),
                countries::updated_at.eq(country.updated_at),
            ))
            .get_result::<Country>(&mut conn)?;

        Ok(result)
    }

    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<Country>> {
        let mut conn = pool.get()?;

        let result = countries::table
            .filter(countries::id.eq(id.to_string()))
            .first::<Country>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_iso_code(pool: &DbPool, iso_code: &str) -> Result<Option<Country>> {
        let mut conn = pool.get()?;

        let result = countries::table
            .filter(countries::iso_code.eq(iso_code))
            .first::<Country>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn list(pool: &DbPool, _query_params: std::collections::HashMap<String, String>) -> Result<Vec<Country>> {
        let mut conn = pool.get()?;

        let result = countries::table
            .order(countries::name.asc())
            .load::<Country>(&mut conn)?;
        Ok(result)
    }

    pub fn update(pool: &DbPool, id: String, data: UpdateCountry) -> Result<Country> {
        let mut conn = pool.get()?;

        let result = diesel::update(countries::table.filter(countries::id.eq(id.to_string())))
            .set((
                data.name.map(|n| countries::name.eq(n)),
                data.iso_code.map(|c| countries::iso_code.eq(c)),
                data.phone_code.map(|p| countries::phone_code.eq(p)),
                countries::updated_at.eq(chrono::Utc::now()),
            ))
            .get_result::<Country>(&mut conn)?;

        Ok(result)
    }

    pub fn delete(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(countries::table.filter(countries::id.eq(id.to_string())))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn count(pool: &DbPool) -> Result<i64> {
        let mut conn = pool.get()?;

        let result = countries::table.count().get_result::<i64>(&mut conn)?;

        Ok(result)
    }
}