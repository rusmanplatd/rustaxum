use anyhow::Result;
use ulid::Ulid;
use diesel::prelude::*;
use serde_json::json;
use crate::database::DbPool;
use crate::schema::{sys_permissions, sys_model_has_permissions, sys_model_has_roles};
use crate::app::models::permission::{Permission, CreatePermission, UpdatePermission};
use crate::app::models::HasRoles;
use crate::app::traits::ServiceActivityLogger;

pub struct PermissionService;

impl ServiceActivityLogger for PermissionService {}

impl PermissionService {
    pub async fn create(pool: &DbPool, data: CreatePermission, created_by: Option<&str>) -> Result<Permission> {
        let permission = Permission::new(
            data.name.clone(),
            data.guard_name.clone(),
            data.resource.clone(),
            data.action.clone(),
        );

        let mut conn = pool.get()?;

        diesel::insert_into(sys_permissions::table)
            .values((
                sys_permissions::id.eq(permission.id.to_string()),
                sys_permissions::name.eq(&permission.name),
                sys_permissions::guard_name.eq(&permission.guard_name),
                sys_permissions::resource.eq(&permission.resource),
                sys_permissions::action.eq(&permission.action),
                sys_permissions::created_at.eq(permission.created_at),
                sys_permissions::updated_at.eq(permission.updated_at),
            ))
            .execute(&mut conn)?;

        // Log the permission creation activity
        let service = PermissionService;
        let properties = json!({
            "permission_name": permission.name,
            "resource": permission.resource,
            "action": permission.action,
            "guard_name": permission.guard_name,
            "created_by": created_by
        });

        if let Err(e) = service.log_created(
            &permission,
            created_by,
            Some(properties)
        ).await {
            eprintln!("Failed to log permission creation activity: {}", e);
        }

        Ok(permission)
    }

    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<Permission>> {
        let mut conn = pool.get()?;

        let permission = sys_permissions::table
            .filter(sys_permissions::id.eq(id))
            .select(Permission::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(permission)
    }

    pub fn find_by_name(pool: &DbPool, name: &str, guard_name: Option<&str>) -> Result<Option<Permission>> {
        let guard = guard_name.unwrap_or("api");

        let mut conn = pool.get()?;

        let permission = sys_permissions::table
            .filter(sys_permissions::name.eq(name))
            .filter(sys_permissions::guard_name.eq(guard))
            .select(Permission::as_select())
            .first(&mut conn)
            .optional()?;

        Ok(permission)
    }

    pub fn update(pool: &DbPool, id: String, data: UpdatePermission) -> Result<Permission> {
        let mut permission = Self::find_by_id(pool, id.clone())?
            .ok_or_else(|| anyhow::anyhow!("Permission not found"))?;

        if let Some(name) = data.name {
            permission.name = name;
        }
        if let Some(guard_name) = data.guard_name {
            permission.guard_name = guard_name;
        }
        if data.resource.is_some() {
            permission.resource = data.resource;
        }
        if let Some(action) = data.action {
            permission.action = action;
        }
        permission.updated_at = chrono::Utc::now();

        let mut conn = pool.get()?;

        diesel::update(sys_permissions::table.filter(sys_permissions::id.eq(id.to_string())))
            .set((
                sys_permissions::name.eq(&permission.name),
                sys_permissions::guard_name.eq(&permission.guard_name),
                sys_permissions::resource.eq(&permission.resource),
                sys_permissions::action.eq(&permission.action),
                sys_permissions::updated_at.eq(permission.updated_at),
            ))
            .execute(&mut conn)?;

        Ok(permission)
    }

    pub fn delete(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(sys_permissions::table.filter(sys_permissions::id.eq(id.to_string())))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn list(pool: &DbPool, limit: i64, offset: i64) -> Result<Vec<Permission>> {
        let mut conn = pool.get()?;

        let permissions = sys_permissions::table
            .order(sys_permissions::created_at.desc())
            .limit(limit)
            .offset(offset)
            .select(Permission::as_select())
            .load(&mut conn)?;

        Ok(permissions)
    }

    pub fn assign_to_role(pool: &DbPool, role_id: String, permission_id: String) -> Result<()> {
        let role_permission_id = Ulid::new();
        let now = chrono::Utc::now();

        let mut conn = pool.get()?;

        diesel::insert_into(sys_model_has_permissions::table)
            .values((
                sys_model_has_permissions::id.eq(role_permission_id.to_string()),
                sys_model_has_permissions::model_type.eq("Role"),
                sys_model_has_permissions::model_id.eq(role_id.to_string()),
                sys_model_has_permissions::permission_id.eq(permission_id.to_string()),
                sys_model_has_permissions::scope_type.eq::<Option<String>>(None),
                sys_model_has_permissions::scope_id.eq::<Option<String>>(None),
                sys_model_has_permissions::created_at.eq(now),
                sys_model_has_permissions::updated_at.eq(now),
            ))
            .on_conflict((sys_model_has_permissions::model_type, sys_model_has_permissions::model_id, sys_model_has_permissions::permission_id))
            .do_nothing()
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn remove_from_role(pool: &DbPool, role_id: String, permission_id: String) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(
            sys_model_has_permissions::table
                .filter(sys_model_has_permissions::model_type.eq("Role"))
                .filter(sys_model_has_permissions::model_id.eq(role_id.to_string()))
                .filter(sys_model_has_permissions::permission_id.eq(permission_id.to_string()))
        )
        .execute(&mut conn)?;

        Ok(())
    }

    pub fn role_has_permission(pool: &DbPool, role_id: String, permission_name: &str, guard_name: Option<&str>) -> Result<bool> {
        let mut conn = pool.get()?;
        let guard = guard_name.unwrap_or("api");

        let count = sys_model_has_permissions::table
            .inner_join(sys_permissions::table.on(sys_model_has_permissions::permission_id.eq(sys_permissions::id)))
            .filter(sys_model_has_permissions::model_type.eq("Role"))
            .filter(sys_model_has_permissions::model_id.eq(role_id.to_string()))
            .filter(sys_permissions::name.eq(permission_name))
            .filter(sys_permissions::guard_name.eq(guard))
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(count > 0)
    }

    /// Generic method to check if a model has a specific permission
    pub fn model_has_permission<T: HasRoles>(pool: &DbPool, model: &T, permission_name: &str, guard_name: Option<&str>) -> Result<bool> {
        let mut conn = pool.get()?;
        let guard = guard_name.unwrap_or("api");

        let count = sys_model_has_roles::table
            .inner_join(sys_model_has_permissions::table.on(
                sys_model_has_roles::role_id.eq(sys_model_has_permissions::model_id)
                .and(sys_model_has_permissions::model_type.eq("Role"))
            ))
            .inner_join(sys_permissions::table.on(sys_model_has_permissions::permission_id.eq(sys_permissions::id)))
            .filter(sys_model_has_roles::model_type.eq(T::model_type()))
            .filter(sys_model_has_roles::model_id.eq(model.model_id()))
            .filter(sys_permissions::name.eq(permission_name))
            .filter(sys_permissions::guard_name.eq(guard))
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(count > 0)
    }

    pub fn get_role_permissions(pool: &DbPool, role_id: String, guard_name: Option<&str>) -> Result<Vec<Permission>> {
        let mut conn = pool.get()?;

        let mut query = sys_permissions::table
            .inner_join(sys_model_has_permissions::table.on(sys_permissions::id.eq(sys_model_has_permissions::permission_id)))
            .filter(sys_model_has_permissions::model_type.eq("Role"))
            .filter(sys_model_has_permissions::model_id.eq(role_id.to_string()))
            .into_boxed();

        if let Some(guard) = guard_name {
            query = query.filter(sys_permissions::guard_name.eq(guard));
        }

        let permissions = query
            .order(sys_permissions::name.asc())
            .select(Permission::as_select())
            .load(&mut conn)?;

        Ok(permissions)
    }

    /// Generic method to get permissions for any model that implements HasRoles
    pub fn get_model_permissions<T: HasRoles>(pool: &DbPool, model: &T, guard_name: Option<&str>) -> Result<Vec<Permission>> {
        let mut conn = pool.get()?;

        let mut query = sys_permissions::table
            .inner_join(sys_model_has_permissions::table.on(
                sys_permissions::id.eq(sys_model_has_permissions::permission_id)
                .and(sys_model_has_permissions::model_type.eq("Role"))
            ))
            .inner_join(sys_model_has_roles::table.on(sys_model_has_permissions::model_id.eq(sys_model_has_roles::role_id)))
            .filter(sys_model_has_roles::model_type.eq(T::model_type()))
            .filter(sys_model_has_roles::model_id.eq(model.model_id()))
            .into_boxed();

        if let Some(guard) = guard_name {
            query = query.filter(sys_permissions::guard_name.eq(guard));
        }

        let permissions = query
            .order(sys_permissions::name.asc())
            .select(Permission::as_select())
            .distinct()
            .load(&mut conn)?;

        Ok(permissions)
    }

    pub fn count(pool: &DbPool) -> Result<i64> {
        let mut conn = pool.get()?;

        let count = sys_permissions::table
            .count()
            .get_result::<i64>(&mut conn)?;

        Ok(count)
    }
}