use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use ulid::Ulid;
use crate::app::models::{
    user::User,
    role::Role,
    permission::Permission,
    policy::{PolicyEffect},
};


#[derive(Debug, Clone)]
pub struct AuthorizationContext {
    pub user: User,
    pub roles: Vec<Role>,
    pub permissions: Vec<Permission>,
    pub attributes: HashMap<String, Value>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Ulid>,
    pub action: String,
    pub environment: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct AuthorizationResult {
    pub allowed: bool,
    pub reason: String,
    pub applied_policies: Vec<String>,
    pub effect: PolicyEffect,
}

impl AuthorizationResult {
    pub fn allow(reason: String) -> Self {
        Self {
            allowed: true,
            reason,
            applied_policies: vec![],
            effect: PolicyEffect::Permit,
        }
    }

    pub fn deny(reason: String) -> Self {
        Self {
            allowed: false,
            reason,
            applied_policies: vec![],
            effect: PolicyEffect::Deny,
        }
    }

    pub fn with_policies(mut self, policies: Vec<String>) -> Self {
        self.applied_policies = policies;
        self
    }
}

#[async_trait]
pub trait PolicyTrait: Send + Sync {
    async fn authorize(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult>;

    async fn authorize_rbac(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult> {
        self.check_role_permissions(context).await
    }

    async fn authorize_abac(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult> {
        self.evaluate_policies(context).await
    }

    async fn check_role_permissions(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult>;

    async fn evaluate_policies(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult>;

    async fn check_permission(&self, context: &AuthorizationContext, permission: &str) -> anyhow::Result<bool>;

    async fn check_role(&self, context: &AuthorizationContext, role: &str) -> anyhow::Result<bool>;

    async fn evaluate_condition(&self, condition: &str, context: &AuthorizationContext) -> anyhow::Result<bool>;
}

#[derive(Debug)]
pub struct RbacAuthorizer;

impl RbacAuthorizer {
    pub fn new() -> Self {
        Self
    }

    pub async fn has_permission(
        &self,
        context: &AuthorizationContext,
        required_permission: &str,
    ) -> bool {
        for permission in &context.permissions {
            if permission.name == required_permission
                || (permission.resource.as_ref().map_or(false, |r| r == context.resource_type.as_ref().unwrap_or(&String::new()))
                    && permission.action == context.action) {
                return true;
            }
        }
        false
    }

    pub async fn has_role(&self, context: &AuthorizationContext, required_role: &str) -> bool {
        context.roles.iter().any(|role| role.name == required_role)
    }

    pub async fn check_resource_ownership(&self, context: &AuthorizationContext) -> bool {
        if let Some(_resource_id) = context.resource_id {
            if let Some(owner_id) = context.attributes.get("owner_id") {
                if let Some(owner_ulid) = owner_id.as_str().and_then(|s| Ulid::from_string(s).ok()) {
                    return owner_ulid == context.user.id;
                }
            }
        }
        false
    }
}

#[derive(Debug)]
pub struct AbacAuthorizer;

impl AbacAuthorizer {
    pub fn new() -> Self {
        Self
    }

    pub async fn evaluate_policy_condition(
        &self,
        condition: &str,
        context: &AuthorizationContext,
    ) -> anyhow::Result<bool> {
        let mut evaluator = PolicyEvaluator::new(context);
        evaluator.evaluate(condition).await
    }

    pub async fn check_time_constraints(&self, context: &AuthorizationContext) -> bool {
        if let Some(current_time) = context.environment.get("current_time") {
            if let Some(allowed_hours) = context.attributes.get("allowed_hours") {
                if let (Some(time), Some(hours)) = (current_time.as_str(), allowed_hours.as_array()) {
                    return hours.iter().any(|h| h.as_str() == Some(time));
                }
            }
        }
        true
    }

    pub async fn check_location_constraints(&self, context: &AuthorizationContext) -> bool {
        if let Some(user_location) = context.environment.get("location") {
            if let Some(allowed_locations) = context.attributes.get("allowed_locations") {
                if let (Some(location), Some(locations)) = (user_location.as_str(), allowed_locations.as_array()) {
                    return locations.iter().any(|l| l.as_str() == Some(location));
                }
            }
        }
        true
    }
}

#[derive(Debug)]
pub struct PolicyEvaluator<'a> {
    context: &'a AuthorizationContext,
}

impl<'a> PolicyEvaluator<'a> {
    pub fn new(context: &'a AuthorizationContext) -> Self {
        Self { context }
    }

    pub async fn evaluate(&mut self, condition: &str) -> anyhow::Result<bool> {
        let conditions = self.parse_condition(condition)?;
        self.evaluate_conditions(&conditions).await
    }

    fn parse_condition(&self, condition: &str) -> anyhow::Result<Vec<ConditionClause>> {
        let mut clauses = Vec::new();

        for clause in condition.split(" AND ") {
            let clause = clause.trim();
            if clause.contains("user.role") {
                if let Some(role) = self.extract_value_from_condition(clause, "user.role") {
                    clauses.push(ConditionClause::HasRole(role));
                }
            } else if clause.contains("user.department") {
                if let Some(dept) = self.extract_value_from_condition(clause, "user.department") {
                    clauses.push(ConditionClause::HasAttribute("department".to_string(), dept));
                }
            } else if clause.contains("resource.owner") {
                clauses.push(ConditionClause::IsOwner);
            } else if clause.contains("time.hour") {
                if let Some(hours) = self.extract_array_from_condition(clause, "time.hour") {
                    clauses.push(ConditionClause::TimeConstraint(hours));
                }
            }
        }

        Ok(clauses)
    }

    async fn evaluate_conditions(&self, conditions: &[ConditionClause]) -> anyhow::Result<bool> {
        for condition in conditions {
            match condition {
                ConditionClause::HasRole(role) => {
                    if !self.context.roles.iter().any(|r| &r.name == role) {
                        return Ok(false);
                    }
                }
                ConditionClause::HasAttribute(attr_name, attr_value) => {
                    if let Some(value) = self.context.attributes.get(attr_name) {
                        if value.as_str() != Some(attr_value) {
                            return Ok(false);
                        }
                    } else {
                        return Ok(false);
                    }
                }
                ConditionClause::IsOwner => {
                    if let Some(_resource_id) = self.context.resource_id {
                        if let Some(owner_id) = self.context.attributes.get("owner_id") {
                            if let Some(owner_ulid) = owner_id.as_str().and_then(|s| Ulid::from_string(s).ok()) {
                                if owner_ulid != self.context.user.id {
                                    return Ok(false);
                                }
                            } else {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        }
                    }
                }
                ConditionClause::TimeConstraint(allowed_hours) => {
                    if let Some(current_hour) = self.context.environment.get("current_hour") {
                        if let Some(hour_str) = current_hour.as_str() {
                            if !allowed_hours.contains(&hour_str.to_string()) {
                                return Ok(false);
                            }
                        }
                    }
                }
            }
        }
        Ok(true)
    }

    fn extract_value_from_condition(&self, condition: &str, prefix: &str) -> Option<String> {
        if let Some(start) = condition.find(&format!("{} ==", prefix)) {
            let start = start + prefix.len() + 4;
            if let Some(end) = condition[start..].find('"') {
                let start_quote = start + end + 1;
                if let Some(end_quote) = condition[start_quote..].find('"') {
                    return Some(condition[start_quote..start_quote + end_quote].to_string());
                }
            }
        }
        None
    }

    fn extract_array_from_condition(&self, condition: &str, prefix: &str) -> Option<Vec<String>> {
        if let Some(start) = condition.find(&format!("{} in", prefix)) {
            let start = start + prefix.len() + 4;
            if let Some(array_start) = condition[start..].find('[') {
                if let Some(array_end) = condition[start + array_start..].find(']') {
                    let array_content = &condition[start + array_start + 1..start + array_start + array_end];
                    return Some(
                        array_content
                            .split(',')
                            .map(|s| s.trim().trim_matches('"').to_string())
                            .collect()
                    );
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
enum ConditionClause {
    HasRole(String),
    HasAttribute(String, String),
    IsOwner,
    TimeConstraint(Vec<String>),
}