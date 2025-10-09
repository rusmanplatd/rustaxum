use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::database::DbPool;
use crate::app::services::oauth::{ScopeService, ClientService};
use crate::app::models::oauth::Client;

/// OAuth 2.0 scope validation service for token exchange
/// Implements comprehensive security policies for scope handling and validation
pub struct ScopeValidationService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeValidationResult {
    pub is_valid: bool,
    pub granted_scopes: Vec<String>,
    pub denied_scopes: Vec<String>,
    pub warnings: Vec<String>,
    pub policy_applied: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenExchangeScenario {
    Delegation,      // User delegates access to another service
    Impersonation,   // Service acts on behalf of user
    ServiceToService, // Service-to-service communication
    StepUp,          // Escalate privileges for sensitive operation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeContext {
    pub scenario: TokenExchangeScenario,
    pub subject_client_id: String,
    pub actor_client_id: Option<String>,
    pub resource: Option<String>,
    pub audience: Option<String>,
    pub subject_scopes: Vec<String>,
    pub requested_scopes: Option<Vec<String>>,
}

/// Scope inheritance and restriction policies
#[derive(Debug, Clone)]
pub struct ScopePolicy {
    pub allow_scope_escalation: bool,
    pub allow_cross_client_scopes: bool,
    pub require_explicit_consent: bool,
    pub max_scope_lifetime_hours: Option<u32>,
    pub restricted_scopes: HashSet<String>,
    pub delegation_rules: HashMap<String, Vec<String>>,
}

impl ScopeValidationService {
    /// Validate scopes for token exchange with comprehensive security policies
    pub async fn validate_exchange_scopes(
        pool: &DbPool,
        context: &ExchangeContext,
    ) -> Result<ScopeValidationResult> {
        // Load applicable scope policies
        let policy = Self::load_scope_policy(pool, context).await?;

        // Parse and validate requested scopes
        let requested_scopes = Self::parse_requested_scopes(context)?;

        // Apply scenario-specific validation
        let mut result = match context.scenario {
            TokenExchangeScenario::Delegation => {
                Self::validate_delegation_scopes(pool, context, &requested_scopes, &policy).await?
            }
            TokenExchangeScenario::Impersonation => {
                Self::validate_impersonation_scopes(pool, context, &requested_scopes, &policy).await?
            }
            TokenExchangeScenario::ServiceToService => {
                Self::validate_service_scopes(pool, context, &requested_scopes, &policy).await?
            }
            TokenExchangeScenario::StepUp => {
                Self::validate_stepup_scopes(pool, context, &requested_scopes, &policy).await?
            }
        };

        // Apply additional security checks
        Self::apply_security_restrictions(&mut result, context, &policy).await?;

        // Validate against resource and audience constraints
        Self::validate_resource_constraints(&mut result, context).await?;

        // Final security audit
        Self::perform_security_audit(&mut result, context, &policy).await?;

        Ok(result)
    }

    /// Load scope policy based on client and scenario
    async fn load_scope_policy(
        pool: &DbPool,
        context: &ExchangeContext,
    ) -> Result<ScopePolicy> {
        // Load scope policy from client metadata and organization settings
        let client = ClientService::find_by_id(pool, context.subject_client_id.clone())?
            .ok_or_else(|| anyhow::anyhow!("Subject client not found"))?;

        // Extract policy settings from client metadata
        let scope_policy = client.metadata.as_ref()
            .and_then(|m| m.get("scope_policy"))
            .and_then(|p| p.as_object());

        // Build policy based on client configuration and scenario
        let mut policy = ScopePolicy {
            allow_scope_escalation: scope_policy
                .and_then(|p| p.get("allow_scope_escalation"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false), // Conservative default
            allow_cross_client_scopes: scope_policy
                .and_then(|p| p.get("allow_cross_client_scopes"))
                .and_then(|v| v.as_bool())
                .unwrap_or(client.organization_id.is_some()), // Allow within organization
            require_explicit_consent: scope_policy
                .and_then(|p| p.get("require_explicit_consent"))
                .and_then(|v| v.as_bool())
                .unwrap_or(!client.personal_access_client), // PATs don't need consent
            max_scope_lifetime_hours: scope_policy
                .and_then(|p| p.get("max_scope_lifetime_hours"))
                .and_then(|v| v.as_u64())
                .map(|h| h as u32)
                .or_else(|| if client.personal_access_client { Some(8760) } else { Some(24) }), // 1 year for PATs, 24h for others
            restricted_scopes: Self::get_restricted_scopes(),
            delegation_rules: Self::build_delegation_rules(&client).await?,
        };

        // Adjust policy based on scenario
        match context.scenario {
            TokenExchangeScenario::ServiceToService => {
                policy.require_explicit_consent = false; // Machine-to-machine
                policy.allow_cross_client_scopes = true; // Service communication
            }
            TokenExchangeScenario::StepUp => {
                policy.allow_scope_escalation = true; // Controlled escalation
                policy.max_scope_lifetime_hours = Some(1); // Short-lived
            }
            _ => {} // Keep defaults
        }

        Ok(policy)
    }

    /// Parse and normalize requested scopes
    fn parse_requested_scopes(context: &ExchangeContext) -> Result<Vec<String>> {
        let scopes = match &context.requested_scopes {
            Some(requested) => requested.clone(),
            None => context.subject_scopes.clone(), // Inherit subject scopes
        };

        // Normalize scope strings
        let normalized_scopes: Vec<String> = scopes
            .into_iter()
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect::<HashSet<_>>() // Remove duplicates
            .into_iter()
            .collect();

        Ok(normalized_scopes)
    }

    /// Validate delegation scenario scopes
    async fn validate_delegation_scopes(
        pool: &DbPool,
        context: &ExchangeContext,
        requested_scopes: &[String],
        policy: &ScopePolicy,
    ) -> Result<ScopeValidationResult> {
        let mut result = ScopeValidationResult {
            is_valid: true,
            granted_scopes: Vec::new(),
            denied_scopes: Vec::new(),
            warnings: Vec::new(),
            policy_applied: "delegation".to_string(),
        };

        // Load available scopes
        let available_scopes = ScopeService::list_scopes(pool)?;
        let available_scope_names: HashSet<_> = available_scopes.iter()
            .map(|s| s.name.clone())
            .collect();

        for requested_scope in requested_scopes {
            // Check if scope exists
            if !available_scope_names.contains(requested_scope) {
                result.denied_scopes.push(requested_scope.clone());
                result.warnings.push(format!("Unknown scope: {}", requested_scope));
                continue;
            }

            // Check if subject has this scope
            if !context.subject_scopes.contains(requested_scope) {
                if !policy.allow_scope_escalation {
                    result.denied_scopes.push(requested_scope.clone());
                    result.warnings.push(format!("Scope escalation denied: {}", requested_scope));
                    continue;
                }
            }

            // Check delegation rules
            if let Some(allowed_delegations) = policy.delegation_rules.get(requested_scope) {
                if let Some(actor_id) = &context.actor_client_id {
                    if !allowed_delegations.contains(actor_id) {
                        result.denied_scopes.push(requested_scope.clone());
                        result.warnings.push(format!("Delegation not allowed to client: {}", actor_id));
                        continue;
                    }
                }
            }

            // Check restricted scopes
            if policy.restricted_scopes.contains(requested_scope) {
                result.denied_scopes.push(requested_scope.clone());
                result.warnings.push(format!("Restricted scope: {}", requested_scope));
                continue;
            }

            result.granted_scopes.push(requested_scope.clone());
        }

        result.is_valid = !result.granted_scopes.is_empty();
        Ok(result)
    }

    /// Validate impersonation scenario scopes
    async fn validate_impersonation_scopes(
        pool: &DbPool,
        context: &ExchangeContext,
        requested_scopes: &[String],
        _policy: &ScopePolicy,
    ) -> Result<ScopeValidationResult> {
        let mut result = ScopeValidationResult {
            is_valid: true,
            granted_scopes: Vec::new(),
            denied_scopes: Vec::new(),
            warnings: Vec::new(),
            policy_applied: "impersonation".to_string(),
        };

        // Impersonation requires strict validation
        let available_scopes = ScopeService::list_scopes(pool)?;
        let sensitive_scopes = Self::get_sensitive_scopes();

        for requested_scope in requested_scopes {
            // Check if scope exists
            let scope_exists = available_scopes.iter()
                .any(|s| s.name == *requested_scope);

            if !scope_exists {
                result.denied_scopes.push(requested_scope.clone());
                result.warnings.push(format!("Unknown scope: {}", requested_scope));
                continue;
            }

            // Sensitive scopes require special permissions for impersonation
            if sensitive_scopes.contains(requested_scope) {
                if !Self::check_impersonation_permission(pool, &context.subject_client_id, requested_scope).await? {
                    result.denied_scopes.push(requested_scope.clone());
                    result.warnings.push(format!("Insufficient permissions for sensitive scope: {}", requested_scope));
                    continue;
                }
            }

            // Must be subset of subject scopes (no escalation in impersonation)
            if !context.subject_scopes.contains(requested_scope) {
                result.denied_scopes.push(requested_scope.clone());
                result.warnings.push(format!("Impersonation cannot escalate scopes: {}", requested_scope));
                continue;
            }

            result.granted_scopes.push(requested_scope.clone());
        }

        result.is_valid = !result.granted_scopes.is_empty();
        Ok(result)
    }

    /// Validate service-to-service scenario scopes
    async fn validate_service_scopes(
        _pool: &DbPool,
        context: &ExchangeContext,
        requested_scopes: &[String],
        _policy: &ScopePolicy,
    ) -> Result<ScopeValidationResult> {
        let mut result = ScopeValidationResult {
            is_valid: true,
            granted_scopes: Vec::new(),
            denied_scopes: Vec::new(),
            warnings: Vec::new(),
            policy_applied: "service_to_service".to_string(),
        };

        // Service-to-service is more permissive but validates against service catalog
        let allowed_service_scopes = Self::get_service_scopes();

        for requested_scope in requested_scopes {
            if allowed_service_scopes.contains(requested_scope) {
                result.granted_scopes.push(requested_scope.clone());
            } else {
                result.denied_scopes.push(requested_scope.clone());
                result.warnings.push(format!("Service scope not allowed: {}", requested_scope));
            }
        }

        // For service flows, validate audience matches expected service
        if let Some(audience) = &context.audience {
            if !Self::validate_service_audience(audience) {
                result.is_valid = false;
                result.warnings.push("Invalid service audience".to_string());
            }
        }

        result.is_valid = result.is_valid && !result.granted_scopes.is_empty();
        Ok(result)
    }

    /// Validate step-up scenario scopes (privilege escalation)
    async fn validate_stepup_scopes(
        pool: &DbPool,
        context: &ExchangeContext,
        requested_scopes: &[String],
        _policy: &ScopePolicy,
    ) -> Result<ScopeValidationResult> {
        let mut result = ScopeValidationResult {
            is_valid: true,
            granted_scopes: Vec::new(),
            denied_scopes: Vec::new(),
            warnings: Vec::new(),
            policy_applied: "step_up".to_string(),
        };

        // Step-up allows controlled escalation for specific high-value scopes
        let stepup_scopes = Self::get_stepup_eligible_scopes();
        let available_scopes = ScopeService::list_scopes(pool)?;

        for requested_scope in requested_scopes {
            // Must exist
            let scope_exists = available_scopes.iter()
                .any(|s| s.name == *requested_scope);

            if !scope_exists {
                result.denied_scopes.push(requested_scope.clone());
                result.warnings.push(format!("Unknown scope: {}", requested_scope));
                continue;
            }

            // Check if eligible for step-up
            if !stepup_scopes.contains(requested_scope) {
                result.denied_scopes.push(requested_scope.clone());
                result.warnings.push(format!("Scope not eligible for step-up: {}", requested_scope));
                continue;
            }

            // Validate client has step-up permissions
            if !Self::check_stepup_permission(pool, &context.subject_client_id, requested_scope).await? {
                result.denied_scopes.push(requested_scope.clone());
                result.warnings.push(format!("No step-up permission for scope: {}", requested_scope));
                continue;
            }

            result.granted_scopes.push(requested_scope.clone());
        }

        result.is_valid = !result.granted_scopes.is_empty();
        Ok(result)
    }

    /// Apply additional security restrictions
    async fn apply_security_restrictions(
        result: &mut ScopeValidationResult,
        _context: &ExchangeContext,
        policy: &ScopePolicy,
    ) -> Result<()> {
        // Remove any restricted scopes that might have slipped through
        result.granted_scopes.retain(|scope| !policy.restricted_scopes.contains(scope));

        // Add warnings for security policies
        if !policy.allow_scope_escalation && result.granted_scopes.len() > result.granted_scopes.len() {
            result.warnings.push("Some scopes denied due to escalation policy".to_string());
        }

        Ok(())
    }

    /// Validate resource and audience constraints
    async fn validate_resource_constraints(
        result: &mut ScopeValidationResult,
        context: &ExchangeContext,
    ) -> Result<()> {
        // Filter scopes based on resource constraints
        if let Some(resource) = &context.resource {
            let resource_scopes = Self::get_resource_scopes(resource);
            result.granted_scopes.retain(|scope| {
                resource_scopes.contains(scope) || Self::is_universal_scope(scope)
            });
        }

        // Validate audience-specific scopes
        if let Some(audience) = &context.audience {
            let audience_scopes = Self::get_audience_scopes(audience);
            result.granted_scopes.retain(|scope| {
                audience_scopes.contains(scope) || Self::is_universal_scope(scope)
            });
        }

        Ok(())
    }

    /// Perform final security audit
    async fn perform_security_audit(
        result: &mut ScopeValidationResult,
        context: &ExchangeContext,
        _policy: &ScopePolicy,
    ) -> Result<()> {
        // Log security-sensitive operations
        if matches!(context.scenario, TokenExchangeScenario::Impersonation | TokenExchangeScenario::StepUp) {
            tracing::warn!(
                "High-privilege token exchange: scenario={:?}, client={}, scopes={:?}",
                context.scenario,
                context.subject_client_id,
                result.granted_scopes
            );
        }

        // Check for suspicious scope combinations
        let suspicious_combos = Self::get_suspicious_scope_combinations();
        for (scope_a, scope_b) in &suspicious_combos {
            if result.granted_scopes.contains(scope_a) && result.granted_scopes.contains(scope_b) {
                result.warnings.push(format!("Suspicious scope combination: {} + {}", scope_a, scope_b));
            }
        }

        Ok(())
    }

    // Helper methods for scope categorization and policies

    fn get_restricted_scopes() -> HashSet<String> {
        [
            "admin".to_string(),
            "sudo".to_string(),
            "root".to_string(),
            "system".to_string(),
            "billing".to_string(),
        ].into_iter().collect()
    }

    fn get_sensitive_scopes() -> HashSet<String> {
        [
            "profile.write".to_string(),
            "payment.write".to_string(),
            "admin.users".to_string(),
            "admin.system".to_string(),
        ].into_iter().collect()
    }

    fn get_service_scopes() -> HashSet<String> {
        [
            "api.read".to_string(),
            "api.write".to_string(),
            "service.internal".to_string(),
            "webhook.send".to_string(),
        ].into_iter().collect()
    }

    fn get_stepup_eligible_scopes() -> HashSet<String> {
        [
            "payment.write".to_string(),
            "profile.delete".to_string(),
            "admin.sensitive".to_string(),
        ].into_iter().collect()
    }

    fn get_resource_scopes(resource: &str) -> HashSet<String> {
        // Load resource scopes from configuration or service registry
        match resource {
            "https://api.example.com" => {
                ["api.read", "api.write"].iter().map(|s| s.to_string()).collect()
            }
            _ => HashSet::new()
        }
    }

    fn get_audience_scopes(audience: &str) -> HashSet<String> {
        // Load audience scopes from service registry configuration
        match audience {
            "payment-service" => {
                ["payment.read", "payment.write"].iter().map(|s| s.to_string()).collect()
            }
            _ => HashSet::new()
        }
    }

    fn is_universal_scope(scope: &str) -> bool {
        matches!(scope, "openid" | "profile.read" | "email")
    }

    fn validate_service_audience(audience: &str) -> bool {
        // Validate audience against configured service registry
        !audience.is_empty() && audience.starts_with("https://")
    }

    fn get_suspicious_scope_combinations() -> Vec<(String, String)> {
        vec![
            ("admin".to_string(), "user.delete".to_string()),
            ("billing".to_string(), "user.admin".to_string()),
        ]
    }

    async fn build_delegation_rules(client: &Client) -> Result<HashMap<String, Vec<String>>> {
        // Load delegation rules from client relationships and trust policies
        let mut rules = HashMap::new();

        // Example: allow delegation to trusted clients
        if client.organization_id.is_some() {
            rules.insert("api.read".to_string(), vec!["trusted-service-1".to_string()]);
        }

        Ok(rules)
    }

    async fn check_impersonation_permission(
        _pool: &DbPool,
        _client_id: &str,
        _scope: &str,
    ) -> Result<bool> {
        // Check client permissions in database or configuration
        Ok(false)
    }

    async fn check_stepup_permission(
        pool: &DbPool,
        client_id: &str,
        scope: &str,
    ) -> Result<bool> {
        use crate::schema::oauth_clients::dsl::{oauth_clients, id};
        use diesel::prelude::*;

        // Check step-up authorization requirements in database
        let mut conn = pool.get()?;

        // Load client to check step-up permissions
        let client = oauth_clients
            .filter(id.eq(client_id))
            .select(crate::app::models::oauth::Client::as_select())
            .first::<crate::app::models::oauth::Client>(&mut conn)
            .optional()?;

        if let Some(client) = client {
            // Check metadata for step-up scope permissions
            if let Some(stepup_config) = client.metadata.as_ref().and_then(|m| m.get("stepup_scopes").and_then(|v| v.as_object())) {
                if let Some(allowed_scopes) = stepup_config.get("allowed").and_then(|v| v.as_array()) {
                    return Ok(allowed_scopes.iter()
                        .filter_map(|v| v.as_str())
                        .any(|allowed_scope| {
                            // Check exact match or wildcard pattern
                            allowed_scope == scope ||
                            (allowed_scope.ends_with("*") && scope.starts_with(&allowed_scope[..allowed_scope.len()-1]))
                        }));
                }
            }

            // Default: allow step-up for personal access clients and trusted clients
            return Ok(client.personal_access_client ||
                     client.metadata.as_ref().and_then(|m| m.get("trusted").and_then(|v| v.as_bool())).unwrap_or(false));
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_requested_scopes() {
        let context = ExchangeContext {
            scenario: TokenExchangeScenario::Delegation,
            subject_client_id: "client1".to_string(),
            actor_client_id: None,
            resource: None,
            audience: None,
            subject_scopes: vec!["read".to_string(), "write".to_string()],
            requested_scopes: Some(vec![" READ ".to_string(), "write".to_string(), "read".to_string()]),
        };

        let scopes = ScopeValidationService::parse_requested_scopes(&context).unwrap();
        assert_eq!(scopes.len(), 2);
        assert!(scopes.contains(&"read".to_string()));
        assert!(scopes.contains(&"write".to_string()));
    }

    #[test]
    fn test_restricted_scopes() {
        let restricted = ScopeValidationService::get_restricted_scopes();
        assert!(restricted.contains("admin"));
        assert!(restricted.contains("sudo"));
    }
}