use anyhow::Result;
use diesel::prelude::*;
use crate::database::DbPool;
use crate::app::models::{
    DieselUlid,
    organization_domain::{OrganizationDomain, CreateOrganizationDomain, UpdateOrganizationDomain, NewOrganizationDomain}
};
use crate::schema::organization_domains;

pub struct OrganizationDomainService;

impl OrganizationDomainService {
    /// Create a new organization domain
    pub fn create(pool: &DbPool, data: CreateOrganizationDomain, created_by: Option<DieselUlid>) -> Result<OrganizationDomain> {
        let mut conn = pool.get()?;

        let new_domain = NewOrganizationDomain::new(data, created_by);

        let domain = diesel::insert_into(organization_domains::table)
            .values(&new_domain)
            .returning(OrganizationDomain::as_returning())
            .get_result(&mut conn)?;

        Ok(domain)
    }

    /// Find organization domain by ID
    pub fn find_by_id(pool: &DbPool, id: &str) -> Result<OrganizationDomain> {
        let mut conn = pool.get()?;

        let domain = organization_domains::table
            .filter(organization_domains::id.eq(id))
            .filter(organization_domains::deleted_at.is_null())
            .select(OrganizationDomain::as_select())
            .first(&mut conn)?;

        Ok(domain)
    }

    /// Find organization domain by code
    pub fn find_by_code(pool: &DbPool, code: &str) -> Result<Option<OrganizationDomain>> {
        let mut conn = pool.get()?;

        let domain = organization_domains::table
            .filter(organization_domains::code.eq(code))
            .filter(organization_domains::deleted_at.is_null())
            .select(OrganizationDomain::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(domain)
    }

    /// List all organization domains
    pub fn list(pool: &DbPool) -> Result<Vec<OrganizationDomain>> {
        let mut conn = pool.get()?;

        let domains = organization_domains::table
            .filter(organization_domains::deleted_at.is_null())
            .order(organization_domains::name.asc())
            .select(OrganizationDomain::as_select())
            .load(&mut conn)?;

        Ok(domains)
    }

    /// Update organization domain
    pub fn update(pool: &DbPool, id: &str, data: UpdateOrganizationDomain, updated_by: Option<DieselUlid>) -> Result<OrganizationDomain> {
        let mut conn = pool.get()?;

        let result = diesel::update(organization_domains::table.filter(organization_domains::id.eq(id)))
            .set((
                data.name.map(|n| organization_domains::name.eq(n)),
                data.code.map(|c| organization_domains::code.eq(c)),
                data.description.map(|d| organization_domains::description.eq(d)),
                organization_domains::updated_at.eq(chrono::Utc::now()),
                updated_by.map(|u| organization_domains::updated_by_id.eq(u.to_string())),
            ))
            .returning(OrganizationDomain::as_returning())
            .get_result(&mut conn)?;

        Ok(result)
    }

    /// Soft delete organization domain
    pub fn delete(pool: &DbPool, id: &str, deleted_by: Option<DieselUlid>) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(organization_domains::table.filter(organization_domains::id.eq(id)))
            .set((
                organization_domains::deleted_at.eq(chrono::Utc::now()),
                deleted_by.map(|u| organization_domains::deleted_by_id.eq(u.to_string())),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    /// Check if domain code is unique
    pub fn is_code_unique(pool: &DbPool, code: &str, exclude_id: Option<&str>) -> Result<bool> {
        let mut conn = pool.get()?;

        let mut query = organization_domains::table
            .filter(organization_domains::code.eq(code))
            .filter(organization_domains::deleted_at.is_null())
            .into_boxed();

        if let Some(id) = exclude_id {
            query = query.filter(organization_domains::id.ne(id));
        }

        let count: i64 = query.count().get_result(&mut conn)?;

        Ok(count == 0)
    }
}
