use anyhow::Result;
use crate::database::DbPool;
use diesel::prelude::*;
use serde_json::json;
use crate::schema::sys_model_has_permissions;
use crate::app::models::DieselUlid;
use crate::app::traits::ServiceActivityLogger;

use crate::app::models::sys_model_has_permission::{SysModelHasPermission, CreateSysModelHasPermission, UpdateSysModelHasPermission};

pub struct SysModelHasPermissionService;

impl ServiceActivityLogger for SysModelHasPermissionService {}

impl SysModelHasPermissionService {
    pub fn create(pool: &DbPool, data: CreateSysModelHasPermission) -> Result<SysModelHasPermission> {
        let permission = SysModelHasPermission::new(
            data.model_type,
            data.model_id,
            data.permission_id,
            data.scope_type,
            data.scope_id,
        );

        let mut conn = pool.get()?;

        let result = diesel::insert_into(sys_model_has_permissions::table)
            .values((
                sys_model_has_permissions::id.eq(permission.id),
                sys_model_has_permissions::model_type.eq(&permission.model_type),
                sys_model_has_permissions::model_id.eq(permission.model_id),
                sys_model_has_permissions::permission_id.eq(permission.permission_id),
                sys_model_has_permissions::scope_type.eq(&permission.scope_type),
                sys_model_has_permissions::scope_id.eq(permission.scope_id),
                sys_model_has_permissions::created_at.eq(permission.created_at),
                sys_model_has_permissions::updated_at.eq(permission.updated_at),
            ))
            .get_result::<SysModelHasPermission>(&mut conn)?;

        Ok(result)
    }

    pub fn find_by_id(pool: &DbPool, id: DieselUlid) -> Result<Option<SysModelHasPermission>> {
        let mut conn = pool.get()?;

        let result = sys_model_has_permissions::table
            .filter(sys_model_has_permissions::id.eq(id))
            .first::<SysModelHasPermission>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_model(pool: &DbPool, model_type: &str, model_id: DieselUlid) -> Result<Vec<SysModelHasPermission>> {
        let mut conn = pool.get()?;

        let result = sys_model_has_permissions::table
            .filter(sys_model_has_permissions::model_type.eq(model_type))
            .filter(sys_model_has_permissions::model_id.eq(model_id))
            .order(sys_model_has_permissions::created_at.desc())
            .load::<SysModelHasPermission>(&mut conn)?;

        Ok(result)
    }

    pub fn list(pool: &DbPool, _query_params: std::collections::HashMap<String, String>) -> Result<Vec<SysModelHasPermission>> {
        let mut conn = pool.get()?;

        let result = sys_model_has_permissions::table
            .order(sys_model_has_permissions::created_at.desc())
            .load::<SysModelHasPermission>(&mut conn)?;

        Ok(result)
    }

    pub fn update(pool: &DbPool, id: DieselUlid, data: UpdateSysModelHasPermission) -> Result<SysModelHasPermission> {
        let mut conn = pool.get()?;

        // Use raw SQL for COALESCE functionality
        use diesel::sql_query;
        let result = sql_query(
            r#"
            UPDATE sys_model_has_permissions
            SET model_type = COALESCE($2, model_type),
                model_id = COALESCE($3, model_id),
                permission_id = COALESCE($4, permission_id),
                scope_type = COALESCE($5, scope_type),
                scope_id = COALESCE($6, scope_id),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, model_type, model_id, permission_id, scope_type, scope_id, created_at, updated_at
            "#
        )
        .bind::<diesel::sql_types::Text, _>(id.to_string())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(data.model_type)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(data.model_id.map(|id| id.to_string()))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(data.permission_id.map(|id| id.to_string()))
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(data.scope_type)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(data.scope_id.map(|id| id.to_string()))
        .get_result::<SysModelHasPermission>(&mut conn)?;

        Ok(result)
    }

    pub fn delete(pool: &DbPool, id: DieselUlid) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(sys_model_has_permissions::table)
            .filter(sys_model_has_permissions::id.eq(id))
            .execute(&mut conn)?;

        Ok(())
    }

    pub async fn assign_permission_to_model(
        pool: &DbPool,
        model_type: &str,
        model_id: DieselUlid,
        permission_id: DieselUlid,
        scope_type: Option<String>,
        scope_id: Option<DieselUlid>,
        assigned_by: Option<&str>,
    ) -> Result<SysModelHasPermission> {
        let data = CreateSysModelHasPermission {
            model_type: model_type.to_string(),
            model_id,
            permission_id,
            scope_type: scope_type.clone(),
            scope_id,
        };
        let result = Self::create(pool, data)?;

        // Log permission assignment activity
        let service = Self;
        let properties = json!({
            "model_type": model_type,
            "model_id": model_id.to_string(),
            "permission_id": permission_id.to_string(),
            "scope_type": scope_type,
            "scope_id": scope_id.map(|id| id.to_string()),
            "action": "permission_assigned"
        });

        if let Err(e) = service.log_system_event(
            "permission_assigned_to_model",
            &format!("Permission {} assigned to {} {}", permission_id, model_type, model_id),
            Some(properties)
        ).await {
            eprintln!("Failed to log permission assignment activity: {}", e);
        }

        Ok(result)
    }

    pub async fn remove_permission_from_model(
        pool: &DbPool,
        model_type: &str,
        model_id: DieselUlid,
        permission_id: DieselUlid,
        removed_by: Option<&str>,
    ) -> Result<()> {
        let mut conn = pool.get()?;

        let rows_affected = diesel::delete(sys_model_has_permissions::table)
            .filter(sys_model_has_permissions::model_type.eq(model_type))
            .filter(sys_model_has_permissions::model_id.eq(model_id))
            .filter(sys_model_has_permissions::permission_id.eq(permission_id))
            .execute(&mut conn)?;

        if rows_affected > 0 {
            // Log permission removal activity
            let service = Self;
            let properties = json!({
                "model_type": model_type,
                "model_id": model_id.to_string(),
                "permission_id": permission_id.to_string(),
                "action": "permission_removed"
            });

            if let Err(e) = service.log_system_event(
                "permission_removed_from_model",
                &format!("Permission {} removed from {} {}", permission_id, model_type, model_id),
                Some(properties)
            ).await {
                eprintln!("Failed to log permission removal activity: {}", e);
            }
        }

        Ok(())
    }

    pub fn get_model_permissions(
        pool: &DbPool,
        model_type: &str,
        model_id: DieselUlid,
        guard_name: Option<&str>,
    ) -> Result<Vec<crate::app::models::permission::Permission>> {
        use crate::app::models::permission::Permission;
        use crate::schema::{sys_permissions, sys_model_has_permissions};

        let mut conn = pool.get()?;

        let mut query = sys_permissions::table
            .inner_join(sys_model_has_permissions::table.on(
                sys_permissions::id.eq(sys_model_has_permissions::permission_id)
            ))
            .filter(sys_model_has_permissions::model_type.eq(model_type))
            .filter(sys_model_has_permissions::model_id.eq(model_id))
            .into_boxed();

        if let Some(guard) = guard_name {
            query = query.filter(sys_permissions::guard_name.eq(guard));
        }

        let result = query
            .select(sys_permissions::all_columns)
            .order(sys_permissions::name.asc())
            .load::<Permission>(&mut conn)?;

        Ok(result)
    }
}