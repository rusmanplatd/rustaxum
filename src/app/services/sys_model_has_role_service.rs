use anyhow::Result;
use crate::app::models::DieselUlid;
use crate::database::DbPool;
use diesel::prelude::*;
use serde_json::json;
use crate::schema::sys_model_has_roles;
use crate::app::traits::ServiceActivityLogger;

use crate::app::models::sys_model_has_role::{SysModelHasRole, CreateSysModelHasRole, UpdateSysModelHasRole};

pub struct SysModelHasRoleService;

impl ServiceActivityLogger for SysModelHasRoleService {}

impl SysModelHasRoleService {
    pub fn create(pool: &DbPool, data: CreateSysModelHasRole) -> Result<SysModelHasRole> {
        let role = SysModelHasRole::new(
            data.model_type,
            data.model_id,
            data.role_id,
            data.scope_type,
            data.scope_id,
        );


        let mut conn = pool.get()?;

        let result = diesel::insert_into(sys_model_has_roles::table)
            .values((
                sys_model_has_roles::id.eq(role.id.to_string()),
                sys_model_has_roles::model_type.eq(&role.model_type),
                sys_model_has_roles::model_id.eq(role.model_id.to_string()),
                sys_model_has_roles::role_id.eq(role.role_id.to_string()),
                sys_model_has_roles::scope_type.eq(&role.scope_type),
                sys_model_has_roles::scope_id.eq(role.scope_id.map(|id| id.to_string())),
                sys_model_has_roles::created_at.eq(role.created_at),
                sys_model_has_roles::updated_at.eq(role.updated_at),
            ))
            .get_result::<SysModelHasRole>(&mut conn)?;

        Ok(result)
    }

    pub fn find_by_id(pool: &DbPool, id: DieselUlid) -> Result<Option<SysModelHasRole>> {
        let mut conn = pool.get()?;

        let result = sys_model_has_roles::table
            .filter(sys_model_has_roles::id.eq(id.to_string()))
            .first::<SysModelHasRole>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_model(pool: &DbPool, model_type: &str, model_id: DieselUlid) -> Result<Vec<SysModelHasRole>> {
        let mut conn = pool.get()?;

        let result = sys_model_has_roles::table
            .filter(sys_model_has_roles::model_type.eq(model_type))
            .filter(sys_model_has_roles::model_id.eq(model_id.to_string()))
            .order(sys_model_has_roles::created_at.desc())
            .load::<SysModelHasRole>(&mut conn)?;

        Ok(result)
    }

    pub fn list(pool: &DbPool, _query_params: std::collections::HashMap<String, String>) -> Result<Vec<SysModelHasRole>> {
        let mut conn = pool.get()?;

        let result = sys_model_has_roles::table
            .order(sys_model_has_roles::created_at.desc())
            .load::<SysModelHasRole>(&mut conn)?;

        Ok(result)
    }

    pub fn update(pool: &DbPool, id: DieselUlid, data: UpdateSysModelHasRole) -> Result<SysModelHasRole> {
        let mut conn = pool.get()?;

        let query = r#"
            UPDATE sys_model_has_roles
            SET model_type = COALESCE($2, model_type),
                model_id = COALESCE($3, model_id),
                role_id = COALESCE($4, role_id),
                scope_type = COALESCE($5, scope_type),
                scope_id = COALESCE($6, scope_id),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, model_type, model_id, role_id, scope_type, scope_id, created_at, updated_at
        "#;

        let result = diesel::sql_query(query)
            .bind::<diesel::sql_types::Text, _>(id.to_string())
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(data.model_type)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(data.model_id.map(|id| id.to_string()))
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(data.role_id.map(|id| id.to_string()))
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(data.scope_type)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(data.scope_id.map(|id| id.to_string()))
            .get_result::<SysModelHasRole>(&mut conn)?;

        Ok(result)
    }

    pub fn delete(pool: &DbPool, id: DieselUlid) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(sys_model_has_roles::table)
            .filter(sys_model_has_roles::id.eq(id.to_string()))
            .execute(&mut conn)?;

        Ok(())
    }

    pub async fn assign_role_to_model(
        pool: &DbPool,
        model_type: &str,
        model_id: DieselUlid,
        role_id: DieselUlid,
        scope_type: Option<String>,
        scope_id: Option<DieselUlid>,
        assigned_by: Option<&str>,
    ) -> Result<SysModelHasRole> {
        let data = CreateSysModelHasRole {
            model_type: model_type.to_string(),
            model_id: model_id,
            role_id: role_id,
            scope_type: scope_type.clone(),
            scope_id: scope_id,
        };
        let result = Self::create(pool, data)?;

        // Log role assignment activity
        let service = Self;
        let properties = json!({
            "model_type": model_type,
            "model_id": model_id.to_string(),
            "role_id": role_id.to_string(),
            "scope_type": scope_type,
            "scope_id": scope_id.map(|id| id.to_string()),
            "action": "role_assigned"
        });

        if let Err(e) = service.log_system_event(
            "role_assigned_to_model",
            &format!("Role {} assigned to {} {}", role_id, model_type, model_id),
            Some(properties)
        ).await {
            eprintln!("Failed to log role assignment activity: {}", e);
        }

        Ok(result)
    }

    pub async fn remove_role_from_model(
        pool: &DbPool,
        model_type: &str,
        model_id: String,
        role_id: String,
        removed_by: Option<&str>,
    ) -> Result<()> {
        let mut conn = pool.get()?;

        let rows_affected = diesel::delete(sys_model_has_roles::table)
            .filter(sys_model_has_roles::model_type.eq(model_type))
            .filter(sys_model_has_roles::model_id.eq(&model_id))
            .filter(sys_model_has_roles::role_id.eq(&role_id))
            .execute(&mut conn)?;

        if rows_affected > 0 {
            // Log role removal activity
            let service = Self;
            let properties = json!({
                "model_type": model_type,
                "model_id": model_id,
                "role_id": role_id,
                "action": "role_removed"
            });

            if let Err(e) = service.log_system_event(
                "role_removed_from_model",
                &format!("Role {} removed from {} {}", role_id, model_type, model_id),
                Some(properties)
            ).await {
                eprintln!("Failed to log role removal activity: {}", e);
            }
        }

        Ok(())
    }

    pub fn get_model_roles(
        pool: &DbPool,
        model_type: &str,
        model_id: String,
        guard_name: Option<&str>,
    ) -> Result<Vec<crate::app::models::role::Role>> {
        use crate::app::models::role::Role;
        use crate::schema::sys_roles;
        let mut conn = pool.get()?;

        let mut query = sys_roles::table
            .inner_join(sys_model_has_roles::table.on(sys_roles::id.eq(sys_model_has_roles::role_id)))
            .filter(sys_model_has_roles::model_type.eq(model_type))
            .filter(sys_model_has_roles::model_id.eq(model_id))
            .select(sys_roles::all_columns)
            .order(sys_roles::name)
            .into_boxed();

        if let Some(guard) = guard_name {
            query = query.filter(sys_roles::guard_name.eq(guard));
        }

        let result = query.select(Role::as_select()).load(&mut conn)?;
        Ok(result)
    }
}