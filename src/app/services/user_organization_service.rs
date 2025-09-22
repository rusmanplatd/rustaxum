use anyhow::Result;
use crate::database::DbPool;
use ulid::Ulid;
use diesel::prelude::*;
use crate::schema::user_organizations;

use crate::app::models::user_organization::{UserOrganization, CreateUserOrganization, UpdateUserOrganization};

pub struct UserOrganizationService;

impl UserOrganizationService {
    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<UserOrganization>> {
        let mut conn = pool.get()?;

        let user_org = user_organizations::table
            .filter(user_organizations::id.eq(id.to_string()))
            .first::<UserOrganization>(&mut conn)
            .optional()?;

        Ok(user_org)
    }

    pub fn create(pool: &DbPool, data: CreateUserOrganization) -> Result<UserOrganization> {
        let mut conn = pool.get()?;

        // Validate ULID format
        Ulid::from_string(&data.user_id)
            .map_err(|_| anyhow::anyhow!("Invalid user ID format"))?;
        Ulid::from_string(&data.organization_id)
            .map_err(|_| anyhow::anyhow!("Invalid organization ID format"))?;
        Ulid::from_string(&data.organization_position_id)
            .map_err(|_| anyhow::anyhow!("Invalid organization position ID format"))?;

        let user_org = UserOrganization::new(
            data.user_id,
            data.organization_id,
            data.organization_position_id,
            data.started_at,
        );

        diesel::insert_into(user_organizations::table)
            .values((
                user_organizations::id.eq(user_org.id.to_string()),
                user_organizations::user_id.eq(user_org.user_id.to_string()),
                user_organizations::organization_id.eq(user_org.organization_id.to_string()),
                user_organizations::organization_position_id.eq(user_org.organization_position_id.to_string()),
                user_organizations::is_active.eq(user_org.is_active),
                user_organizations::started_at.eq(user_org.started_at),
                user_organizations::ended_at.eq(user_org.ended_at),
                user_organizations::created_at.eq(user_org.created_at),
                user_organizations::updated_at.eq(user_org.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(user_org)
    }

    pub fn update(pool: &DbPool, id: String, data: UpdateUserOrganization) -> Result<UserOrganization> {
        let mut conn = pool.get()?;

        // First get the existing record
        let mut user_org = Self::find_by_id(pool, id)?
            .ok_or_else(|| anyhow::anyhow!("User organization not found"))?;

        // Update fields if provided
        if let Some(organization_id_str) = data.organization_id {
            let organization_id = Ulid::from_string(&organization_id_str)
                .map_err(|_| anyhow::anyhow!("Invalid organization ID format"))?;
            user_org.organization_id = organization_id.to_string();
        }
        if let Some(organization_position_id_str) = data.organization_position_id {
            let organization_position_id = Ulid::from_string(&organization_position_id_str)
                .map_err(|_| anyhow::anyhow!("Invalid organization position ID format"))?;
            user_org.organization_position_id = organization_position_id.to_string();
        }
        if let Some(is_active) = data.is_active {
            user_org.is_active = is_active;
        }
        if let Some(started_at) = data.started_at {
            user_org.started_at = started_at;
        }
        if let Some(ended_at) = data.ended_at {
            user_org.ended_at = Some(ended_at);
        }

        user_org.updated_at = chrono::Utc::now();

        diesel::update(user_organizations::table.filter(user_organizations::id.eq(id.to_string())))
            .set((
                user_organizations::organization_id.eq(user_org.organization_id.to_string()),
                user_organizations::organization_position_id.eq(user_org.organization_position_id.to_string()),
                user_organizations::is_active.eq(user_org.is_active),
                user_organizations::started_at.eq(user_org.started_at),
                user_organizations::ended_at.eq(user_org.ended_at),
                user_organizations::updated_at.eq(user_org.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(user_org)
    }

    pub fn delete(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(user_organizations::table.filter(user_organizations::id.eq(id.to_string())))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn count_by_organization(pool: &DbPool, organization_id: String) -> Result<i64> {
        let mut conn = pool.get()?;

        let count = user_organizations::table
            .filter(user_organizations::organization_id.eq(organization_id.to_string()))
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(count)
    }

    pub fn list_by_user(pool: &DbPool, user_id: String) -> Result<Vec<UserOrganization>> {
        let mut conn = pool.get()?;

        let user_orgs = user_organizations::table
            .filter(user_organizations::user_id.eq(user_id.to_string()))
            .load::<UserOrganization>(&mut conn)?;

        Ok(user_orgs)
    }

    pub fn find_by_user_and_organization(pool: &DbPool, user_id: String, organization_id: String) -> Result<Option<UserOrganization>> {
        let mut conn = pool.get()?;

        let user_org = user_organizations::table
            .filter(user_organizations::user_id.eq(user_id.to_string()))
            .filter(user_organizations::organization_id.eq(organization_id.to_string()))
            .first::<UserOrganization>(&mut conn)
            .optional()?;

        Ok(user_org)
    }
}