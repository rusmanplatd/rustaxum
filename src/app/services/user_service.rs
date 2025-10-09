use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::json;
use crate::app::models::DieselUlid;
use crate::database::{DbPool};
use crate::schema::{sys_users, user_organizations, organizations, organization_positions, organization_position_levels,
    sys_model_has_roles, sys_roles, sys_model_has_permissions, sys_permissions};
use crate::app::models::user::{User, CreateUser, UpdateUser};
use crate::app::models::user_organization::UserOrganization;
use crate::app::models::organization::Organization;
use crate::app::models::organization_position::OrganizationPosition;
use crate::app::models::organization_position_level::OrganizationPositionLevel;
use crate::app::models::role::Role;
use crate::app::models::permission::Permission;
use crate::app::traits::ServiceActivityLogger;
use crate::app::resources::user_organization_resource::{
    UserOrganizationResourceWithRelations, UserBasicInfo, OrganizationBasicInfo,
    OrganizationPositionBasicInfo, OrganizationPositionLevelBasicInfo, RoleBasicInfo, PermissionBasicInfo
};
use crate::app::resources::user_resource::{UserResourceWithRolesAndPermissions, UserRoleBasicInfo, UserPermissionBasicInfo};
use std::collections::{HashMap, HashSet};

pub struct UserService;

impl ServiceActivityLogger for UserService {}

impl UserService {
    pub async fn create_user(pool: &DbPool, data: CreateUser, created_by: Option<DieselUlid>) -> Result<User> {
        let mut conn = pool.get()?;

        let new_user = User::to_new_user(data.name, data.email, data.password, created_by);

        let created_user = diesel::insert_into(sys_users::table)
            .values(&new_user)
            .returning(User::as_select())
            .get_result::<User>(&mut conn)?;

        // Log the user creation activity
        let service = UserService;
        let causer_id = created_by.map(|id| id.to_string());
        let properties = json!({
            "user_name": created_user.name.clone(),
            "user_email": created_user.email.clone(),
            "created_by": causer_id
        });

        if let Err(e) = service.log_created(
            &created_user,
            causer_id.as_deref(),
            Some(properties)
        ).await {
            eprintln!("Failed to log user creation activity: {}", e);
        }

        Ok(created_user)
    }

    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<User>> {
        let mut conn = pool.get()?;

        let result = sys_users::table
            .filter(sys_users::id.eq(id))
            .filter(sys_users::deleted_at.is_null())
            .select(User::as_select())
            .first::<User>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_email(pool: &DbPool, email: &str) -> Result<Option<User>> {
        let mut conn = pool.get()?;

        let result = sys_users::table
            .filter(sys_users::email.eq(email))
            .filter(sys_users::deleted_at.is_null())
            .select(User::as_select())
            .first::<User>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_reset_token(pool: &DbPool, token: &str) -> Result<Option<User>> {
        let mut conn = pool.get()?;

        let result = sys_users::table
            .filter(sys_users::password_reset_token.eq(token))
            .filter(sys_users::password_reset_expires_at.gt(Utc::now()))
            .filter(sys_users::deleted_at.is_null())
            .select(User::as_select())
            .first::<User>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub async fn update_user(pool: &DbPool, id: String, data: UpdateUser, updated_by: Option<DieselUlid>) -> Result<User> {
        let mut conn = pool.get()?;

        let original_user = sys_users::table
            .filter(sys_users::id.eq(id.to_string()))
            .filter(sys_users::deleted_at.is_null())
            .select(User::as_select())
            .first::<User>(&mut conn)
            .optional()?;

        let result = diesel::update(sys_users::table
            .filter(sys_users::id.eq(id.to_string()))
            .filter(sys_users::deleted_at.is_null()))
            .set((
                data.name.as_ref().map(|n| sys_users::name.eq(n)),
                data.email.as_ref().map(|e| sys_users::email.eq(e)),
                sys_users::updated_at.eq(Utc::now()),
                sys_users::updated_by_id.eq(updated_by),
            ))
            .returning(User::as_select())
            .get_result::<User>(&mut conn)?;

        // Log the user update activity
        let service = UserService;
        let causer_id = updated_by.map(|id| id.to_string());

        let mut changes = json!({});
        if let Some(original) = original_user {
            if let Some(ref new_name) = data.name {
                if &original.name != new_name {
                    changes["name"] = json!({
                        "from": original.name,
                        "to": new_name
                    });
                }
            }
            if let Some(ref new_email) = data.email {
                if &original.email != new_email {
                    changes["email"] = json!({
                        "from": original.email,
                        "to": new_email
                    });
                }
            }
        }

        if let Err(e) = service.log_updated(
            &result,
            changes,
            causer_id.as_deref()
        ).await {
            eprintln!("Failed to log user update activity: {}", e);
        }

        Ok(result)
    }

    pub fn update_password(pool: &DbPool, id: DieselUlid, new_password: String, updated_by: Option<DieselUlid>) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table
            .filter(sys_users::id.eq(id.to_string()))
            .filter(sys_users::deleted_at.is_null()))
            .set((
                sys_users::password.eq(new_password),
                sys_users::updated_at.eq(Utc::now()),
                sys_users::updated_by_id.eq(updated_by),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn update_last_login(pool: &DbPool, id: DieselUlid) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table
            .filter(sys_users::id.eq(id.to_string()))
            .filter(sys_users::deleted_at.is_null()))
            .set((
                sys_users::last_login_at.eq(Some(Utc::now())),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn update_failed_attempts(pool: &DbPool, id: DieselUlid, attempts: i32, locked_until: Option<DateTime<Utc>>) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table
            .filter(sys_users::id.eq(id))
            .filter(sys_users::deleted_at.is_null()))
            .set((
                sys_users::failed_login_attempts.eq(attempts),
                sys_users::locked_until.eq(locked_until),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn reset_failed_attempts(pool: &DbPool, id: DieselUlid) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table
            .filter(sys_users::id.eq(id.to_string()))
            .filter(sys_users::deleted_at.is_null()))
            .set((
                sys_users::failed_login_attempts.eq(0),
                sys_users::locked_until.eq::<Option<DateTime<Utc>>>(None),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn update_password_reset_token(pool: &DbPool, id: DieselUlid, token: Option<String>, expires_at: Option<DateTime<Utc>>) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table
            .filter(sys_users::id.eq(id.to_string()))
            .filter(sys_users::deleted_at.is_null()))
            .set((
                sys_users::password_reset_token.eq(token),
                sys_users::password_reset_expires_at.eq(expires_at),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn update_refresh_token(pool: &DbPool, id: DieselUlid, token: Option<String>, expires_at: Option<DateTime<Utc>>) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table
            .filter(sys_users::id.eq(id.to_string()))
            .filter(sys_users::deleted_at.is_null()))
            .set((
                sys_users::refresh_token.eq(token),
                sys_users::refresh_token_expires_at.eq(expires_at),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn find_by_refresh_token(pool: &DbPool, token: &str) -> Result<Option<User>> {
        let mut conn = pool.get()?;

        let result = sys_users::table
            .filter(sys_users::refresh_token.eq(token))
            .filter(sys_users::refresh_token_expires_at.gt(Utc::now()))
            .filter(sys_users::deleted_at.is_null())
            .select(User::as_select())
            .first::<User>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub async fn soft_delete(pool: &DbPool, id: String, deleted_by: Option<DieselUlid>) -> Result<()> {
        let mut conn = pool.get()?;

        // Get the user before deletion for logging
        let user = sys_users::table
            .filter(sys_users::id.eq(id.clone()))
            .filter(sys_users::deleted_at.is_null())
            .select(User::as_select())
            .first::<User>(&mut conn)
            .optional()?;

        diesel::update(sys_users::table
            .filter(sys_users::id.eq(id))
            .filter(sys_users::deleted_at.is_null()))
            .set((
                sys_users::deleted_at.eq(Some(Utc::now())),
                sys_users::deleted_by_id.eq(deleted_by),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        // Log the user deletion activity
        if let Some(user) = user {
            let service = UserService;
            let causer_id = deleted_by.map(|id| id.to_string());

            if let Err(e) = service.log_deleted(
                &user,
                causer_id.as_deref()
            ).await {
                eprintln!("Failed to log user deletion activity: {}", e);
            }
        }

        Ok(())
    }

    pub fn restore(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table.filter(sys_users::id.eq(id)))
            .set((
                sys_users::deleted_at.eq::<Option<DateTime<Utc>>>(None),
                sys_users::deleted_by_id.eq::<Option<DieselUlid>>(None),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn hard_delete(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(sys_users::table.filter(sys_users::id.eq(id)))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn find_deleted(pool: &DbPool) -> Result<Vec<User>> {
        let mut conn = pool.get()?;

        let result = sys_users::table
            .filter(sys_users::deleted_at.is_not_null())
            .select(User::as_select())
            .load::<User>(&mut conn)?;

        Ok(result)
    }

    /// Find user by ID with all their organization relationships and related data
    pub fn find_by_id_with_organizations(pool: &DbPool, id: String) -> Result<Option<(User, Vec<UserOrganizationResourceWithRelations>)>> {
        let mut conn = pool.get()?;

        // First get the user
        let user = sys_users::table
            .filter(sys_users::id.eq(&id))
            .filter(sys_users::deleted_at.is_null())
            .select(User::as_select())
            .first::<User>(&mut conn)
            .optional()?;

        let user = match user {
            Some(user) => user,
            None => return Ok(None),
        };

        // Get user organizations with all related data using left joins
        let user_org_data = user_organizations::table
            .filter(user_organizations::user_id.eq(&id))
            .left_join(organizations::table.on(organizations::id.eq(user_organizations::organization_id)))
            .left_join(organization_positions::table.on(organization_positions::id.eq(user_organizations::organization_position_id)))
            .left_join(organization_position_levels::table.on(organization_position_levels::id.eq(organization_positions::organization_position_level_id)))
            .select((
                UserOrganization::as_select(),
                organizations::all_columns.nullable(),
                organization_positions::all_columns.nullable(),
                organization_position_levels::all_columns.nullable(),
            ))
            .load::<(UserOrganization, Option<Organization>, Option<OrganizationPosition>, Option<OrganizationPositionLevel>)>(&mut conn)?;

        // Get all user organization IDs for loading roles and permissions
        let user_org_ids: Vec<String> = user_org_data
            .iter()
            .map(|(user_org, _, _, _)| user_org.id.to_string())
            .collect();

        // Load roles for all user organizations
        let user_org_roles = if !user_org_ids.is_empty() {
            sys_model_has_roles::table
                .filter(sys_model_has_roles::model_type.eq("UserOrganization"))
                .filter(sys_model_has_roles::model_id.eq_any(&user_org_ids))
                .inner_join(sys_roles::table.on(sys_roles::id.eq(sys_model_has_roles::role_id)))
                .select((sys_model_has_roles::model_id, Role::as_select()))
                .load::<(String, Role)>(&mut conn)?
        } else {
            vec![]
        };

        // Create a map of user_org_id -> roles
        let mut roles_map: HashMap<String, Vec<Role>> = HashMap::new();
        for (user_org_id, role) in user_org_roles {
            roles_map.entry(user_org_id).or_insert_with(Vec::new).push(role);
        }

        // Get all role IDs for loading permissions through roles
        let role_ids: Vec<String> = roles_map
            .values()
            .flat_map(|roles| roles.iter().map(|r| r.id.to_string()))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // Load direct permissions for user organizations
        let direct_permissions = if !user_org_ids.is_empty() {
            sys_model_has_permissions::table
                .filter(sys_model_has_permissions::model_type.eq("UserOrganization"))
                .filter(sys_model_has_permissions::model_id.eq_any(&user_org_ids))
                .inner_join(sys_permissions::table.on(sys_permissions::id.eq(sys_model_has_permissions::permission_id)))
                .select((sys_model_has_permissions::model_id, Permission::as_select()))
                .load::<(String, Permission)>(&mut conn)?
        } else {
            vec![]
        };

        // Load permissions through roles
        let role_permissions = if !role_ids.is_empty() {
            sys_model_has_permissions::table
                .filter(sys_model_has_permissions::model_type.eq("Role"))
                .filter(sys_model_has_permissions::model_id.eq_any(&role_ids))
                .inner_join(sys_permissions::table.on(sys_permissions::id.eq(sys_model_has_permissions::permission_id)))
                .select((sys_model_has_permissions::model_id, Permission::as_select()))
                .load::<(String, Permission)>(&mut conn)?
        } else {
            vec![]
        };

        // Create a map of role_id -> permissions
        let mut role_permissions_map: HashMap<String, Vec<Permission>> = HashMap::new();
        for (role_id, permission) in role_permissions {
            role_permissions_map.entry(role_id).or_insert_with(Vec::new).push(permission);
        }

        // Create a map of user_org_id -> direct permissions
        let mut direct_permissions_map: HashMap<String, Vec<Permission>> = HashMap::new();
        for (user_org_id, permission) in direct_permissions {
            direct_permissions_map.entry(user_org_id).or_insert_with(Vec::new).push(permission);
        }

        // Transform the data into UserOrganizationResourceWithRelations
        let user_organizations: Vec<UserOrganizationResourceWithRelations> = user_org_data
            .into_iter()
            .map(|(user_org, org, pos, level)| {
                let user_org_id = user_org.id.to_string();

                let user_basic = Some(UserBasicInfo {
                    id: user.id.to_string(),
                    name: user.name.clone(),
                    email: user.email.clone(),
                });

                let organization_basic = org.map(|o| OrganizationBasicInfo {
                    id: o.id.to_string(),
                    name: o.name,
                    code: o.code,
                });

                let organization_position_basic = pos.map(|p| OrganizationPositionBasicInfo {
                    id: p.id.to_string(),
                    name: p.name,
                    code: if p.code.is_empty() { None } else { Some(p.code) },
                    organization_position_level: level.map(|l| OrganizationPositionLevelBasicInfo {
                        id: l.id.to_string(),
                        name: l.name,
                        level: l.level,
                    }),
                });

                // Get roles for this user organization
                let roles = roles_map.get(&user_org_id).cloned().unwrap_or_default();
                let role_basic_info: Vec<RoleBasicInfo> = roles.iter().map(|r| RoleBasicInfo {
                    id: r.id.to_string(),
                    name: r.name.clone(),
                    description: r.description.clone(),
                    guard_name: r.guard_name.clone(),
                }).collect();

                // Get all permissions (direct + from roles)
                let mut all_permissions: Vec<PermissionBasicInfo> = Vec::new();

                // Add direct permissions
                if let Some(direct_perms) = direct_permissions_map.get(&user_org_id) {
                    for perm in direct_perms {
                        all_permissions.push(PermissionBasicInfo {
                            id: perm.id.to_string(),
                            name: format!("{}.{}", perm.resource.clone().unwrap_or_default(), perm.action.clone()),
                            resource: perm.resource.clone(),
                            action: perm.action.clone(),
                            guard_name: perm.guard_name.clone(),
                            source: Some("direct".to_string()),
                        });
                    }
                }

                // Add permissions from roles
                for role in &roles {
                    if let Some(role_perms) = role_permissions_map.get(&role.id.to_string()) {
                        for perm in role_perms {
                            // Check if we already have this permission (avoid duplicates)
                            if !all_permissions.iter().any(|p| p.id == perm.id.to_string()) {
                                all_permissions.push(PermissionBasicInfo {
                                    id: perm.id.to_string(),
                                    name: format!("{}.{}", perm.resource.clone().unwrap_or_default(), perm.action.clone()),
                                    resource: perm.resource.clone(),
                                    action: perm.action.clone(),
                                    guard_name: perm.guard_name.clone(),
                                    source: Some(format!("role:{}", role.name)),
                                });
                            }
                        }
                    }
                }

                UserOrganizationResourceWithRelations {
                    id: user_org_id,
                    user_id: user_org.user_id.to_string(),
                    organization_id: user_org.organization_id.to_string(),
                    organization_position_id: user_org.organization_position_id.to_string(),
                    is_active: user_org.is_active,
                    started_at: user_org.started_at,
                    ended_at: user_org.ended_at,
                    created_at: user_org.created_at,
                    updated_at: user_org.updated_at,
                    user: user_basic,
                    organization: organization_basic,
                    organization_position: organization_position_basic,
                    roles: role_basic_info,
                    permissions: all_permissions,
                }
            })
            .collect();

        Ok(Some((user, user_organizations)))
    }

    /// Find user by ID with all their organization relationships and user-level roles/permissions
    pub fn find_by_id_with_organizations_and_user_roles(pool: &DbPool, id: String) -> Result<Option<(UserResourceWithRolesAndPermissions, Vec<UserOrganizationResourceWithRelations>)>> {
        let mut conn = pool.get()?;

        // First get the user
        let user = sys_users::table
            .filter(sys_users::id.eq(&id))
            .filter(sys_users::deleted_at.is_null())
            .select(User::as_select())
            .first::<User>(&mut conn)
            .optional()?;

        let user = match user {
            Some(user) => user,
            None => return Ok(None),
        };

        // Get user organizations with all related data using left joins
        let user_org_data = user_organizations::table
            .filter(user_organizations::user_id.eq(&id))
            .left_join(organizations::table.on(organizations::id.eq(user_organizations::organization_id)))
            .left_join(organization_positions::table.on(organization_positions::id.eq(user_organizations::organization_position_id)))
            .left_join(organization_position_levels::table.on(organization_position_levels::id.eq(organization_positions::organization_position_level_id)))
            .select((
                UserOrganization::as_select(),
                organizations::all_columns.nullable(),
                organization_positions::all_columns.nullable(),
                organization_position_levels::all_columns.nullable(),
            ))
            .load::<(UserOrganization, Option<Organization>, Option<OrganizationPosition>, Option<OrganizationPositionLevel>)>(&mut conn)?;

        // Get all user organization IDs for loading roles and permissions
        let user_org_ids: Vec<String> = user_org_data
            .iter()
            .map(|(user_org, _, _, _)| user_org.id.to_string())
            .collect();

        // Load roles for all user organizations
        let user_org_roles = if !user_org_ids.is_empty() {
            sys_model_has_roles::table
                .filter(sys_model_has_roles::model_type.eq("UserOrganization"))
                .filter(sys_model_has_roles::model_id.eq_any(&user_org_ids))
                .inner_join(sys_roles::table.on(sys_roles::id.eq(sys_model_has_roles::role_id)))
                .select((sys_model_has_roles::model_id, Role::as_select()))
                .load::<(String, Role)>(&mut conn)?
        } else {
            vec![]
        };

        // Load roles directly assigned to the user
        let user_direct_roles = sys_model_has_roles::table
            .filter(sys_model_has_roles::model_type.eq("User"))
            .filter(sys_model_has_roles::model_id.eq(&id))
            .inner_join(sys_roles::table.on(sys_roles::id.eq(sys_model_has_roles::role_id)))
            .select(Role::as_select())
            .load::<Role>(&mut conn)?;

        // Create a map of user_org_id -> roles
        let mut roles_map: HashMap<String, Vec<Role>> = HashMap::new();
        for (user_org_id, role) in user_org_roles {
            roles_map.entry(user_org_id).or_insert_with(Vec::new).push(role);
        }

        // Get all role IDs (both user-level and organization-level) for loading permissions through roles
        let mut all_role_ids: HashSet<String> = roles_map
            .values()
            .flat_map(|roles| roles.iter().map(|r| r.id.to_string()))
            .collect();

        for role in &user_direct_roles {
            all_role_ids.insert(role.id.to_string());
        }
        let role_ids: Vec<String> = all_role_ids.into_iter().collect();

        // Load direct permissions for user organizations
        let direct_permissions = if !user_org_ids.is_empty() {
            sys_model_has_permissions::table
                .filter(sys_model_has_permissions::model_type.eq("UserOrganization"))
                .filter(sys_model_has_permissions::model_id.eq_any(&user_org_ids))
                .inner_join(sys_permissions::table.on(sys_permissions::id.eq(sys_model_has_permissions::permission_id)))
                .select((sys_model_has_permissions::model_id, Permission::as_select()))
                .load::<(String, Permission)>(&mut conn)?
        } else {
            vec![]
        };

        // Load direct permissions for the user
        let user_direct_permissions = sys_model_has_permissions::table
            .filter(sys_model_has_permissions::model_type.eq("User"))
            .filter(sys_model_has_permissions::model_id.eq(&id))
            .inner_join(sys_permissions::table.on(sys_permissions::id.eq(sys_model_has_permissions::permission_id)))
            .select(Permission::as_select())
            .load::<Permission>(&mut conn)?;

        // Load permissions through roles
        let role_permissions = if !role_ids.is_empty() {
            sys_model_has_permissions::table
                .filter(sys_model_has_permissions::model_type.eq("Role"))
                .filter(sys_model_has_permissions::model_id.eq_any(&role_ids))
                .inner_join(sys_permissions::table.on(sys_permissions::id.eq(sys_model_has_permissions::permission_id)))
                .select((sys_model_has_permissions::model_id, Permission::as_select()))
                .load::<(String, Permission)>(&mut conn)?
        } else {
            vec![]
        };

        // Create a map of role_id -> permissions
        let mut role_permissions_map: HashMap<String, Vec<Permission>> = HashMap::new();
        for (role_id, permission) in role_permissions {
            role_permissions_map.entry(role_id).or_insert_with(Vec::new).push(permission);
        }

        // Create a map of user_org_id -> direct permissions
        let mut direct_permissions_map: HashMap<String, Vec<Permission>> = HashMap::new();
        for (user_org_id, permission) in direct_permissions {
            direct_permissions_map.entry(user_org_id).or_insert_with(Vec::new).push(permission);
        }

        // Transform the user organization data into UserOrganizationResourceWithRelations
        let user_organizations: Vec<UserOrganizationResourceWithRelations> = user_org_data
            .into_iter()
            .map(|(user_org, org, pos, level)| {
                let user_org_id = user_org.id.to_string();

                let user_basic = Some(UserBasicInfo {
                    id: user.id.to_string(),
                    name: user.name.clone(),
                    email: user.email.clone(),
                });

                let organization_basic = org.map(|o| OrganizationBasicInfo {
                    id: o.id.to_string(),
                    name: o.name,
                    code: o.code,
                });

                let organization_position_basic = pos.map(|p| OrganizationPositionBasicInfo {
                    id: p.id.to_string(),
                    name: p.name,
                    code: if p.code.is_empty() { None } else { Some(p.code) },
                    organization_position_level: level.map(|l| OrganizationPositionLevelBasicInfo {
                        id: l.id.to_string(),
                        name: l.name,
                        level: l.level,
                    }),
                });

                // Get roles for this user organization
                let roles = roles_map.get(&user_org_id).cloned().unwrap_or_default();
                let role_basic_info: Vec<RoleBasicInfo> = roles.iter().map(|r| RoleBasicInfo {
                    id: r.id.to_string(),
                    name: r.name.clone(),
                    description: r.description.clone(),
                    guard_name: r.guard_name.clone(),
                }).collect();

                // Get all permissions (direct + from roles)
                let mut all_permissions: Vec<PermissionBasicInfo> = Vec::new();

                // Add direct permissions
                if let Some(direct_perms) = direct_permissions_map.get(&user_org_id) {
                    for perm in direct_perms {
                        all_permissions.push(PermissionBasicInfo {
                            id: perm.id.to_string(),
                            name: format!("{}.{}", perm.resource.clone().unwrap_or_default(), perm.action.clone()),
                            resource: perm.resource.clone(),
                            action: perm.action.clone(),
                            guard_name: perm.guard_name.clone(),
                            source: Some("direct".to_string()),
                        });
                    }
                }

                // Add permissions from roles
                for role in &roles {
                    if let Some(role_perms) = role_permissions_map.get(&role.id.to_string()) {
                        for perm in role_perms {
                            // Check if we already have this permission (avoid duplicates)
                            if !all_permissions.iter().any(|p| p.id == perm.id.to_string()) {
                                all_permissions.push(PermissionBasicInfo {
                                    id: perm.id.to_string(),
                                    name: format!("{}.{}", perm.resource.clone().unwrap_or_default(), perm.action.clone()),
                                    resource: perm.resource.clone(),
                                    action: perm.action.clone(),
                                    guard_name: perm.guard_name.clone(),
                                    source: Some(format!("role:{}", role.name)),
                                });
                            }
                        }
                    }
                }

                UserOrganizationResourceWithRelations {
                    id: user_org_id,
                    user_id: user_org.user_id.to_string(),
                    organization_id: user_org.organization_id.to_string(),
                    organization_position_id: user_org.organization_position_id.to_string(),
                    is_active: user_org.is_active,
                    started_at: user_org.started_at,
                    ended_at: user_org.ended_at,
                    created_at: user_org.created_at,
                    updated_at: user_org.updated_at,
                    user: user_basic,
                    organization: organization_basic,
                    organization_position: organization_position_basic,
                    roles: role_basic_info,
                    permissions: all_permissions,
                }
            })
            .collect();

        // Build user-level roles and permissions
        let user_roles: Vec<UserRoleBasicInfo> = user_direct_roles.iter().map(|r| UserRoleBasicInfo {
            id: r.id.to_string(),
            name: r.name.clone(),
            slug: r.name.to_lowercase().replace(" ", "_"), // Generate slug from name
            description: r.description.clone(),
            source: "direct".to_string(),
        }).collect();

        // Build user-level permissions (direct + from user-level roles)
        let mut user_permissions: Vec<UserPermissionBasicInfo> = Vec::new();

        // Add direct permissions to user
        for perm in &user_direct_permissions {
            user_permissions.push(UserPermissionBasicInfo {
                id: perm.id.to_string(),
                name: format!("{}.{}", perm.resource.clone().unwrap_or_default(), perm.action.clone()),
                slug: format!("{}.{}", perm.resource.clone().unwrap_or_default(), perm.action.clone()).to_lowercase().replace(" ", "_"), // Generate slug from name
                description: Some(format!("{}: {}", perm.resource.as_deref().unwrap_or("system"), perm.action)), // Generate description
                source: "direct".to_string(),
                source_role_id: None,
            });
        }

        // Add permissions from user-level roles
        for role in &user_direct_roles {
            if let Some(role_perms) = role_permissions_map.get(&role.id.to_string()) {
                for perm in role_perms {
                    // Check if we already have this permission (avoid duplicates)
                    if !user_permissions.iter().any(|p| p.id == perm.id.to_string()) {
                        user_permissions.push(UserPermissionBasicInfo {
                            id: perm.id.to_string(),
                            name: format!("{}.{}", perm.resource.clone().unwrap_or_default(), perm.action.clone()),
                            slug: format!("{}.{}", perm.resource.clone().unwrap_or_default(), perm.action.clone()).to_lowercase().replace(" ", "_"), // Generate slug from name
                            description: Some(format!("{}: {}", perm.resource.as_deref().unwrap_or("system"), perm.action)), // Generate description
                            source: "role".to_string(),
                            source_role_id: Some(role.id.to_string()),
                        });
                    }
                }
            }
        }

        // Build the enhanced user resource
        let enhanced_user = UserResourceWithRolesAndPermissions {
            id: user.id.to_string(),
            name: user.name,
            email: user.email,
            email_verified_at: user.email_verified_at,
            username: user.username,
            avatar: user.avatar,
            birthdate: user.birthdate,
            last_login_at: user.last_login_at,
            last_seen_at: user.last_seen_at,
            locale: user.locale,
            phone_number: user.phone_number,
            phone_verified_at: user.phone_verified_at,
            zoneinfo: user.zoneinfo,
            created_at: user.created_at,
            updated_at: user.updated_at,
            roles: user_roles,
            permissions: user_permissions,
        };

        Ok(Some((enhanced_user, user_organizations)))
    }
}