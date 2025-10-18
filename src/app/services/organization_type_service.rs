use anyhow::Result;
use diesel::prelude::*;
use chrono::Utc;
use serde_json::json;
use crate::database::DbPool;
use crate::app::models::{
    DieselUlid,
    organization_type::{OrganizationType, CreateOrganizationType, UpdateOrganizationType}
};
use crate::schema::organization_types;
use crate::app::traits::ServiceActivityLogger;

pub struct OrganizationTypeService;

impl ServiceActivityLogger for OrganizationTypeService {}

impl OrganizationTypeService {
    /// Create a new organization type
    pub async fn create(pool: &DbPool, data: CreateOrganizationType, created_by: &str) -> Result<OrganizationType> {
        let mut conn = pool.get()?;

        let created_by_ulid = DieselUlid::from_string(created_by)?;
        let new_type = OrganizationType::new(data, created_by_ulid);

        let org_type = diesel::insert_into(organization_types::table)
            .values(&new_type)
            .returning(OrganizationType::as_returning())
            .get_result(&mut conn)?;

        // Log the type creation activity
        let service = OrganizationTypeService;
        let properties = json!({
            "type_name": org_type.name.clone(),
            "type_code": org_type.code.clone(),
            "level": org_type.level,
            "domain_id": org_type.domain_id.to_string(),
            "created_by": created_by
        });

        if let Err(e) = service.log_created(
            &org_type,
            Some(created_by),
            Some(properties)
        ).await {
            eprintln!("Failed to log organization type creation activity: {}", e);
        }

        Ok(org_type)
    }

    /// Find organization type by ID
    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<OrganizationType>> {
        let mut conn = pool.get()?;

        let org_type = organization_types::table
            .filter(organization_types::id.eq(id.to_string()))
            .filter(organization_types::deleted_at.is_null())
            .select(OrganizationType::as_select())
            .first(&mut conn)
            .optional()?;

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
    pub fn list(pool: &DbPool, _query_params: std::collections::HashMap<String, String>) -> Result<Vec<OrganizationType>> {
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
    pub async fn update(pool: &DbPool, id: String, data: UpdateOrganizationType, updated_by: &str) -> Result<OrganizationType> {
        let mut conn = pool.get()?;

        // Get the original record for logging
        let original = organization_types::table
            .filter(organization_types::id.eq(&id))
            .filter(organization_types::deleted_at.is_null())
            .select(OrganizationType::as_select())
            .first::<OrganizationType>(&mut conn)
            .optional()?;

        let updated_by_ulid = DieselUlid::from_string(updated_by)?;

        let result = diesel::update(organization_types::table.filter(organization_types::id.eq(&id)))
            .set((
                data.name.as_ref().map(|n| organization_types::name.eq(n)),
                data.domain_id.as_ref().map(|d| organization_types::domain_id.eq(d)),
                data.code.as_ref().map(|c| organization_types::code.eq(c)),
                data.description.as_ref().map(|d| organization_types::description.eq(d)),
                data.level.as_ref().map(|l| organization_types::level.eq(l)),
                organization_types::updated_at.eq(Utc::now()),
                organization_types::updated_by_id.eq(updated_by_ulid),
            ))
            .returning(OrganizationType::as_returning())
            .get_result(&mut conn)?;

        // Log the update activity
        let service = OrganizationTypeService;
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
            if let Some(ref new_domain_id) = data.domain_id {
                if &original.domain_id != new_domain_id {
                    changes["domain_id"] = json!({
                        "from": original.domain_id.to_string(),
                        "to": new_domain_id.to_string()
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
            if let Some(ref new_level) = data.level {
                if &original.level != new_level {
                    changes["level"] = json!({
                        "from": original.level,
                        "to": new_level
                    });
                }
            }
        }

        if let Err(e) = service.log_updated(
            &result,
            changes,
            Some(updated_by)
        ).await {
            eprintln!("Failed to log organization type update activity: {}", e);
        }

        Ok(result)
    }

    /// Soft delete organization type
    pub async fn delete(pool: &DbPool, id: String, deleted_by: &str) -> Result<()> {
        let mut conn = pool.get()?;

        // Get the type before deletion for logging
        let org_type = organization_types::table
            .filter(organization_types::id.eq(&id))
            .filter(organization_types::deleted_at.is_null())
            .select(OrganizationType::as_select())
            .first::<OrganizationType>(&mut conn)
            .optional()?;

        let deleted_by_ulid = DieselUlid::from_string(deleted_by)?;

        diesel::update(organization_types::table.filter(organization_types::id.eq(&id)))
            .set((
                organization_types::deleted_at.eq(Utc::now()),
                organization_types::deleted_by_id.eq(deleted_by_ulid),
            ))
            .execute(&mut conn)?;

        // Log the deletion activity
        if let Some(org_type) = org_type {
            let service = OrganizationTypeService;

            if let Err(e) = service.log_deleted(
                &org_type,
                Some(deleted_by)
            ).await {
                eprintln!("Failed to log organization type deletion activity: {}", e);
            }
        }

        Ok(())
    }

    /// Count organization types
    pub fn count(pool: &DbPool) -> Result<i64> {
        let mut conn = pool.get()?;

        let result = organization_types::table
            .filter(organization_types::deleted_at.is_null())
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(result)
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
