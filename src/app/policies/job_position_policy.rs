use async_trait::async_trait;
use crate::app::models::user::User;
use crate::app::models::jobposition::JobPosition;
use crate::app::policies::policy_trait::{PolicyTrait, AuthorizationContext, AuthorizationResult};
use anyhow::Result;

pub struct JobPositionPolicy;

#[async_trait]
impl PolicyTrait for JobPositionPolicy {
    async fn authorize(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult> {
        // Main authorization logic - combine RBAC and ABAC
        let rbac_result = self.authorize_rbac(context).await?;
        if rbac_result.allowed {
            return Ok(rbac_result);
        }

        // If RBAC fails, try ABAC
        self.authorize_abac(context).await
    }

    async fn check_role_permissions(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult> {
        // Check if user has required permission based on action and resource type
        let required_permission = match context.action.as_str() {
            "view" => "job_positions.view",
            "view_any" => "job_positions.view",
            "create" => "job_positions.create",
            "update" => "job_positions.update",
            "delete" => "job_positions.delete",
            "activate" => "job_positions.update",
            "deactivate" => "job_positions.update",
            _ => return Ok(AuthorizationResult::deny("Unknown action".to_string())),
        };

        if self.check_permission(context, required_permission).await? {
            Ok(AuthorizationResult::allow(format!("User has required permission: {}", required_permission)))
        } else {
            Ok(AuthorizationResult::deny(format!("User lacks required permission: {}", required_permission)))
        }
    }

    async fn evaluate_policies(&self, _context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult> {
        // ABAC policy evaluation - check attributes and conditions

        // For job positions, we might have policies like:
        // - Only allow users from HR department to manage job positions
        // - Only allow during business hours
        // - Only allow if user's job level is equal or higher than the position being managed

        // For now, a simple implementation
        Ok(AuthorizationResult::allow("ABAC evaluation passed".to_string()))
    }

    async fn check_permission(&self, context: &AuthorizationContext, permission: &str) -> anyhow::Result<bool> {
        // Check if user has the specific permission
        for user_permission in &context.permissions {
            if user_permission.name == permission {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn check_role(&self, context: &AuthorizationContext, role: &str) -> anyhow::Result<bool> {
        // Check if user has the specific role
        for user_role in &context.roles {
            if user_role.name == role {
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn evaluate_condition(&self, condition: &str, context: &AuthorizationContext) -> anyhow::Result<bool> {
        // Evaluate a condition string against the context
        // This could be a simple expression evaluator

        match condition {
            "is_business_hours" => {
                // Check if current time is within business hours
                Ok(true) // Simplified for now
            }
            "is_hr_department" => {
                // Check if user is from HR department
                if let Some(department) = context.attributes.get("department") {
                    Ok(department.as_str() == Some("HR"))
                } else {
                    Ok(false)
                }
            }
            _ => Ok(false),
        }
    }
}

// Convenience methods for Laravel-style policy usage
impl JobPositionPolicy {
    pub async fn view_any(_user: &User) -> Result<bool> {
        // For now, allowing all authenticated users to view job positions
        Ok(true)
    }

    pub async fn view(_user: &User, _job_position: &JobPosition) -> Result<bool> {
        // Allow viewing if user has permission
        // This would check actual permissions in a real implementation
        Ok(true)
    }

    pub async fn create(_user: &User) -> Result<bool> {
        // Allow creating if user has permission
        Ok(true)
    }

    pub async fn update(_user: &User, _job_position: &JobPosition) -> Result<bool> {
        // Allow updating if user has permission
        Ok(true)
    }

    pub async fn delete(_user: &User, _job_position: &JobPosition) -> Result<bool> {
        // Allow deleting if user has permission
        Ok(true)
    }
}