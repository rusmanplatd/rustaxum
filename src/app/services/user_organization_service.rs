use anyhow::Result;
use crate::database::DbPool;
use diesel::prelude::*;
use serde_json::json;
use crate::schema::user_organizations;
use crate::app::traits::ServiceActivityLogger;

use crate::app::models::user_organization::{UserOrganization, CreateUserOrganization, UpdateUserOrganization, NewUserOrganization};

pub struct UserOrganizationService;

impl ServiceActivityLogger for UserOrganizationService {}

impl UserOrganizationService {
    pub fn find_by_id(pool: &DbPool, id: &str) -> Result<Option<UserOrganization>> {
        let mut conn = pool.get()?;

        let user_org = user_organizations::table
            .filter(user_organizations::id.eq(id))
            .first::<UserOrganization>(&mut conn)
            .optional()?;

        Ok(user_org)
    }

    pub async fn create(pool: &DbPool, data: CreateUserOrganization, created_by: Option<&str>) -> Result<UserOrganization> {
        let mut conn = pool.get()?;

        let new_user_org = NewUserOrganization {
            id: crate::app::models::DieselUlid::new(),
            user_id: data.user_id,
            organization_id: data.organization_id,
            organization_position_id: data.organization_position_id,
            is_active: true,
            started_at: data.started_at.unwrap_or_else(|| chrono::Utc::now()),
            ended_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
            created_by: created_by.map(|s| s.to_string()),
            updated_by: None,
            deleted_by: None,
        };

        let result = diesel::insert_into(user_organizations::table)
            .values(&new_user_org)
            .get_result::<UserOrganization>(&mut conn)?;

        // Log user-organization assignment activity
        let service = Self;
        let properties = json!({
            "user_id": result.user_id.to_string(),
            "organization_id": result.organization_id.to_string(),
            "organization_position_id": result.organization_position_id.map(|id| id.to_string()),
            "started_at": result.started_at,
            "action": "user_organization_created"
        });

        if let Err(e) = service.log_created(
            &result,
            created_by,
            Some(properties)
        ).await {
            eprintln!("Failed to log user organization creation activity: {}", e);
        }

        Ok(result)
    }

    pub fn update(pool: &DbPool, id: String, data: UpdateUserOrganization) -> Result<UserOrganization> {
        let mut conn = pool.get()?;

        // First get the existing record
        let mut user_org = Self::find_by_id(pool, &id)?
            .ok_or_else(|| anyhow::anyhow!("User organization not found"))?;

        // Update fields if provided
        if let Some(organization_id) = data.organization_id {
            user_org.organization_id = organization_id;
        }
        if let Some(organization_position_id) = data.organization_position_id {
            user_org.organization_position_id = organization_position_id;
        }
        if let Some(is_active) = data.is_active {
            user_org.is_active = is_active;
        }
        if let Some(started_at) = data.started_at {
            user_org.started_at = started_at;
        }
        if let Some(ended_at) = data.ended_at {
            user_org.ended_at = ended_at;
        }

        user_org.updated_at = chrono::Utc::now();

        diesel::update(user_organizations::table.filter(user_organizations::id.eq(&id)))
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