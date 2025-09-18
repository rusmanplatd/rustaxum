use serde_json::{json, Value};
use std::collections::HashMap;
use ulid::Ulid;
use crate::app::models::{
    user::User,
    role::Role,
    permission::Permission,
    policy::{Policy, PolicyEffect},
    attribute::AttributeType,
    post::Post,
};
use crate::app::policies::{
    policy_trait::{AuthorizationContext, PolicyTrait},
    policy_service::PolicyService,
    post_policy::PostPolicy,
};

pub struct PolicyExamples;

impl PolicyExamples {
    pub async fn demonstrate_rbac() -> anyhow::Result<()> {
        println!("=== RBAC (Role-Based Access Control) Examples ===\n");

        let user = User::new(
            "John Doe".to_string(),
            "john@example.com".to_string(),
            "password".to_string()
        );

        let author_role = Role::new("author".to_string(), Some("Can write posts".to_string()), None);
        let editor_role = Role::new("editor".to_string(), Some("Can edit posts".to_string()), None);
        let admin_role = Role::new("admin".to_string(), Some("Full access".to_string()), None);

        let create_permission = Permission::new(
            "create:post".to_string(),
            None,
            Some("post".to_string()),
            "create".to_string()
        );

        let edit_permission = Permission::new(
            "edit:post".to_string(),
            None,
            Some("post".to_string()),
            "edit".to_string()
        );

        let delete_permission = Permission::new(
            "delete:post".to_string(),
            None,
            Some("post".to_string()),
            "delete".to_string()
        );

        println!("1. Author Role - Can Create Posts:");
        let context = AuthorizationContext {
            user: user.clone(),
            roles: vec![author_role.clone()],
            permissions: vec![create_permission.clone()],
            attributes: HashMap::new(),
            resource_type: Some("post".to_string()),
            resource_id: None,
            action: "create".to_string(),
            environment: HashMap::new(),
        };

        let post_policy = PostPolicy::new();
        let can_create = post_policy.create(&context).await?;
        println!("   Can create posts: {}\n", can_create);

        println!("2. Editor Role - Can Edit Posts:");
        let context = AuthorizationContext {
            user: user.clone(),
            roles: vec![editor_role.clone()],
            permissions: vec![edit_permission.clone()],
            attributes: HashMap::new(),
            resource_type: Some("post".to_string()),
            resource_id: Some(Ulid::new()),
            action: "edit".to_string(),
            environment: HashMap::new(),
        };

        let result = post_policy.authorize(&context).await?;
        println!("   Authorization result: {} - {}\n", result.allowed, result.reason);

        println!("3. Admin Role - Can Delete Posts:");
        let context = AuthorizationContext {
            user: user.clone(),
            roles: vec![admin_role.clone()],
            permissions: vec![delete_permission.clone()],
            attributes: HashMap::new(),
            resource_type: Some("post".to_string()),
            resource_id: Some(Ulid::new()),
            action: "delete".to_string(),
            environment: HashMap::new(),
        };

        let post = Post::new("Test Post".to_string(), "Content".to_string(), user.id);
        let can_delete = post_policy.delete(&context, &post).await?;
        println!("   Can delete posts: {}\n", can_delete);

        Ok(())
    }

    pub async fn demonstrate_abac() -> anyhow::Result<()> {
        println!("=== ABAC (Attribute-Based Access Control) Examples ===\n");

        let user = User::new(
            "Alice Smith".to_string(),
            "alice@example.com".to_string(),
            "password".to_string()
        );

        let editor_role = Role::new("editor".to_string(), None, None);

        println!("1. Department-Based Access Control:");
        let mut attributes = HashMap::new();
        attributes.insert("department".to_string(), json!("marketing"));
        attributes.insert("post_department".to_string(), json!("marketing"));

        let context = AuthorizationContext {
            user: user.clone(),
            roles: vec![editor_role.clone()],
            permissions: vec![],
            attributes: attributes.clone(),
            resource_type: Some("post".to_string()),
            resource_id: Some(Ulid::new()),
            action: "edit".to_string(),
            environment: HashMap::new(),
        };

        let post_policy = PostPolicy::new();
        let post = Post::new("Marketing Post".to_string(), "Content".to_string(), user.id);
        let can_edit = post_policy.update(&context, &post).await?;
        println!("   Can edit marketing post (same department): {}\n", can_edit);

        println!("2. Time-Based Access Control:");
        let mut environment = HashMap::new();
        environment.insert("current_hour".to_string(), json!("14")); // 2 PM

        attributes.insert("owner_id".to_string(), json!(user.id.to_string()));
        attributes.insert("can_publish_own_posts".to_string(), json!(true));

        let context = AuthorizationContext {
            user: user.clone(),
            roles: vec![],
            permissions: vec![],
            attributes: attributes.clone(),
            resource_type: Some("post".to_string()),
            resource_id: Some(Ulid::new()),
            action: "publish".to_string(),
            environment: environment.clone(),
        };

        let can_publish_business_hours = post_policy.publish(&context, &post).await?;
        println!("   Can publish during business hours (2 PM): {}", can_publish_business_hours);

        environment.insert("current_hour".to_string(), json!("22")); // 10 PM
        let context_late = AuthorizationContext {
            environment,
            ..context
        };

        let can_publish_after_hours = post_policy.publish(&context_late, &post).await?;
        println!("   Can publish after hours (10 PM): {}\n", can_publish_after_hours);

        println!("3. Location-Based Access Control:");
        let admin_role = Role::new("admin".to_string(), None, None);
        let mut environment = HashMap::new();
        environment.insert("location".to_string(), json!("office"));
        environment.insert("current_hour".to_string(), json!("10"));

        let mut attributes = HashMap::new();
        attributes.insert("bulk_update_locations".to_string(), json!(["office", "secure_facility"]));

        let context = AuthorizationContext {
            user: user.clone(),
            roles: vec![admin_role],
            permissions: vec![],
            attributes,
            resource_type: Some("post".to_string()),
            resource_id: None,
            action: "bulk_update".to_string(),
            environment,
        };

        let can_bulk_update = post_policy.bulk_update(&context).await?;
        println!("   Can perform bulk update from office: {}\n", can_bulk_update);

        Ok(())
    }

    pub async fn demonstrate_policy_evaluation() -> anyhow::Result<()> {
        println!("=== Policy Evaluation Examples ===\n");

        let user = User::new(
            "Bob Wilson".to_string(),
            "bob@example.com".to_string(),
            "password".to_string()
        );

        let moderator_role = Role::new("moderator".to_string(), None, None);

        println!("1. Complex Policy Conditions:");

        let policy1 = Policy::new(
            "moderator_access".to_string(),
            Some("Moderators can access content during business hours".to_string()),
            PolicyEffect::Permit,
            "role:moderator:*:moderate".to_string(),
            Some("user.role == \"moderator\" AND time.hour in [\"9\", \"10\", \"11\", \"12\", \"13\", \"14\", \"15\", \"16\", \"17\"]".to_string()),
            Some(100),
            Some(true),
        );

        let policy2 = Policy::new(
            "department_restriction".to_string(),
            Some("Users can only access their department's content".to_string()),
            PolicyEffect::Deny,
            "*:*:*".to_string(),
            Some("user.department != resource.department".to_string()),
            Some(200),
            Some(true),
        );

        let mut attributes = HashMap::new();
        attributes.insert("department".to_string(), json!("moderation"));
        attributes.insert("resource_department".to_string(), json!("moderation"));

        let mut environment = HashMap::new();
        environment.insert("current_hour".to_string(), json!("14"));

        let context = AuthorizationContext {
            user: user.clone(),
            roles: vec![moderator_role],
            permissions: vec![],
            attributes,
            resource_type: Some("post".to_string()),
            resource_id: Some(Ulid::new()),
            action: "moderate".to_string(),
            environment,
        };

        let post_policy = PostPolicy::new();
        let can_moderate = post_policy.moderate(&context).await?;
        println!("   Can moderate content: {}\n", can_moderate);

        println!("2. Ownership-Based Access:");
        let mut attributes = HashMap::new();
        attributes.insert("owner_id".to_string(), json!(user.id.to_string()));

        let context = AuthorizationContext {
            user: user.clone(),
            roles: vec![],
            permissions: vec![],
            attributes,
            resource_type: Some("post".to_string()),
            resource_id: Some(Ulid::new()),
            action: "edit".to_string(),
            environment: HashMap::new(),
        };

        let post = Post::new("User's Post".to_string(), "Content".to_string(), user.id);
        let can_edit_own = post_policy.update(&context, &post).await?;
        println!("   Can edit own post: {}\n", can_edit_own);

        Ok(())
    }

    pub async fn demonstrate_combined_rbac_abac() -> anyhow::Result<()> {
        println!("=== Combined RBAC + ABAC Examples ===\n");

        let user = User::new(
            "Charlie Brown".to_string(),
            "charlie@example.com".to_string(),
            "password".to_string()
        );

        let author_role = Role::new("author".to_string(), None, None);
        let edit_permission = Permission::new(
            "edit:own_posts".to_string(),
            None,
            Some("post".to_string()),
            "edit".to_string()
        );

        println!("1. RBAC + Ownership + Time Constraints:");
        let mut attributes = HashMap::new();
        attributes.insert("owner_id".to_string(), json!(user.id.to_string()));
        attributes.insert("department".to_string(), json!("content"));
        attributes.insert("seniority_level".to_string(), json!("senior"));

        let mut environment = HashMap::new();
        environment.insert("current_hour".to_string(), json!("11"));
        environment.insert("current_date".to_string(), json!("2024-01-15"));

        let context = AuthorizationContext {
            user: user.clone(),
            roles: vec![author_role],
            permissions: vec![edit_permission],
            attributes,
            resource_type: Some("post".to_string()),
            resource_id: Some(Ulid::new()),
            action: "edit".to_string(),
            environment,
        };

        let post_policy = PostPolicy::new();
        let result = post_policy.authorize(&context).await?;
        println!("   Authorization result: {}", result.allowed);
        println!("   Reason: {}", result.reason);
        println!("   Applied policies: {:?}\n", result.applied_policies);

        println!("2. Multi-Factor Authorization:");

        let context_factors = AuthorizationContext {
            user: user.clone(),
            roles: vec![Role::new("senior_author".to_string(), None, None)],
            permissions: vec![
                Permission::new("create:post".to_string(), None, Some("post".to_string()), "create".to_string()),
                Permission::new("edit:any_post".to_string(), None, Some("post".to_string()), "edit".to_string()),
            ],
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("security_clearance".to_string(), json!("level_2"));
                attrs.insert("two_factor_enabled".to_string(), json!(true));
                attrs.insert("last_security_training".to_string(), json!("2024-01-01"));
                attrs
            },
            resource_type: Some("post".to_string()),
            resource_id: Some(Ulid::new()),
            action: "publish".to_string(),
            environment: {
                let mut env = HashMap::new();
                env.insert("ip_address".to_string(), json!("192.168.1.100"));
                env.insert("device_trusted".to_string(), json!(true));
                env.insert("current_hour".to_string(), json!("13"));
                env
            },
        };

        let post = Post::new("Important Post".to_string(), "Sensitive content".to_string(), user.id);
        let can_publish_sensitive = post_policy.publish(&context_factors, &post).await?;
        println!("   Can publish sensitive content: {}\n", can_publish_sensitive);

        Ok(())
    }

    pub async fn run_all_examples() -> anyhow::Result<()> {
        println!("🔐 Policy System Examples - RBAC & ABAC Implementation\n");
        println!("=====================================================\n");

        Self::demonstrate_rbac().await?;
        println!("\n{}\n", "=".repeat(60));

        Self::demonstrate_abac().await?;
        println!("\n{}\n", "=".repeat(60));

        Self::demonstrate_policy_evaluation().await?;
        println!("\n{}\n", "=".repeat(60));

        Self::demonstrate_combined_rbac_abac().await?;

        println!("\n🎉 All examples completed successfully!");
        println!("\nKey Features Demonstrated:");
        println!("• Role-based access control (RBAC)");
        println!("• Attribute-based access control (ABAC)");
        println!("• Time-based constraints");
        println!("• Location-based constraints");
        println!("• Department-based access");
        println!("• Resource ownership");
        println!("• Multi-factor authorization");
        println!("• Policy condition evaluation");
        println!("• Combined RBAC + ABAC scenarios");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rbac_examples() {
        let result = PolicyExamples::demonstrate_rbac().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_abac_examples() {
        let result = PolicyExamples::demonstrate_abac().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_policy_evaluation_examples() {
        let result = PolicyExamples::demonstrate_policy_evaluation().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_combined_examples() {
        let result = PolicyExamples::demonstrate_combined_rbac_abac().await;
        assert!(result.is_ok());
    }
}