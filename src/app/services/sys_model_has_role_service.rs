use anyhow::Result;
use ulid::Ulid;
use crate::database::DbPool;
use diesel::prelude::*;
use crate::schema::sys_model_has_roles;

use crate::app::models::sys_model_has_role::{SysModelHasRole, CreateSysModelHasRole, UpdateSysModelHasRole};

pub struct SysModelHasRoleService;

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

    pub fn find_by_id(pool: &DbPool, id: Ulid) -> Result<Option<SysModelHasRole>> {
        let mut conn = pool.get()?;

        let result = sys_model_has_roles::table
            .filter(sys_model_has_roles::id.eq(id.to_string()))
            .first::<SysModelHasRole>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_model(pool: &DbPool, model_type: &str, model_id: Ulid) -> Result<Vec<SysModelHasRole>> {
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

    pub fn update(pool: &DbPool, id: Ulid, data: UpdateSysModelHasRole) -> Result<SysModelHasRole> {
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

    pub fn delete(pool: &DbPool, id: Ulid) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(sys_model_has_roles::table)
            .filter(sys_model_has_roles::id.eq(id.to_string()))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn assign_role_to_model(
        pool: &DbPool,
        model_type: &str,
        model_id: Ulid,
        role_id: Ulid,
        scope_type: Option<String>,
        scope_id: Option<Ulid>,
    ) -> Result<SysModelHasRole> {
        let data = CreateSysModelHasRole {
            model_type: model_type.to_string(),
            model_id,
            role_id,
            scope_type,
            scope_id,
        };
        Self::create(pool, data)
    }

    pub fn remove_role_from_model(
        pool: &DbPool,
        model_type: &str,
        model_id: Ulid,
        role_id: Ulid,
    ) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(sys_model_has_roles::table)
            .filter(sys_model_has_roles::model_type.eq(model_type))
            .filter(sys_model_has_roles::model_id.eq(model_id.to_string()))
            .filter(sys_model_has_roles::role_id.eq(role_id.to_string()))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn get_model_roles(
        pool: &DbPool,
        model_type: &str,
        model_id: Ulid,
        guard_name: Option<&str>,
    ) -> Result<Vec<crate::app::models::role::Role>> {
        use crate::app::models::role::Role;
        use crate::schema::sys_roles;
        let mut conn = pool.get()?;

        let mut query = sys_roles::table
            .inner_join(sys_model_has_roles::table.on(sys_roles::id.eq(sys_model_has_roles::role_id)))
            .filter(sys_model_has_roles::model_type.eq(model_type))
            .filter(sys_model_has_roles::model_id.eq(model_id.to_string()))
            .select(sys_roles::all_columns)
            .order(sys_roles::name)
            .into_boxed();

        if let Some(guard) = guard_name {
            query = query.filter(sys_roles::guard_name.eq(guard));
        }

        let result = query.load::<Role>(&mut conn)?;
        Ok(result)
    }
}