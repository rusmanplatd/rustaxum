use anyhow::Result;
use diesel::prelude::*;
use chrono::Utc;
use serde_json::json;
use crate::database::DbPool;
use crate::app::models::{
    DieselUlid,
    organization_domain::{OrganizationDomain, CreateOrganizationDomain, UpdateOrganizationDomain}
};
use crate::schema::organization_domains;
use crate::app::traits::ServiceActivityLogger;

pub struct OrganizationDomainService;

impl ServiceActivityLogger for OrganizationDomainService {}

impl OrganizationDomainService {
    /// Create a new organization domain
    pub async fn create(pool: &DbPool, data: CreateOrganizationDomain, created_by: &str) -> Result<OrganizationDomain> {
        let mut conn = pool.get()?;

        let created_by_ulid = DieselUlid::from_string(created_by)?;
        let new_domain = OrganizationDomain::new(data, created_by_ulid);

        let domain = diesel::insert_into(organization_domains::table)
            .values(&new_domain)
            .returning(OrganizationDomain::as_returning())
            .get_result(&mut conn)?;

        // Log the domain creation activity
        let service = OrganizationDomainService;
        let properties = json!({
            "domain_name": domain.name.clone(),
            "domain_code": domain.code.clone(),
            "created_by": created_by
        });

        if let Err(e) = service.log_created(
            &domain,
            Some(created_by),
            Some(properties)
        ).await {
            eprintln!("Failed to log organization domain creation activity: {}", e);
        }

        Ok(domain)
    }

    /// Find organization domain by ID
    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<OrganizationDomain>> {
        let mut conn = pool.get()?;

        let domain = organization_domains::table
            .filter(organization_domains::id.eq(id.to_string()))
            .filter(organization_domains::deleted_at.is_null())
            .select(OrganizationDomain::as_select())
            .first(&mut conn)
            .optional()?;

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
    pub fn list(pool: &DbPool, _query_params: std::collections::HashMap<String, String>) -> Result<Vec<OrganizationDomain>> {
        let mut conn = pool.get()?;

        let domains = organization_domains::table
            .filter(organization_domains::deleted_at.is_null())
            .order(organization_domains::name.asc())
            .select(OrganizationDomain::as_select())
            .load(&mut conn)?;

        Ok(domains)
    }

    /// Update organization domain
    pub async fn update(pool: &DbPool, id: String, data: UpdateOrganizationDomain, updated_by: &str) -> Result<OrganizationDomain> {
        let mut conn = pool.get()?;

        // Get the original record for logging
        let original = organization_domains::table
            .filter(organization_domains::id.eq(&id))
            .filter(organization_domains::deleted_at.is_null())
            .select(OrganizationDomain::as_select())
            .first::<OrganizationDomain>(&mut conn)
            .optional()?;

        let updated_by_ulid = DieselUlid::from_string(updated_by)?;

        let result = diesel::update(organization_domains::table.filter(organization_domains::id.eq(&id)))
            .set((
                data.name.as_ref().map(|n| organization_domains::name.eq(n)),
                data.code.as_ref().map(|c| organization_domains::code.eq(c)),
                data.description.as_ref().map(|d| organization_domains::description.eq(d)),
                organization_domains::updated_at.eq(Utc::now()),
                organization_domains::updated_by_id.eq(updated_by_ulid),
            ))
            .returning(OrganizationDomain::as_returning())
            .get_result(&mut conn)?;

        // Log the update activity
        let service = OrganizationDomainService;
        let mut changes = json!({});

        if let Some(original) = original {
            if let Some(ref new_name) = data.name {
                if &original.name != new_name {
                    changes["name"] = json!({
                        "from": original.name,
                        "to": new_name
                    });
                }
            }
            if let Some(ref new_code) = data.code {
                if &original.code != new_code {
                    changes["code"] = json!({
                        "from": original.code,
                        "to": new_code
                    });
                }
            }
            if let Some(ref new_description) = data.description {
                if &original.description != new_description {
                    changes["description"] = json!({
                        "from": original.description,
                        "to": new_description
                    });
                }
            }
        }

        if let Err(e) = service.log_updated(
            &result,
            changes,
            Some(updated_by)
        ).await {
            eprintln!("Failed to log organization domain update activity: {}", e);
        }

        Ok(result)
    }

    /// Soft delete organization domain
    pub async fn delete(pool: &DbPool, id: String, deleted_by: &str) -> Result<()> {
        let mut conn = pool.get()?;

        // Get the domain before deletion for logging
        let domain = organization_domains::table
            .filter(organization_domains::id.eq(&id))
            .filter(organization_domains::deleted_at.is_null())
            .select(OrganizationDomain::as_select())
            .first::<OrganizationDomain>(&mut conn)
            .optional()?;

        let deleted_by_ulid = DieselUlid::from_string(deleted_by)?;

        diesel::update(organization_domains::table.filter(organization_domains::id.eq(&id)))
            .set((
                organization_domains::deleted_at.eq(Utc::now()),
                organization_domains::deleted_by_id.eq(deleted_by_ulid),
            ))
            .execute(&mut conn)?;

        // Log the deletion activity
        if let Some(domain) = domain {
            let service = OrganizationDomainService;

            if let Err(e) = service.log_deleted(
                &domain,
                Some(deleted_by)
            ).await {
                eprintln!("Failed to log organization domain deletion activity: {}", e);
            }
        }

        Ok(())
    }

    /// Count organization domains
    pub fn count(pool: &DbPool) -> Result<i64> {
        let mut conn = pool.get()?;

        let result = organization_domains::table
            .filter(organization_domains::deleted_at.is_null())
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(result)
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
