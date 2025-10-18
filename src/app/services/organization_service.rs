use anyhow::Result;
use diesel::prelude::*;
use chrono::Utc;
use serde_json::json;
use crate::database::DbPool;
use crate::schema::organizations;
use std::collections::HashMap;

use crate::app::models::organization::{Organization, CreateOrganization, UpdateOrganization};
use crate::app::models::DieselUlid;
use crate::app::traits::ServiceActivityLogger;

pub struct OrganizationService;

impl ServiceActivityLogger for OrganizationService {}

impl OrganizationService {
    pub async fn create(pool: &DbPool, data: CreateOrganization, created_by: &str) -> Result<Organization> {
        let mut conn = pool.get()?;

        let created_by_ulid = DieselUlid::from_string(created_by)?;
        let new_org = Organization::new(data.clone(), created_by_ulid);

        let result = diesel::insert_into(organizations::table)
            .values(&new_org)
            .get_result::<Organization>(&mut conn)?;

        // Log the organization creation activity
        let service = OrganizationService;
        let properties = json!({
            "organization_name": result.name.clone(),
            "organization_code": result.code.clone(),
            "type_id": result.type_id.to_string(),
            "domain_id": result.domain_id.to_string(),
            "created_by": created_by
        });

        if let Err(e) = service.log_created(
            &result,
            Some(created_by),
            Some(properties)
        ).await {
            eprintln!("Failed to log organization creation activity: {}", e);
        }

        Ok(result)
    }

    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<Organization>> {
        let mut conn = pool.get()?;

        let result = organizations::table
            .filter(organizations::id.eq(id.to_string()))
            .filter(organizations::deleted_at.is_null())
            .first::<Organization>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_code(pool: &DbPool, code: &str) -> Result<Option<Organization>> {
        let mut conn = pool.get()?;

        let result = organizations::table
            .filter(organizations::code.eq(code))
            .filter(organizations::deleted_at.is_null())
            .first::<Organization>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn list(pool: &DbPool, _query_params: HashMap<String, String>) -> Result<Vec<Organization>> {
        let mut conn = pool.get()?;

        let result = organizations::table
            .filter(organizations::deleted_at.is_null())
            .order(organizations::name.asc())
            .load::<Organization>(&mut conn)?;

        Ok(result)
    }

    pub async fn update(pool: &DbPool, id: String, data: UpdateOrganization, updated_by: &str) -> Result<Organization> {
        let mut conn = pool.get()?;

        // Get the original record for logging
        let original = organizations::table
            .filter(organizations::id.eq(&id))
            .filter(organizations::deleted_at.is_null())
            .first::<Organization>(&mut conn)
            .optional()?;

        let updated_by_ulid = DieselUlid::from_string(updated_by)?;

        let result = diesel::update(organizations::table.filter(organizations::id.eq(&id)))
            .set((
                data.name.as_ref().map(|n| organizations::name.eq(n)),
                data.domain_id.as_ref().map(|d| organizations::domain_id.eq(d)),
                data.type_id.as_ref().map(|t| organizations::type_id.eq(t)),
                data.parent_id.as_ref().map(|p| organizations::parent_id.eq(p)),
                data.code.as_ref().map(|c| organizations::code.eq(c)),
                data.description.as_ref().map(|d| organizations::description.eq(d)),
                data.is_active.as_ref().map(|a| organizations::is_active.eq(a)),
                organizations::updated_at.eq(Utc::now()),
                organizations::updated_by_id.eq(updated_by_ulid),
            ))
            .get_result::<Organization>(&mut conn)?;

        // Log the update activity
        let service = OrganizationService;
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
            if let Some(ref new_is_active) = data.is_active {
                if &original.is_active != new_is_active {
                    changes["is_active"] = json!({
                        "from": original.is_active,
                        "to": new_is_active
                    });
                }
            }
        }

        if let Err(e) = service.log_updated(
            &result,
            changes,
            Some(updated_by)
        ).await {
            eprintln!("Failed to log organization update activity: {}", e);
        }

        Ok(result)
    }

    pub async fn delete(pool: &DbPool, id: String, deleted_by: &str) -> Result<()> {
        let mut conn = pool.get()?;

        // Get the organization before deletion for logging
        let organization = organizations::table
            .filter(organizations::id.eq(&id))
            .filter(organizations::deleted_at.is_null())
            .first::<Organization>(&mut conn)
            .optional()?;

        let deleted_by_ulid = DieselUlid::from_string(deleted_by)?;

        diesel::update(organizations::table.filter(organizations::id.eq(&id)))
            .set((
                organizations::deleted_at.eq(Utc::now()),
                organizations::deleted_by_id.eq(deleted_by_ulid),
            ))
            .execute(&mut conn)?;

        // Log the deletion activity
        if let Some(organization) = organization {
            let service = OrganizationService;

            if let Err(e) = service.log_deleted(
                &organization,
                Some(deleted_by)
            ).await {
                eprintln!("Failed to log organization deletion activity: {}", e);
            }
        }

        Ok(())
    }

    pub fn find_children(pool: &DbPool, parent_id: String) -> Result<Vec<Organization>> {
        let mut conn = pool.get()?;

        let results = organizations::table
            .filter(organizations::parent_id.eq(parent_id.to_string()))
            .filter(organizations::deleted_at.is_null())
            .order(organizations::name.asc())
            .load::<Organization>(&mut conn)?;

        Ok(results)
    }

    pub fn find_root_organizations(pool: &DbPool) -> Result<Vec<Organization>> {
        let mut conn = pool.get()?;

        let results = organizations::table
            .filter(organizations::parent_id.is_null())
            .filter(organizations::deleted_at.is_null())
            .order(organizations::name.asc())
            .load::<Organization>(&mut conn)?;

        Ok(results)
    }

    pub fn count(pool: &DbPool) -> Result<i64> {
        let mut conn = pool.get()?;

        let result = organizations::table
            .filter(organizations::deleted_at.is_null())
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(result)
    }
}