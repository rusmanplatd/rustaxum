use diesel::prelude::*;
use diesel::pg::PgConnection;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Centralized role & permission relationship loader for complex authorization queries
pub struct RolePermissionLoader;

impl RolePermissionLoader {
    /// Load role & permission relationships for any model
    pub fn load_role_permission_relationships(
        table_name: &str,
        ids: &[String],
        relationships: &[String],
        conn: &mut PgConnection,
    ) -> Result<HashMap<String, Value>> {
        let mut results = HashMap::new();

        for relationship in relationships {
            match relationship.as_str() {
                // Direct role relationships
                "roles" => {
                    let data = Self::load_model_roles(table_name, ids, conn)?;
                    results.insert("roles".to_string(), data);
                },
                "roles.permissions" => {
                    let data = Self::load_model_roles_with_permissions(table_name, ids, conn)?;
                    results.insert("roles.permissions".to_string(), data);
                },

                // Direct permission relationships
                "permissions" => {
                    let data = Self::load_model_permissions(table_name, ids, conn)?;
                    results.insert("permissions".to_string(), data);
                },
                "permissions.roles" => {
                    let data = Self::load_model_permissions_with_roles(table_name, ids, conn)?;
                    results.insert("permissions.roles".to_string(), data);
                },

                // Role hierarchy and organization context
                "roles.organization" => {
                    let data = Self::load_roles_with_organization(table_name, ids, conn)?;
                    results.insert("roles.organization".to_string(), data);
                },
                "permissions.organization" => {
                    let data = Self::load_permissions_with_organization(table_name, ids, conn)?;
                    results.insert("permissions.organization".to_string(), data);
                },

                // Complete authorization context
                "authorizationContext" => {
                    let data = Self::load_complete_authorization_context(table_name, ids, conn)?;
                    results.insert("authorizationContext".to_string(), data);
                },

                // Scoped roles and permissions
                "scopedRoles" => {
                    let data = Self::load_scoped_roles(table_name, ids, conn)?;
                    results.insert("scopedRoles".to_string(), data);
                },
                "scopedPermissions" => {
                    let data = Self::load_scoped_permissions(table_name, ids, conn)?;
                    results.insert("scopedPermissions".to_string(), data);
                },

                _ => {
                    tracing::debug!("Unknown role/permission relationship: {}", relationship);
                }
            }
        }

        Ok(results)
    }

    /// Load all roles assigned to models
    pub fn load_model_roles(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let model_type = Self::get_model_type_from_table(table_name);
        let query = format!(
            "SELECT {}.id as model_id,
                    r.id as role_id, r.name as role_name, r.description as role_description,
                    r.guard_name, r.scope_type as role_scope_type, r.scope_id as role_scope_id,
                    mhr.scope_type as assignment_scope_type, mhr.scope_id as assignment_scope_id,
                    mhr.created_at as assigned_at
             FROM {}
             LEFT JOIN sys_model_has_roles mhr ON {}.id = mhr.model_id AND mhr.model_type = '{}'
             LEFT JOIN sys_roles r ON mhr.role_id = r.id AND r.deleted_at IS NULL
             WHERE {}.id = ANY($1) AND mhr.deleted_at IS NULL",
            table_name, table_name, table_name, model_type, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<RoleRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    /// Load all permissions assigned to models (both direct and through roles)
    pub fn load_model_permissions(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let model_type = Self::get_model_type_from_table(table_name);
        let query = format!(
            "SELECT DISTINCT {}.id as model_id,
                    p.id as permission_id, p.action, p.resource, p.guard_name,
                    p.scope_type as permission_scope_type, p.scope_id as permission_scope_id,
                    CASE
                        WHEN mhp.id IS NOT NULL THEN 'direct'
                        WHEN rhp.id IS NOT NULL THEN 'role'
                        ELSE 'unknown'
                    END as assignment_type,
                    COALESCE(mhp.scope_type, mhr.scope_type) as assignment_scope_type,
                    COALESCE(mhp.scope_id, mhr.scope_id) as assignment_scope_id
             FROM {}
             LEFT JOIN sys_model_has_permissions mhp ON {}.id = mhp.model_id AND mhp.model_type = '{}'
             LEFT JOIN sys_permissions p1 ON mhp.permission_id = p1.id AND p1.deleted_at IS NULL AND mhp.deleted_at IS NULL
             LEFT JOIN sys_model_has_roles mhr ON {}.id = mhr.model_id AND mhr.model_type = '{}'
             LEFT JOIN sys_roles r ON mhr.role_id = r.id AND r.deleted_at IS NULL AND mhr.deleted_at IS NULL
             LEFT JOIN sys_role_has_permissions rhp ON r.id = rhp.role_id AND rhp.deleted_at IS NULL
             LEFT JOIN sys_permissions p2 ON rhp.permission_id = p2.id AND p2.deleted_at IS NULL
             LEFT JOIN sys_permissions p ON COALESCE(p1.id, p2.id) = p.id
             WHERE {}.id = ANY($1) AND p.id IS NOT NULL",
            table_name, table_name, table_name, model_type, table_name, model_type, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<PermissionRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    /// Load roles with their associated permissions
    pub fn load_model_roles_with_permissions(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let model_type = Self::get_model_type_from_table(table_name);
        let query = format!(
            "SELECT {}.id as model_id,
                    r.id as role_id, r.name as role_name, r.description as role_description,
                    r.guard_name as role_guard_name,
                    p.id as permission_id, p.action, p.resource, p.guard_name as permission_guard_name,
                    mhr.scope_type as assignment_scope_type, mhr.scope_id as assignment_scope_id
             FROM {}
             LEFT JOIN sys_model_has_roles mhr ON {}.id = mhr.model_id AND mhr.model_type = '{}'
             LEFT JOIN sys_roles r ON mhr.role_id = r.id AND r.deleted_at IS NULL
             LEFT JOIN sys_role_has_permissions rhp ON r.id = rhp.role_id AND rhp.deleted_at IS NULL
             LEFT JOIN sys_permissions p ON rhp.permission_id = p.id AND p.deleted_at IS NULL
             WHERE {}.id = ANY($1) AND mhr.deleted_at IS NULL",
            table_name, table_name, table_name, model_type, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<RolePermissionRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    /// Load permissions with their associated roles
    pub fn load_model_permissions_with_roles(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let model_type = Self::get_model_type_from_table(table_name);
        let query = format!(
            "SELECT DISTINCT {}.id as model_id,
                    p.id as permission_id, p.action, p.resource, p.guard_name as permission_guard_name,
                    r.id as role_id, r.name as role_name, r.guard_name as role_guard_name,
                    CASE
                        WHEN mhp.id IS NOT NULL THEN 'direct'
                        WHEN rhp.id IS NOT NULL THEN 'role'
                        ELSE 'unknown'
                    END as assignment_type
             FROM {}
             LEFT JOIN sys_model_has_permissions mhp ON {}.id = mhp.model_id AND mhp.model_type = '{}' AND mhp.deleted_at IS NULL
             LEFT JOIN sys_permissions p1 ON mhp.permission_id = p1.id AND p1.deleted_at IS NULL
             LEFT JOIN sys_model_has_roles mhr ON {}.id = mhr.model_id AND mhr.model_type = '{}' AND mhr.deleted_at IS NULL
             LEFT JOIN sys_roles r ON mhr.role_id = r.id AND r.deleted_at IS NULL
             LEFT JOIN sys_role_has_permissions rhp ON r.id = rhp.role_id AND rhp.deleted_at IS NULL
             LEFT JOIN sys_permissions p2 ON rhp.permission_id = p2.id AND p2.deleted_at IS NULL
             LEFT JOIN sys_permissions p ON COALESCE(p1.id, p2.id) = p.id
             WHERE {}.id = ANY($1) AND p.id IS NOT NULL",
            table_name, table_name, table_name, model_type, table_name, model_type, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<PermissionRoleRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    /// Load roles with organization context
    pub fn load_roles_with_organization(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let model_type = Self::get_model_type_from_table(table_name);
        let query = format!(
            "SELECT {}.id as model_id,
                    r.id as role_id, r.name as role_name, r.description as role_description,
                    o.id as org_id, o.name as org_name,
                    mhr.scope_type as assignment_scope_type, mhr.scope_id as assignment_scope_id
             FROM {}
             LEFT JOIN sys_model_has_roles mhr ON {}.id = mhr.model_id AND mhr.model_type = '{}'
             LEFT JOIN sys_roles r ON mhr.role_id = r.id AND r.deleted_at IS NULL
             LEFT JOIN organizations o ON r.organization_id = o.id
             WHERE {}.id = ANY($1) AND mhr.deleted_at IS NULL",
            table_name, table_name, table_name, model_type, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<RoleOrganizationRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    /// Load permissions with organization context
    pub fn load_permissions_with_organization(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let model_type = Self::get_model_type_from_table(table_name);
        let query = format!(
            "SELECT DISTINCT {}.id as model_id,
                    p.id as permission_id, p.action, p.resource,
                    o.id as org_id, o.name as org_name
             FROM {}
             LEFT JOIN sys_model_has_permissions mhp ON {}.id = mhp.model_id AND mhp.model_type = '{}' AND mhp.deleted_at IS NULL
             LEFT JOIN sys_permissions p1 ON mhp.permission_id = p1.id AND p1.deleted_at IS NULL
             LEFT JOIN sys_model_has_roles mhr ON {}.id = mhr.model_id AND mhr.model_type = '{}' AND mhr.deleted_at IS NULL
             LEFT JOIN sys_roles r ON mhr.role_id = r.id AND r.deleted_at IS NULL
             LEFT JOIN sys_role_has_permissions rhp ON r.id = rhp.role_id AND rhp.deleted_at IS NULL
             LEFT JOIN sys_permissions p2 ON rhp.permission_id = p2.id AND p2.deleted_at IS NULL
             LEFT JOIN sys_permissions p ON COALESCE(p1.id, p2.id) = p.id
             LEFT JOIN organizations o ON p.organization_id = o.id
             WHERE {}.id = ANY($1) AND p.id IS NOT NULL",
            table_name, table_name, table_name, model_type, table_name, model_type, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<PermissionOrganizationRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    /// Load complete authorization context (roles, permissions, and organization info)
    pub fn load_complete_authorization_context(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let model_type = Self::get_model_type_from_table(table_name);
        let query = format!(
            "SELECT {}.id as model_id,
                    r.id as role_id, r.name as role_name, r.description as role_description,
                    p.id as permission_id, p.action, p.resource,
                    o.id as org_id, o.name as org_name,
                    mhr.scope_type as role_scope_type, mhr.scope_id as role_scope_id,
                    CASE
                        WHEN mhp.id IS NOT NULL THEN 'direct_permission'
                        WHEN rhp.id IS NOT NULL THEN 'role_permission'
                        WHEN mhr.id IS NOT NULL THEN 'role_only'
                        ELSE 'none'
                    END as authorization_type
             FROM {}
             LEFT JOIN sys_model_has_roles mhr ON {}.id = mhr.model_id AND mhr.model_type = '{}' AND mhr.deleted_at IS NULL
             LEFT JOIN sys_roles r ON mhr.role_id = r.id AND r.deleted_at IS NULL
             LEFT JOIN sys_role_has_permissions rhp ON r.id = rhp.role_id AND rhp.deleted_at IS NULL
             LEFT JOIN sys_permissions p1 ON rhp.permission_id = p1.id AND p1.deleted_at IS NULL
             LEFT JOIN sys_model_has_permissions mhp ON {}.id = mhp.model_id AND mhp.model_type = '{}' AND mhp.deleted_at IS NULL
             LEFT JOIN sys_permissions p2 ON mhp.permission_id = p2.id AND p2.deleted_at IS NULL
             LEFT JOIN sys_permissions p ON COALESCE(p1.id, p2.id) = p.id
             LEFT JOIN organizations o ON COALESCE(r.organization_id, p.organization_id) = o.id
             WHERE {}.id = ANY($1)",
            table_name, table_name, table_name, model_type, table_name, model_type, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<AuthorizationContextRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    /// Load scoped roles (with scope filtering)
    pub fn load_scoped_roles(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let model_type = Self::get_model_type_from_table(table_name);
        let query = format!(
            "SELECT {}.id as model_id,
                    r.id as role_id, r.name as role_name,
                    r.scope_type as role_scope_type, r.scope_id as role_scope_id,
                    mhr.scope_type as assignment_scope_type, mhr.scope_id as assignment_scope_id,
                    CASE
                        WHEN r.scope_type IS NOT NULL THEN 'role_scoped'
                        WHEN mhr.scope_type IS NOT NULL THEN 'assignment_scoped'
                        ELSE 'global'
                    END as scope_level
             FROM {}
             LEFT JOIN sys_model_has_roles mhr ON {}.id = mhr.model_id AND mhr.model_type = '{}'
             LEFT JOIN sys_roles r ON mhr.role_id = r.id AND r.deleted_at IS NULL
             WHERE {}.id = ANY($1) AND mhr.deleted_at IS NULL",
            table_name, table_name, table_name, model_type, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<ScopedRoleRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    /// Load scoped permissions (with scope filtering)
    pub fn load_scoped_permissions(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let model_type = Self::get_model_type_from_table(table_name);
        let query = format!(
            "SELECT DISTINCT {}.id as model_id,
                    p.id as permission_id, p.action, p.resource,
                    p.scope_type as permission_scope_type, p.scope_id as permission_scope_id,
                    COALESCE(mhp.scope_type, mhr.scope_type) as assignment_scope_type,
                    COALESCE(mhp.scope_id, mhr.scope_id) as assignment_scope_id,
                    CASE
                        WHEN p.scope_type IS NOT NULL THEN 'permission_scoped'
                        WHEN COALESCE(mhp.scope_type, mhr.scope_type) IS NOT NULL THEN 'assignment_scoped'
                        ELSE 'global'
                    END as scope_level,
                    CASE
                        WHEN mhp.id IS NOT NULL THEN 'direct'
                        WHEN rhp.id IS NOT NULL THEN 'role'
                        ELSE 'unknown'
                    END as assignment_type
             FROM {}
             LEFT JOIN sys_model_has_permissions mhp ON {}.id = mhp.model_id AND mhp.model_type = '{}' AND mhp.deleted_at IS NULL
             LEFT JOIN sys_permissions p1 ON mhp.permission_id = p1.id AND p1.deleted_at IS NULL
             LEFT JOIN sys_model_has_roles mhr ON {}.id = mhr.model_id AND mhr.model_type = '{}' AND mhr.deleted_at IS NULL
             LEFT JOIN sys_roles r ON mhr.role_id = r.id AND r.deleted_at IS NULL
             LEFT JOIN sys_role_has_permissions rhp ON r.id = rhp.role_id AND rhp.deleted_at IS NULL
             LEFT JOIN sys_permissions p2 ON rhp.permission_id = p2.id AND p2.deleted_at IS NULL
             LEFT JOIN sys_permissions p ON COALESCE(p1.id, p2.id) = p.id
             WHERE {}.id = ANY($1) AND p.id IS NOT NULL",
            table_name, table_name, table_name, model_type, table_name, model_type, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<ScopedPermissionRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    /// Get model type from table name (for sys_model_has_* queries)
    fn get_model_type_from_table(table_name: &str) -> &str {
        match table_name {
            "sys_users" => "User",
            "organizations" => "Organization",
            "countries" => "Country",
            "provinces" => "Province",
            "cities" => "City",
            "districts" => "District",
            "villages" => "Village",
            "sys_roles" => "Role",
            "sys_permissions" => "Permission",
            _ => "Unknown"
        }
    }
}

/// Record types for role & permission queries
#[derive(Debug, serde::Serialize, serde::Deserialize, QueryableByName)]
pub struct RoleRecord {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub model_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub guard_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_scope_type: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_scope_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub assignment_scope_type: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub assignment_scope_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub assigned_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, QueryableByName)]
pub struct PermissionRecord {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub model_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub permission_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub action: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub resource: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub guard_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub permission_scope_type: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub permission_scope_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub assignment_type: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub assignment_scope_type: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub assignment_scope_id: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, QueryableByName)]
pub struct RolePermissionRecord {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub model_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_guard_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub permission_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub action: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub resource: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub permission_guard_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub assignment_scope_type: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub assignment_scope_id: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, QueryableByName)]
pub struct PermissionRoleRecord {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub model_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub permission_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub action: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub resource: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub permission_guard_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_guard_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub assignment_type: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, QueryableByName)]
pub struct RoleOrganizationRecord {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub model_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub org_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub org_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub assignment_scope_type: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub assignment_scope_id: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, QueryableByName)]
pub struct PermissionOrganizationRecord {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub model_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub permission_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub action: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub resource: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub org_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub org_name: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, QueryableByName)]
pub struct AuthorizationContextRecord {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub model_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub permission_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub action: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub resource: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub org_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub org_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_scope_type: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_scope_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub authorization_type: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, QueryableByName)]
pub struct ScopedRoleRecord {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub model_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_scope_type: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub role_scope_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub assignment_scope_type: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub assignment_scope_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub scope_level: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, QueryableByName)]
pub struct ScopedPermissionRecord {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub model_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub permission_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub action: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub resource: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub permission_scope_type: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub permission_scope_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub assignment_scope_type: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub assignment_scope_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub scope_level: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub assignment_type: Option<String>,
}