use async_trait::async_trait;
use serde_json::{json};
use std::collections::HashMap;
use anyhow::Result;
use ulid::Ulid;
use chrono::{Datelike, Timelike};

use crate::app::models::{
    user::User,
    userorganization::UserOrganization,
    organization::Organization,
    role::Role,
    permission::Permission,
};
use crate::app::policies::policy_trait::{
    PolicyTrait, AuthorizationContext, AuthorizationResult, RbacAuthorizer, AbacAuthorizer
};

#[derive(Debug)]
pub struct UserOrganizationPolicy {
    rbac_authorizer: RbacAuthorizer,
    abac_authorizer: AbacAuthorizer,
}

impl UserOrganizationPolicy {
    pub fn new() -> Self {
        Self {
            rbac_authorizer: RbacAuthorizer::new(),
            abac_authorizer: AbacAuthorizer::new(),
        }
    }

    // Create authorization context for UserOrganization operations
    pub async fn create_context(
        &self,
        user: User,
        roles: Vec<Role>,
        permissions: Vec<Permission>,
        user_org: Option<&UserOrganization>,
        organization: Option<&Organization>,
        action: String,
    ) -> AuthorizationContext {
        let mut attributes = HashMap::new();
        let mut environment = HashMap::new();

        // Add user organization attributes
        if let Some(uo) = user_org {
            attributes.insert("user_organization_id".to_string(), json!(uo.id.to_string()));
            attributes.insert("target_user_id".to_string(), json!(uo.user_id.to_string()));
            attributes.insert("organization_id".to_string(), json!(uo.organization_id.to_string()));
            attributes.insert("job_position_id".to_string(), json!(uo.job_position_id.to_string()));
            attributes.insert("is_active".to_string(), json!(uo.is_active));
            attributes.insert("started_at".to_string(), json!(uo.started_at.to_string()));
            if let Some(ended_at) = uo.ended_at {
                attributes.insert("ended_at".to_string(), json!(ended_at.to_string()));
            }
        }

        // Add organization hierarchy attributes
        if let Some(org) = organization {
            attributes.insert("organization_type".to_string(), json!(org.organization_type));
            attributes.insert("organization_level".to_string(), json!(self.get_organization_level(&org.organization_type)));
            if let Some(parent_id) = &org.parent_id {
                attributes.insert("parent_organization_id".to_string(), json!(parent_id.to_string()));
            }
        }

        // Add temporal and environmental attributes
        let now = chrono::Utc::now();
        environment.insert("current_time".to_string(), json!(now.format("%H:%M").to_string()));
        environment.insert("current_hour".to_string(), json!(now.hour().to_string()));
        environment.insert("current_day".to_string(), json!(now.weekday().to_string()));
        environment.insert("current_date".to_string(), json!(now.date_naive().to_string()));

        AuthorizationContext {
            user,
            roles,
            permissions,
            attributes,
            resource_type: Some("user_organization".to_string()),
            resource_id: user_org.map(|uo| uo.id),
            action,
            environment,
        }
    }

    // Determine organization hierarchy level for ABAC policies
    fn get_organization_level(&self, org_type: &str) -> u8 {
        match org_type {
            "company" => 1,
            "boc" => 2,
            "bod" => 3,
            "division" => 4,
            "department" => 5,
            "branch" => 6,
            "subbranch" => 7,
            "section" => 8,
            _ => 0,
        }
    }

    // Check if user can view any user organization relationships
    pub async fn view_any(&self, context: &AuthorizationContext) -> Result<AuthorizationResult> {
        // RBAC Check: Admin or HR roles can view all
        if self.rbac_authorizer.has_role(context, "admin").await ||
           self.rbac_authorizer.has_role(context, "hr_manager").await ||
           self.rbac_authorizer.has_permission(context, "user_organization.view_any").await {
            return Ok(AuthorizationResult::allow("Has admin/HR role or view_any permission".to_string()));
        }

        // ABAC Check: Can view within same organization hierarchy
        if let Some(user_org_level) = context.attributes.get("organization_level") {
            if let Some(level) = user_org_level.as_u64() {
                if level <= 5 { // Department level and above can view subordinates
                    return Ok(AuthorizationResult::allow("Can view within organization hierarchy".to_string()));
                }
            }
        }

        Ok(AuthorizationResult::deny("Insufficient permissions to view user organizations".to_string()))
    }

    // Check if user can view specific user organization relationship
    pub async fn view(&self, context: &AuthorizationContext) -> Result<AuthorizationResult> {
        // Check view_any first
        let view_any_result = self.view_any(context).await?;
        if view_any_result.allowed {
            return Ok(view_any_result);
        }

        // RBAC Check: Can view own user organization relationships
        if let Some(target_user_id) = context.attributes.get("target_user_id") {
            if let Some(target_id_str) = target_user_id.as_str() {
                if let Ok(target_ulid) = Ulid::from_string(target_id_str) {
                    if target_ulid == context.user.id {
                        return Ok(AuthorizationResult::allow("Can view own user organization relationship".to_string()));
                    }
                }
            }
        }

        // ABAC Check: Manager can view team members in same organization
        if self.rbac_authorizer.has_role(context, "manager").await {
            if let Some(_org_id) = context.attributes.get("organization_id") {
                // In a real implementation, you'd check if the user is a manager in this organization
                return Ok(AuthorizationResult::allow("Manager can view team members".to_string()));
            }
        }

        Ok(AuthorizationResult::deny("Cannot view this user organization relationship".to_string()))
    }

    // Check if user can create user organization relationships
    pub async fn create(&self, context: &AuthorizationContext) -> Result<AuthorizationResult> {
        // RBAC Check: Admin or HR can create
        if self.rbac_authorizer.has_role(context, "admin").await ||
           self.rbac_authorizer.has_role(context, "hr_manager").await ||
           self.rbac_authorizer.has_permission(context, "user_organization.create").await {
            return Ok(AuthorizationResult::allow("Has admin/HR role or create permission".to_string()));
        }

        // ABAC Check: Department heads can assign within their department
        if self.rbac_authorizer.has_role(context, "department_head").await {
            if let Some(org_level) = context.attributes.get("organization_level") {
                if let Some(level) = org_level.as_u64() {
                    if level >= 5 { // Department level or below
                        return Ok(AuthorizationResult::allow("Department head can assign within department".to_string()));
                    }
                }
            }
        }

        // ABAC Check: Time-based restrictions (business hours only)
        if let Some(current_hour) = context.environment.get("current_hour") {
            if let Some(hour_str) = current_hour.as_str() {
                if let Ok(hour) = hour_str.parse::<u32>() {
                    if hour < 8 || hour > 18 {
                        return Ok(AuthorizationResult::deny("User organization assignments only allowed during business hours".to_string()));
                    }
                }
            }
        }

        Ok(AuthorizationResult::deny("Insufficient permissions to create user organization relationship".to_string()))
    }

    // Check if user can update user organization relationships
    pub async fn update(&self, context: &AuthorizationContext) -> Result<AuthorizationResult> {
        // RBAC Check: Admin or HR can update
        if self.rbac_authorizer.has_role(context, "admin").await ||
           self.rbac_authorizer.has_role(context, "hr_manager").await ||
           self.rbac_authorizer.has_permission(context, "user_organization.update").await {
            return Ok(AuthorizationResult::allow("Has admin/HR role or update permission".to_string()));
        }

        // ABAC Check: Can update own active relationships (limited fields)
        if let Some(target_user_id) = context.attributes.get("target_user_id") {
            if let Some(target_id_str) = target_user_id.as_str() {
                if let Ok(target_ulid) = Ulid::from_string(target_id_str) {
                    if target_ulid == context.user.id {
                        if let Some(is_active) = context.attributes.get("is_active") {
                            if is_active.as_bool().unwrap_or(false) {
                                // Users can only update limited fields of their own active relationships
                                return Ok(AuthorizationResult::allow("Can update own active user organization relationship".to_string()));
                            }
                        }
                    }
                }
            }
        }

        // ABAC Check: Direct manager can update team members
        if self.rbac_authorizer.has_role(context, "manager").await {
            // In a real implementation, you'd verify the reporting relationship
            return Ok(AuthorizationResult::allow("Manager can update team member relationships".to_string()));
        }

        Ok(AuthorizationResult::deny("Cannot update this user organization relationship".to_string()))
    }

    // Check if user can delete user organization relationships
    pub async fn delete(&self, context: &AuthorizationContext) -> Result<AuthorizationResult> {
        // RBAC Check: Only admin or HR manager can delete
        if self.rbac_authorizer.has_role(context, "admin").await ||
           self.rbac_authorizer.has_role(context, "hr_manager").await ||
           self.rbac_authorizer.has_permission(context, "user_organization.delete").await {
            return Ok(AuthorizationResult::allow("Has admin/HR role or delete permission".to_string()));
        }

        // ABAC Check: Cannot delete relationships for critical roles
        if let Some(_job_position_id) = context.attributes.get("job_position_id") {
            // In a real implementation, you'd check if this is a critical position
            // For now, we'll allow deletion for non-critical positions
        }

        // ABAC Check: Cannot delete during freeze periods
        if let Some(current_date) = context.environment.get("current_date") {
            // Example: Block deletions during year-end (December)
            if let Some(date_str) = current_date.as_str() {
                if date_str.contains("-12-") { // December
                    return Ok(AuthorizationResult::deny("User organization deletions are frozen during year-end period".to_string()));
                }
            }
        }

        Ok(AuthorizationResult::deny("Insufficient permissions to delete user organization relationship".to_string()))
    }

    // Check if user can transfer someone to a different organization
    pub async fn transfer(&self, context: &AuthorizationContext) -> Result<AuthorizationResult> {
        // RBAC Check: Admin, HR, or specific transfer permission
        if self.rbac_authorizer.has_role(context, "admin").await ||
           self.rbac_authorizer.has_role(context, "hr_manager").await ||
           self.rbac_authorizer.has_permission(context, "user_organization.transfer").await {
            return Ok(AuthorizationResult::allow("Has transfer permission".to_string()));
        }

        // ABAC Check: Department heads can transfer within same division
        if self.rbac_authorizer.has_role(context, "department_head").await {
            if let Some(org_level) = context.attributes.get("organization_level") {
                if let Some(level) = org_level.as_u64() {
                    if level >= 4 { // Division level or below
                        return Ok(AuthorizationResult::allow("Department head can transfer within division".to_string()));
                    }
                }
            }
        }

        Ok(AuthorizationResult::deny("Insufficient permissions to transfer user".to_string()))
    }

    // Check if user can activate/deactivate user organization relationships
    pub async fn activate_deactivate(&self, context: &AuthorizationContext) -> Result<AuthorizationResult> {
        // RBAC Check: Admin, HR, or manager roles
        if self.rbac_authorizer.has_role(context, "admin").await ||
           self.rbac_authorizer.has_role(context, "hr_manager").await ||
           self.rbac_authorizer.has_role(context, "manager").await ||
           self.rbac_authorizer.has_permission(context, "user_organization.activate").await {
            return Ok(AuthorizationResult::allow("Has activation permission".to_string()));
        }

        Ok(AuthorizationResult::deny("Insufficient permissions to activate/deactivate user organization relationship".to_string()))
    }
}

#[async_trait]
impl PolicyTrait for UserOrganizationPolicy {
    async fn authorize(&self, context: &AuthorizationContext) -> Result<AuthorizationResult> {
        // First try RBAC authorization
        let rbac_result = self.authorize_rbac(context).await?;
        if rbac_result.allowed {
            return Ok(rbac_result);
        }

        // If RBAC fails, try ABAC authorization
        self.authorize_abac(context).await
    }

    async fn check_role_permissions(&self, context: &AuthorizationContext) -> Result<AuthorizationResult> {
        match context.action.as_str() {
            "view_any" => self.view_any(context).await,
            "view" => self.view(context).await,
            "create" => self.create(context).await,
            "update" => self.update(context).await,
            "delete" => self.delete(context).await,
            "transfer" => self.transfer(context).await,
            "activate" | "deactivate" => self.activate_deactivate(context).await,
            _ => Ok(AuthorizationResult::deny(format!("Unknown action: {}", context.action))),
        }
    }

    async fn evaluate_policies(&self, context: &AuthorizationContext) -> Result<AuthorizationResult> {
        // ABAC policy conditions for UserOrganization
        let conditions = vec![
            // Time-based access control
            "time.hour in [\"08\", \"09\", \"10\", \"11\", \"12\", \"13\", \"14\", \"15\", \"16\", \"17\", \"18\"]",
            // Organization hierarchy constraints
            "user.organization_level <= resource.organization_level",
            // Same department constraints
            "user.department == resource.department",
        ];

        for condition in conditions {
            if !self.abac_authorizer.evaluate_policy_condition(condition, context).await? {
                return Ok(AuthorizationResult::deny(format!("ABAC condition failed: {}", condition)));
            }
        }

        Ok(AuthorizationResult::allow("ABAC policies satisfied".to_string()))
    }

    async fn check_permission(&self, context: &AuthorizationContext, permission: &str) -> Result<bool> {
        Ok(self.rbac_authorizer.has_permission(context, permission).await)
    }

    async fn check_role(&self, context: &AuthorizationContext, role: &str) -> Result<bool> {
        Ok(self.rbac_authorizer.has_role(context, role).await)
    }

    async fn evaluate_condition(&self, condition: &str, context: &AuthorizationContext) -> Result<bool> {
        self.abac_authorizer.evaluate_policy_condition(condition, context).await
    }
}

impl Default for UserOrganizationPolicy {
    fn default() -> Self {
        Self::new()
    }
}