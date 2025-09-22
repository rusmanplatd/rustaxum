use anyhow::Result;
use ulid::Ulid;
use diesel::prelude::*;
use crate::database::DbPool;
use crate::schema::organizations;
use std::collections::HashMap;

use crate::app::models::organization::{Organization, CreateOrganization, UpdateOrganization};

pub struct OrganizationService;

impl OrganizationService {
    pub fn create(pool: &DbPool, data: CreateOrganization) -> Result<Organization> {
        let mut conn = pool.get()?;

        let parent_id = if let Some(parent_id_str) = data.parent_id {
            Some(Ulid::from_string(&parent_id_str)?)
        } else {
            None
        };

        let organization = Organization::new(
            data.name,
            data.organization_type,
            parent_id,
            data.code,
            data.description,
        );

        let result = diesel::insert_into(organizations::table)
            .values((
                organizations::id.eq(organization.id.to_string()),
                organizations::name.eq(&organization.name),
                organizations::type_.eq(&organization.organization_type),
                organizations::parent_id.eq(organization.parent_id.map(|id| id.to_string())),
                organizations::code.eq(&organization.code),
                organizations::description.eq(&organization.description),
                organizations::is_active.eq(organization.is_active),
                organizations::created_at.eq(organization.created_at),
                organizations::updated_at.eq(organization.updated_at),
            ))
            .get_result::<Organization>(&mut conn)?;

        Ok(result)
    }

    pub fn find_by_id(pool: &DbPool, id: Ulid) -> Result<Option<Organization>> {
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

        let parent_id = if let Some(parent_id_str) = data.parent_id {
            Some(Ulid::from_string(&parent_id_str)?)
        } else {
            None
        };

        let result = diesel::update(organizations::table.filter(organizations::id.eq(id.to_string())))
            .set((
                data.name.map(|n| organizations::name.eq(n)),
                data.organization_type.map(|t| organizations::type_.eq(t)),
                parent_id.map(|p| organizations::parent_id.eq(p.to_string())),
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