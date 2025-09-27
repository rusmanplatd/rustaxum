use diesel::prelude::*;
use diesel::pg::PgConnection;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Centralized audit relationship loader for deep audit trails
pub struct AuditRelationshipLoader;

impl AuditRelationshipLoader {
    /// Load audit relationships for any model with audit fields
    pub fn load_audit_relationships(
        table_name: &str,
        ids: &[String],
        relationships: &[String],
        conn: &mut PgConnection,
    ) -> Result<HashMap<String, Value>> {
        let mut results = HashMap::new();

        for relationship in relationships {
            match relationship.as_str() {
                "createdBy" => {
                    let data = Self::load_created_by_users(table_name, ids, conn)?;
                    results.insert("createdBy".to_string(), data);
                },
                "updatedBy" => {
                    let data = Self::load_updated_by_users(table_name, ids, conn)?;
                    results.insert("updatedBy".to_string(), data);
                },
                "deletedBy" => {
                    let data = Self::load_deleted_by_users(table_name, ids, conn)?;
                    results.insert("deletedBy".to_string(), data);
                },
                "createdBy.organizations" => {
                    let data = Self::load_created_by_organizations(table_name, ids, conn)?;
                    results.insert("createdBy.organizations".to_string(), data);
                },
                "updatedBy.organizations" => {
                    let data = Self::load_updated_by_organizations(table_name, ids, conn)?;
                    results.insert("updatedBy.organizations".to_string(), data);
                },
                "deletedBy.organizations" => {
                    let data = Self::load_deleted_by_organizations(table_name, ids, conn)?;
                    results.insert("deletedBy.organizations".to_string(), data);
                },
                "createdBy.organizations.position" => {
                    let data = Self::load_created_by_positions(table_name, ids, conn)?;
                    results.insert("createdBy.organizations.position".to_string(), data);
                },
                "updatedBy.organizations.position" => {
                    let data = Self::load_updated_by_positions(table_name, ids, conn)?;
                    results.insert("updatedBy.organizations.position".to_string(), data);
                },
                "deletedBy.organizations.position" => {
                    let data = Self::load_deleted_by_positions(table_name, ids, conn)?;
                    results.insert("deletedBy.organizations.position".to_string(), data);
                },
                "createdBy.organizations.position.level" => {
                    let data = Self::load_created_by_position_levels(table_name, ids, conn)?;
                    results.insert("createdBy.organizations.position.level".to_string(), data);
                },
                "updatedBy.organizations.position.level" => {
                    let data = Self::load_updated_by_position_levels(table_name, ids, conn)?;
                    results.insert("updatedBy.organizations.position.level".to_string(), data);
                },
                "deletedBy.organizations.position.level" => {
                    let data = Self::load_deleted_by_position_levels(table_name, ids, conn)?;
                    results.insert("deletedBy.organizations.position.level".to_string(), data);
                },
                _ => {
                    tracing::debug!("Unknown audit relationship: {}", relationship);
                }
            }
        }

        Ok(results)
    }

    /// Generate SQL query for loading audit user data
    pub fn load_created_by_users(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let query = format!(
            "SELECT {}.id as record_id, u.id, u.name, u.email
             FROM {}
             LEFT JOIN sys_users u ON {}.created_by_id = u.id
             WHERE {}.id = ANY($1)",
            table_name, table_name, table_name, table_name
        );

        // Execute raw SQL query
        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<AuditUserRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    pub fn load_updated_by_users(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let query = format!(
            "SELECT {}.id as record_id, u.id, u.name, u.email
             FROM {}
             LEFT JOIN sys_users u ON {}.updated_by_id = u.id
             WHERE {}.id = ANY($1)",
            table_name, table_name, table_name, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<AuditUserRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    pub fn load_deleted_by_users(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let query = format!(
            "SELECT {}.id as record_id, u.id, u.name, u.email
             FROM {}
             LEFT JOIN sys_users u ON {}.deleted_by_id = u.id
             WHERE {}.id = ANY($1)",
            table_name, table_name, table_name, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<AuditUserRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    pub fn load_created_by_organizations(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let query = format!(
            "SELECT {}.id as record_id, o.id as org_id, o.name as org_name
             FROM {}
             LEFT JOIN sys_users u ON {}.created_by_id = u.id
             LEFT JOIN user_organizations uo ON u.id = uo.user_id
             LEFT JOIN organizations o ON uo.organization_id = o.id
             WHERE {}.id = ANY($1)",
            table_name, table_name, table_name, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<AuditOrgRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    pub fn load_updated_by_organizations(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let query = format!(
            "SELECT {}.id as record_id, o.id as org_id, o.name as org_name
             FROM {}
             LEFT JOIN sys_users u ON {}.updated_by_id = u.id
             LEFT JOIN user_organizations uo ON u.id = uo.user_id
             LEFT JOIN organizations o ON uo.organization_id = o.id
             WHERE {}.id = ANY($1)",
            table_name, table_name, table_name, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<AuditOrgRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    pub fn load_deleted_by_organizations(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let query = format!(
            "SELECT {}.id as record_id, o.id as org_id, o.name as org_name
             FROM {}
             LEFT JOIN sys_users u ON {}.deleted_by_id = u.id
             LEFT JOIN user_organizations uo ON u.id = uo.user_id
             LEFT JOIN organizations o ON uo.organization_id = o.id
             WHERE {}.id = ANY($1)",
            table_name, table_name, table_name, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<AuditOrgRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    pub fn load_created_by_positions(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let query = format!(
            "SELECT {}.id as record_id, p.id as position_id, p.name as position_name
             FROM {}
             LEFT JOIN sys_users u ON {}.created_by_id = u.id
             LEFT JOIN user_organizations uo ON u.id = uo.user_id
             LEFT JOIN organization_positions p ON uo.organization_position_id = p.id
             WHERE {}.id = ANY($1)",
            table_name, table_name, table_name, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<AuditPositionRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    pub fn load_updated_by_positions(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let query = format!(
            "SELECT {}.id as record_id, p.id as position_id, p.name as position_name
             FROM {}
             LEFT JOIN sys_users u ON {}.updated_by_id = u.id
             LEFT JOIN user_organizations uo ON u.id = uo.user_id
             LEFT JOIN organization_positions p ON uo.organization_position_id = p.id
             WHERE {}.id = ANY($1)",
            table_name, table_name, table_name, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<AuditPositionRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    pub fn load_deleted_by_positions(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let query = format!(
            "SELECT {}.id as record_id, p.id as position_id, p.name as position_name
             FROM {}
             LEFT JOIN sys_users u ON {}.deleted_by_id = u.id
             LEFT JOIN user_organizations uo ON u.id = uo.user_id
             LEFT JOIN organization_positions p ON uo.organization_position_id = p.id
             WHERE {}.id = ANY($1)",
            table_name, table_name, table_name, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<AuditPositionRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    pub fn load_created_by_position_levels(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let query = format!(
            "SELECT {}.id as record_id,
                    u.id as user_id, u.name as user_name, u.email as user_email,
                    o.id as org_id, o.name as org_name,
                    p.id as position_id, p.name as position_name,
                    l.id as level_id, l.name as level_name, l.level as level_order
             FROM {}
             LEFT JOIN sys_users u ON {}.created_by_id = u.id
             LEFT JOIN user_organizations uo ON u.id = uo.user_id
             LEFT JOIN organizations o ON uo.organization_id = o.id
             LEFT JOIN organization_positions p ON uo.organization_position_id = p.id
             LEFT JOIN organization_position_levels l ON p.organization_position_level_id = l.id
             WHERE {}.id = ANY($1)",
            table_name, table_name, table_name, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<AuditFullRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    pub fn load_updated_by_position_levels(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let query = format!(
            "SELECT {}.id as record_id,
                    u.id as user_id, u.name as user_name, u.email as user_email,
                    o.id as org_id, o.name as org_name,
                    p.id as position_id, p.name as position_name,
                    l.id as level_id, l.name as level_name, l.level as level_order
             FROM {}
             LEFT JOIN sys_users u ON {}.updated_by_id = u.id
             LEFT JOIN user_organizations uo ON u.id = uo.user_id
             LEFT JOIN organizations o ON uo.organization_id = o.id
             LEFT JOIN organization_positions p ON uo.organization_position_id = p.id
             LEFT JOIN organization_position_levels l ON p.organization_position_level_id = l.id
             WHERE {}.id = ANY($1)",
            table_name, table_name, table_name, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<AuditFullRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    pub fn load_deleted_by_position_levels(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        let query = format!(
            "SELECT {}.id as record_id,
                    u.id as user_id, u.name as user_name, u.email as user_email,
                    o.id as org_id, o.name as org_name,
                    p.id as position_id, p.name as position_name,
                    l.id as level_id, l.name as level_name, l.level as level_order
             FROM {}
             LEFT JOIN sys_users u ON {}.deleted_by_id = u.id
             LEFT JOIN user_organizations uo ON u.id = uo.user_id
             LEFT JOIN organizations o ON uo.organization_id = o.id
             LEFT JOIN organization_positions p ON uo.organization_position_id = p.id
             LEFT JOIN organization_position_levels l ON p.organization_position_level_id = l.id
             WHERE {}.id = ANY($1)",
            table_name, table_name, table_name, table_name
        );

        let result = diesel::sql_query(&query)
            .bind::<diesel::sql_types::Array<diesel::sql_types::Text>, _>(ids)
            .load::<AuditFullRecord>(conn)?;

        Ok(serde_json::to_value(result)?)
    }

    // Alias methods for backward compatibility
    pub fn load_created_by_levels(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        Self::load_created_by_position_levels(table_name, ids, conn)
    }

    pub fn load_updated_by_levels(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        Self::load_updated_by_position_levels(table_name, ids, conn)
    }

    pub fn load_deleted_by_levels(table_name: &str, ids: &[String], conn: &mut PgConnection) -> Result<Value> {
        Self::load_deleted_by_position_levels(table_name, ids, conn)
    }
}

/// Record types for audit queries
#[derive(Debug, serde::Serialize, serde::Deserialize, QueryableByName)]
pub struct AuditUserRecord {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub record_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub email: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, QueryableByName)]
pub struct AuditOrgRecord {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub record_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub org_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub org_name: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, QueryableByName)]
pub struct AuditPositionRecord {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub record_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub position_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub position_name: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, QueryableByName)]
pub struct AuditFullRecord {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub record_id: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub user_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub user_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub user_email: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub org_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub org_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub position_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub position_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub level_id: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub level_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Integer>)]
    pub level_order: Option<i32>,
}