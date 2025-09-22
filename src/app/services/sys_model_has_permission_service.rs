use anyhow::Result;
use crate::database::DbPool;
use diesel::prelude::*;
use crate::schema::sys_model_has_permissions;

use crate::app::models::sys_model_has_permission::{SysModelHasPermission, CreateSysModelHasPermission, UpdateSysModelHasPermission};

pub struct SysModelHasPermissionService;

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
                sys_model_has_permissions::id.eq(permission.id.to_string()),
                sys_model_has_permissions::model_type.eq(&permission.model_type),
                sys_model_has_permissions::model_id.eq(permission.model_id.to_string()),
                sys_model_has_permissions::permission_id.eq(permission.permission_id.to_string()),
                sys_model_has_permissions::scope_type.eq(&permission.scope_type),
                sys_model_has_permissions::scope_id.eq(permission.scope_id.map(|id| id.to_string())),
                sys_model_has_permissions::created_at.eq(permission.created_at),
                sys_model_has_permissions::updated_at.eq(permission.updated_at),
            ))
            .get_result::<SysModelHasPermission>(&mut conn)?;

        Ok(result)
    }

    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<SysModelHasPermission>> {
        let mut conn = pool.get()?;

        let result = sys_model_has_permissions::table
            .filter(sys_model_has_permissions::id.eq(id.to_string()))
            .first::<SysModelHasPermission>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_model(pool: &DbPool, model_type: &str, model_id: String) -> Result<Vec<SysModelHasPermission>> {
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

    pub fn update(pool: &DbPool, id: String, data: UpdateSysModelHasPermission) -> Result<SysModelHasPermission> {
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

    pub fn delete(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(sys_model_has_permissions::table)
            .filter(sys_model_has_permissions::id.eq(id.to_string()))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn assign_permission_to_model(
        pool: &DbPool,
        model_type: &str,
        model_id: String,
        permission_id: String,
        scope_type: Option<String>,
        scope_id: Option<String>,
    ) -> Result<SysModelHasPermission> {
        let data = CreateSysModelHasPermission {
            model_type: model_type.to_string(),
            model_id,
            permission_id,
            scope_type,
            scope_id,
        };
        Self::create(pool, data)
    }

    pub fn remove_permission_from_model(
        pool: &DbPool,
        model_type: &str,
        model_id: String,
        permission_id: String,
    ) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(sys_model_has_permissions::table)
            .filter(sys_model_has_permissions::model_type.eq(model_type))
            .filter(sys_model_has_permissions::model_id.eq(model_id.to_string()))
            .filter(sys_model_has_permissions::permission_id.eq(permission_id.to_string()))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn get_model_permissions(
        pool: &DbPool,
        model_type: &str,
        model_id: String,
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
            .filter(sys_model_has_permissions::model_id.eq(model_id.to_string()))
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