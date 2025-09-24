use anyhow::Result;
use ulid::Ulid;
use diesel::prelude::*;
use serde_json::json;
use crate::database::{DbPool};
use crate::schema::sys_roles;
use crate::app::models::role::{Role, CreateRole, UpdateRole};
use crate::app::models::HasRoles;
use crate::app::traits::ServiceActivityLogger;

pub struct RoleService;

impl ServiceActivityLogger for RoleService {}

impl RoleService {
    pub async fn create(pool: &DbPool, data: CreateRole, created_by: Option<&str>) -> Result<Role> {
        let mut conn = pool.get()?;
        let role = Role::new(data.name.clone(), data.description.clone(), data.guard_name.clone());

        diesel::insert_into(sys_roles::table)
            .values((
                sys_roles::id.eq(role.id.to_string()),
                sys_roles::name.eq(&role.name),
                sys_roles::description.eq(&role.description),
                sys_roles::guard_name.eq(&role.guard_name),
                sys_roles::created_at.eq(role.created_at),
                sys_roles::updated_at.eq(role.updated_at),
            ))
            .execute(&mut conn)?;

        // Log the role creation activity
        let service = RoleService;
        let properties = json!({
            "role_name": role.name,
            "role_description": role.description,
            "guard_name": role.guard_name,
            "created_by": created_by
        });

        if let Err(e) = service.log_created(
            &role,
            created_by,
            Some(properties)
        ).await {
            eprintln!("Failed to log role creation activity: {}", e);
        }

        Ok(role)
    }

    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<Role>> {
        let mut conn = pool.get()?;

        let role = sys_roles::table
            .filter(sys_roles::id.eq(id))
            .select(Role::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(role)
    }

    pub fn find_by_name(pool: &DbPool, name: &str, guard_name: Option<&str>) -> Result<Option<Role>> {
        let mut conn = pool.get()?;
        let guard = guard_name.unwrap_or("api");

        let role = sys_roles::table
            .filter(sys_roles::name.eq(name))
            .filter(sys_roles::guard_name.eq(guard))
            .select(Role::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(role)
    }

    pub async fn update(pool: &DbPool, id: String, data: UpdateRole, updated_by: Option<&str>) -> Result<Role> {
        let mut conn = pool.get()?;
        let original_role = Self::find_by_id(pool, id.clone())?
            .ok_or_else(|| anyhow::anyhow!("Role not found"))?;

        let mut role = original_role.clone();
        let mut changes = json!({});

        if let Some(ref name) = data.name {
            if &role.name != name {
                changes["name"] = json!({
                    "from": role.name,
                    "to": name
                });
                role.name = name.clone();
            }
        }
        if let Some(ref description) = data.description {
            if role.description.as_ref() != Some(description) {
                changes["description"] = json!({
                    "from": role.description,
                    "to": description
                });
                role.description = Some(description.clone());
            }
        }
        if let Some(ref guard_name) = data.guard_name {
            if &role.guard_name != guard_name {
                changes["guard_name"] = json!({
                    "from": role.guard_name,
                    "to": guard_name
                });
                role.guard_name = guard_name.clone();
            }
        }
        role.updated_at = chrono::Utc::now();

        diesel::update(sys_roles::table.filter(sys_roles::id.eq(id.to_string())))
            .set((
                sys_roles::name.eq(&role.name),
                sys_roles::description.eq(&role.description),
                sys_roles::guard_name.eq(&role.guard_name),
                sys_roles::updated_at.eq(role.updated_at),
            ))
            .execute(&mut conn)?;

        // Log the role update activity
        let service = RoleService;
        if let Err(e) = service.log_updated(
            &role,
            changes,
            updated_by
        ).await {
            eprintln!("Failed to log role update activity: {}", e);
        }

        Ok(role)
    }

    pub async fn delete(pool: &DbPool, id: String, deleted_by: Option<&str>) -> Result<()> {
        let mut conn = pool.get()?;

        // Get the role before deletion for logging
        let role = Self::find_by_id(pool, id.clone())?
            .ok_or_else(|| anyhow::anyhow!("Role not found"))?;

        diesel::delete(sys_roles::table.filter(sys_roles::id.eq(id.to_string())))
            .execute(&mut conn)?;

        // Log the role deletion activity
        let service = RoleService;
        if let Err(e) = service.log_deleted(
            &role,
            deleted_by
        ).await {
            eprintln!("Failed to log role deletion activity: {}", e);
        }

        Ok(())
    }

    pub fn list(pool: &DbPool, limit: i64, offset: i64) -> Result<Vec<Role>> {
        let mut conn = pool.get()?;

        let roles = sys_roles::table
            .order(sys_roles::created_at.desc())
            .limit(limit)
            .offset(offset)
            .select(Role::as_select())
            .load(&mut conn)?;

        Ok(roles)
    }

    /// Generic method to assign a role to any model that implements HasRoles
    pub async fn assign_to_model<T: HasRoles>(pool: &DbPool, model: &T, role_id: String) -> Result<()> {
        use crate::schema::sys_model_has_roles;
        let mut conn = pool.get()?;
        let model_role_id = Ulid::new();
        let now = chrono::Utc::now();

        diesel::insert_into(sys_model_has_roles::table)
            .values((
                sys_model_has_roles::id.eq(model_role_id.to_string()),
                sys_model_has_roles::model_type.eq(T::model_type()),
                sys_model_has_roles::model_id.eq(model.model_id()),
                sys_model_has_roles::role_id.eq(role_id.clone()),
                sys_model_has_roles::scope_type.eq::<Option<String>>(None),
                sys_model_has_roles::scope_id.eq::<Option<String>>(None),
                sys_model_has_roles::created_at.eq(now),
                sys_model_has_roles::updated_at.eq(now),
            ))
            .on_conflict((sys_model_has_roles::model_type, sys_model_has_roles::model_id, sys_model_has_roles::role_id))
            .do_nothing()
            .execute(&mut conn)?;

        // Log the role assignment activity
        let service = RoleService;
        let properties = json!({
            "model_type": T::model_type(),
            "model_id": model.model_id(),
            "role_id": role_id,
            "action": "role_assigned"
        });

        if let Err(e) = service.log_system_event(
            "role_assignment",
            &format!("Assigned role {} to {} {}", role_id, T::model_type(), model.model_id()),
            Some(properties)
        ).await {
            eprintln!("Failed to log role assignment activity: {}", e);
        }

        Ok(())
    }

    /// Generic method to remove a role from any model that implements HasRoles
    pub async fn remove_from_model<T: HasRoles>(pool: &DbPool, model: &T, role_id: String) -> Result<()> {
        use crate::schema::sys_model_has_roles;
        let mut conn = pool.get()?;

        diesel::delete(
            sys_model_has_roles::table
                .filter(sys_model_has_roles::model_type.eq(T::model_type()))
                .filter(sys_model_has_roles::model_id.eq(model.model_id()))
                .filter(sys_model_has_roles::role_id.eq(role_id.clone()))
        )
        .execute(&mut conn)?;

        // Log the role removal activity
        let service = RoleService;
        let properties = json!({
            "model_type": T::model_type(),
            "model_id": model.model_id(),
            "role_id": role_id,
            "action": "role_removed",
        });

        if let Err(e) = service.log_system_event(
            "role_removal",
            &format!("Removed role {} from {} {}", role_id, T::model_type(), model.model_id()),
            Some(properties)
        ).await {
            eprintln!("Failed to log role removal activity: {}", e);
        }

        Ok(())
    }

    /// Generic method to check if a model has a specific role
    pub fn model_has_role<T: HasRoles>(pool: &DbPool, model: &T, role_name: &str, guard_name: Option<&str>) -> Result<bool> {
        use crate::schema::{sys_model_has_roles, sys_roles};
        let mut conn = pool.get()?;
        let guard = guard_name.unwrap_or("api");

        let count = sys_model_has_roles::table
            .inner_join(sys_roles::table.on(sys_model_has_roles::role_id.eq(sys_roles::id)))
            .filter(sys_model_has_roles::model_type.eq(T::model_type()))
            .filter(sys_model_has_roles::model_id.eq(model.model_id()))
            .filter(sys_roles::name.eq(role_name))
            .filter(sys_roles::guard_name.eq(guard))
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(count > 0)
    }

    /// Generic method to get roles for any model that implements HasRoles
    pub fn get_model_roles<T: HasRoles>(pool: &DbPool, model: &T, guard_name: Option<&str>) -> Result<Vec<Role>> {
        use crate::schema::{sys_model_has_roles, sys_roles};
        let mut conn = pool.get()?;

        let mut query = sys_roles::table
            .inner_join(sys_model_has_roles::table.on(sys_roles::id.eq(sys_model_has_roles::role_id)))
            .filter(sys_model_has_roles::model_type.eq(T::model_type()))
            .filter(sys_model_has_roles::model_id.eq(model.model_id()))
            .select(sys_roles::all_columns)
            .order(sys_roles::name)
            .into_boxed();

        if let Some(guard) = guard_name {
            query = query.filter(sys_roles::guard_name.eq(guard));
        }

        let roles = query.select(Role::as_select()).load(&mut conn)?;
        Ok(roles)
    }

    pub fn count(pool: &DbPool) -> Result<i64> {
        let mut conn = pool.get()?;

        let count = sys_roles::table.count().get_result::<i64>(&mut conn)?;
        Ok(count)
    }
}