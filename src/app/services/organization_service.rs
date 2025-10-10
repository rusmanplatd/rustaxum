use anyhow::Result;
use diesel::prelude::*;
use serde_json::json;
use crate::database::DbPool;
use crate::schema::organizations;
use std::collections::HashMap;

use crate::app::models::organization::{Organization, CreateOrganization, UpdateOrganization};
use crate::app::traits::ServiceActivityLogger;

pub struct OrganizationService;

impl ServiceActivityLogger for OrganizationService {}

impl OrganizationService {
    pub async fn create(pool: &DbPool, data: CreateOrganization, created_by: Option<&str>) -> Result<Organization> {
        let mut conn = pool.get()?;

        // Convert created_by string to DieselUlid if provided
        let created_by_ulid = created_by.and_then(|id_str| {
            crate::app::models::DieselUlid::from_string(id_str).ok()
        });

        let new_org = Organization::new(data.clone(), created_by_ulid);

        let result = diesel::insert_into(organizations::table)
            .values(&new_org)
            .get_result::<Organization>(&mut conn)?;

        // Log the organization creation activity
        let service = OrganizationService;
        let properties = json!({
            "organization_name": result.name,
            "organization_code": result.code,
            "created_by": created_by
        });

        if let Err(e) = service.log_created(
            &result,
            created_by,
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
            .first::<Organization>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_code(pool: &DbPool, code: &str) -> Result<Option<Organization>> {
        let mut conn = pool.get()?;

        let result = organizations::table
            .filter(organizations::code.eq(code))
            .first::<Organization>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn list(pool: &DbPool, _query_params: HashMap<String, String>) -> Result<Vec<Organization>> {
        let mut conn = pool.get()?;

        let result = organizations::table
            .order(organizations::name.asc())
            .load::<Organization>(&mut conn)?;

        Ok(result)
    }

    pub fn update(pool: &DbPool, id: String, data: UpdateOrganization) -> Result<Organization> {
        let mut conn = pool.get()?;

        let parent_id = if let Some(parent_id_ulid) = data.parent_id {
            Some(parent_id_ulid)
        } else {
            None
        };

        let result = diesel::update(organizations::table.filter(organizations::id.eq(id.to_string())))
            .set((
                data.name.map(|n| organizations::name.eq(n)),
                data.domain_id.map(|d| organizations::domain_id.eq(d.to_string())),
                data.type_id.map(|t| organizations::type_id.eq(t.to_string())),
                parent_id.map(|p| organizations::parent_id.eq(p)),
                data.code.map(|c| organizations::code.eq(c)),
                data.description.map(|d| organizations::description.eq(d)),
                data.is_active.map(|a| organizations::is_active.eq(a)),
                organizations::updated_at.eq(chrono::Utc::now()),
            ))
            .get_result::<Organization>(&mut conn)?;

        Ok(result)
    }

    pub fn delete(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;

        let rows_affected = diesel::delete(organizations::table.filter(organizations::id.eq(id.to_string())))
            .execute(&mut conn)?;

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("Organization not found"));
        }

        Ok(())
    }

    pub fn find_children(pool: &DbPool, parent_id: String) -> Result<Vec<Organization>> {
        let mut conn = pool.get()?;

        let results = organizations::table
            .filter(organizations::parent_id.eq(parent_id.to_string()))
            .order(organizations::name.asc())
            .load::<Organization>(&mut conn)?;

        Ok(results)
    }

    pub fn find_root_organizations(pool: &DbPool) -> Result<Vec<Organization>> {
        let mut conn = pool.get()?;

        let results = organizations::table
            .filter(organizations::parent_id.is_null())
            .order(organizations::name.asc())
            .load::<Organization>(&mut conn)?;

        Ok(results)
    }
}