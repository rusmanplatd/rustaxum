use async_trait::async_trait;
use crate::app::policies::policy_trait::{
    PolicyTrait, AuthorizationContext, AuthorizationResult, RbacAuthorizer, AbacAuthorizer
};
use crate::app::models::policy::{Policy, PolicyEffect};

#[derive(Debug)]
pub struct BasePolicy {
    rbac_authorizer: RbacAuthorizer,
    abac_authorizer: AbacAuthorizer,
    policies: Vec<Policy>,
}

impl BasePolicy {
    pub fn new() -> Self {
        Self {
            rbac_authorizer: RbacAuthorizer::new(),
            abac_authorizer: AbacAuthorizer::new(),
            policies: vec![],
        }
    }

    pub fn with_policies(mut self, policies: Vec<Policy>) -> Self {
        self.policies = policies;
        self
    }

    async fn combine_results(
        &self,
        rbac_result: AuthorizationResult,
        abac_result: AuthorizationResult,
    ) -> AuthorizationResult {
        if !rbac_result.allowed {
            return rbac_result;
        }

        if !abac_result.allowed {
            return abac_result;
        }

        AuthorizationResult::allow(format!(
            "RBAC: {} | ABAC: {}",
            rbac_result.reason,
            abac_result.reason
        )).with_policies([rbac_result.applied_policies, abac_result.applied_policies].concat())
    }

    async fn evaluate_policy_rules(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult> {
        let mut applied_policies = Vec::new();
        let mut final_effect = PolicyEffect::Deny;
        let mut denial_reason = String::new();

        for policy in &self.policies {
            if !policy.is_active {
                continue;
            }

            let matches_target = self.matches_target(&policy.target, context).await?;
            if !matches_target {
                continue;
            }

            let condition_result = if let Some(condition) = &policy.condition {
                self.evaluate_condition(condition, context).await?
            } else {
                true
            };

            if condition_result {
                applied_policies.push(policy.name.clone());

                match policy.effect {
                    PolicyEffect::Permit => {
                        final_effect = PolicyEffect::Permit;
                    }
                    PolicyEffect::Deny => {
                        denial_reason = format!("Denied by policy: {}", policy.name);
                        return Ok(AuthorizationResult::deny(denial_reason)
                            .with_policies(applied_policies));
                    }
                }
            }
        }

        match final_effect {
            PolicyEffect::Permit => Ok(AuthorizationResult::allow(
                "Authorized by policy evaluation".to_string()
            ).with_policies(applied_policies)),
            PolicyEffect::Deny => Ok(AuthorizationResult::deny(
                "No permit policy matched".to_string()
            ).with_policies(applied_policies)),
        }
    }

    async fn matches_target(&self, target: &str, context: &AuthorizationContext) -> anyhow::Result<bool> {
        let parts: Vec<&str> = target.split(':').collect();
        if parts.len() != 3 {
            return Ok(false);
        }

        let (subject_pattern, resource_pattern, action_pattern) = (parts[0], parts[1], parts[2]);

        let subject_matches = self.matches_subject_pattern(subject_pattern, context).await?;
        let resource_matches = self.matches_resource_pattern(resource_pattern, context).await?;
        let action_matches = action_pattern == "*" || action_pattern == context.action;

        Ok(subject_matches && resource_matches && action_matches)
    }

    async fn matches_subject_pattern(&self, pattern: &str, context: &AuthorizationContext) -> anyhow::Result<bool> {
        if pattern == "*" {
            return Ok(true);
        }

        if pattern.starts_with("user:") {
            let user_id = pattern.strip_prefix("user:").unwrap();
            return Ok(context.user.id.to_string() == user_id);
        }

        if pattern.starts_with("role:") {
            let role_name = pattern.strip_prefix("role:").unwrap();
            return Ok(context.roles.iter().any(|r| r.name == role_name));
        }

        if pattern.starts_with("group:") {
            let group_name = pattern.strip_prefix("group:").unwrap();
            if let Some(user_groups) = context.attributes.get("groups") {
                if let Some(groups_array) = user_groups.as_array() {
                    return Ok(groups_array.iter().any(|g| g.as_str() == Some(group_name)));
                }
            }
        }

        Ok(false)
    }

    async fn matches_resource_pattern(&self, pattern: &str, context: &AuthorizationContext) -> anyhow::Result<bool> {
        if pattern == "*" {
            return Ok(true);
        }

        if let Some(resource_type) = &context.resource_type {
            if pattern == resource_type {
                return Ok(true);
            }

            if pattern.contains(':') {
                let parts: Vec<&str> = pattern.split(':').collect();
                if parts.len() == 2 {
                    let (res_type, res_id) = (parts[0], parts[1]);
                    if res_type == resource_type {
                        if let Some(resource_id) = context.resource_id {
                            return Ok(res_id == "*" || res_id == resource_id.to_string());
                        }
                    }
                }
            }
        }

        Ok(false)
    }
}

#[async_trait]
impl PolicyTrait for BasePolicy {
    async fn authorize(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult> {
        let rbac_result = self.authorize_rbac(context).await?;
        let abac_result = self.authorize_abac(context).await?;

        Ok(self.combine_results(rbac_result, abac_result).await)
    }

    async fn authorize_rbac(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult> {
        self.check_role_permissions(context).await
    }

    async fn authorize_abac(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult> {
        self.evaluate_policies(context).await
    }

    async fn check_role_permissions(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult> {
        let required_permission = format!("{}:{}",
            context.resource_type.as_ref().unwrap_or(&"*".to_string()),
            context.action
        );

        if self.rbac_authorizer.has_permission(context, &required_permission).await {
            return Ok(AuthorizationResult::allow(
                format!("User has required permission: {}", required_permission)
            ));
        }

        if self.rbac_authorizer.check_resource_ownership(context).await {
            return Ok(AuthorizationResult::allow(
                "User owns the resource".to_string()
            ));
        }

        for role in &context.roles {
            if role.name == "admin" || role.name == "super_admin" {
                return Ok(AuthorizationResult::allow(
                    format!("User has admin role: {}", role.name)
                ));
            }
        }

        Ok(AuthorizationResult::deny(
            format!("User lacks required permission: {}", required_permission)
        ))
    }

    async fn evaluate_policies(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult> {
        if !self.abac_authorizer.check_time_constraints(context).await {
            return Ok(AuthorizationResult::deny(
                "Time constraints not met".to_string()
            ));
        }

        if !self.abac_authorizer.check_location_constraints(context).await {
            return Ok(AuthorizationResult::deny(
                "Location constraints not met".to_string()
            ));
        }

        self.evaluate_policy_rules(context).await
    }

    async fn check_permission(&self, context: &AuthorizationContext, permission: &str) -> anyhow::Result<bool> {
        Ok(self.rbac_authorizer.has_permission(context, permission).await)
    }

    async fn check_role(&self, context: &AuthorizationContext, role: &str) -> anyhow::Result<bool> {
        Ok(self.rbac_authorizer.has_role(context, role).await)
    }

    async fn evaluate_condition(&self, condition: &str, context: &AuthorizationContext) -> anyhow::Result<bool> {
        self.abac_authorizer.evaluate_policy_condition(condition, context).await
    }
}