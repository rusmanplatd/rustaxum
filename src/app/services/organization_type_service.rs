use anyhow::Result;
use diesel::prelude::*;
use crate::database::DbPool;
use crate::app::models::{
    DieselUlid,
    organization_type::{OrganizationType, CreateOrganizationType, UpdateOrganizationType}
};
use crate::schema::organization_types;

pub struct OrganizationTypeService;

impl OrganizationTypeService {
    /// Create a new organization type
    pub fn create(pool: &DbPool, data: CreateOrganizationType, created_by: Option<DieselUlid>) -> Result<OrganizationType> {
        let mut conn = pool.get()?;

        let new_type = OrganizationType::new(data, created_by);

        let org_type = diesel::insert_into(organization_types::table)
            .values(&new_type)
            .returning(OrganizationType::as_returning())
            .get_result(&mut conn)?;

        Ok(org_type)
    }

    /// Find organization type by ID
    pub fn find_by_id(pool: &DbPool, id: &str) -> Result<OrganizationType> {
        let mut conn = pool.get()?;

        let org_type = organization_types::table
            .filter(organization_types::id.eq(id))
            .filter(organization_types::deleted_at.is_null())
            .select(OrganizationType::as_select())
            .first(&mut conn)?;

        Ok(org_type)
    }

    /// Find organization type by code within a domain
    pub fn find_by_code(pool: &DbPool, domain_id: &str, code: &str) -> Result<Option<OrganizationType>> {
        let mut conn = pool.get()?;

        let org_type = organization_types::table
            .filter(organization_types::domain_id.eq(domain_id))
            .filter(organization_types::code.eq(code))
            .filter(organization_types::deleted_at.is_null())
            .select(OrganizationType::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(org_type)
    }

    /// List all organization types
    pub fn list(pool: &DbPool) -> Result<Vec<OrganizationType>> {
        let mut conn = pool.get()?;

        let types = organization_types::table
            .filter(organization_types::deleted_at.is_null())
            .order((organization_types::domain_id.asc(), organization_types::level.asc()))
            .select(OrganizationType::as_select())
            .load(&mut conn)?;

        Ok(types)
    }

    /// List organization types by domain
    pub fn list_by_domain(pool: &DbPool, domain_id: &str) -> Result<Vec<OrganizationType>> {
        let mut conn = pool.get()?;

        let types = organization_types::table
            .filter(organization_types::domain_id.eq(domain_id))
            .filter(organization_types::deleted_at.is_null())
            .order(organization_types::level.asc())
            .select(OrganizationType::as_select())
            .load(&mut conn)?;

        Ok(types)
    }

    /// List organization types by level
    pub fn list_by_level(pool: &DbPool, level: i32) -> Result<Vec<OrganizationType>> {
        let mut conn = pool.get()?;

        let types = organization_types::table
            .filter(organization_types::level.eq(level))
            .filter(organization_types::deleted_at.is_null())
            .order(organization_types::name.asc())
            .select(OrganizationType::as_select())
            .load(&mut conn)?;

        Ok(types)
    }

    /// Update organization type
    pub fn update(pool: &DbPool, id: &str, data: UpdateOrganizationType, updated_by: Option<DieselUlid>) -> Result<OrganizationType> {
        let mut conn = pool.get()?;

        let result = diesel::update(organization_types::table.filter(organization_types::id.eq(id)))
            .set((
                data.name.map(|n| organization_types::name.eq(n)),
                data.domain_id.map(|d| organization_types::domain_id.eq(d.to_string())),
                data.code.map(|c| organization_types::code.eq(c)),
                data.description.map(|d| organization_types::description.eq(d)),
                data.level.map(|l| organization_types::level.eq(l)),
                organization_types::updated_at.eq(chrono::Utc::now()),
                updated_by.map(|u| organization_types::updated_by_id.eq(u.to_string())),
            ))
            .returning(OrganizationType::as_returning())
            .get_result(&mut conn)?;

        Ok(result)
    }

    /// Soft delete organization type
    pub fn delete(pool: &DbPool, id: &str, deleted_by: Option<DieselUlid>) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(organization_types::table.filter(organization_types::id.eq(id)))
            .set((
                organization_types::deleted_at.eq(chrono::Utc::now()),
                deleted_by.map(|u| organization_types::deleted_by_id.eq(u.to_string())),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    /// Check if type code is unique within a domain
    pub fn is_code_unique(pool: &DbPool, domain_id: &str, code: &str, exclude_id: Option<&str>) -> Result<bool> {
        let mut conn = pool.get()?;

        let mut query = organization_types::table
            .filter(organization_types::domain_id.eq(domain_id))
            .filter(organization_types::code.eq(code))
            .filter(organization_types::deleted_at.is_null())
            .into_boxed();

        if let Some(id) = exclude_id {
            query = query.filter(organization_types::id.ne(id));
        }

        let count: i64 = query.count().get_result(&mut conn)?;

        Ok(count == 0)
    }
}
