use async_trait::async_trait;
use ulid::Ulid;
use crate::app::models::{post::Post};
use crate::app::policies::policy_trait::{
    PolicyTrait, AuthorizationContext, AuthorizationResult
};
use crate::app::policies::base_policy::BasePolicy;

#[derive(Debug)]
pub struct PostPolicy {
    base: BasePolicy,
}

impl PostPolicy {
    pub fn new() -> Self {
        Self {
            base: BasePolicy::new(),
        }
    }

    pub async fn view(&self, context: &AuthorizationContext, post: &Post) -> anyhow::Result<bool> {
        let result = self.authorize(context).await?;

        if result.allowed {
            return Ok(true);
        }

        if post.published {
            return Ok(true);
        }

        if let Some(owner_id) = context.attributes.get("owner_id") {
            if let Some(owner_ulid) = owner_id.as_str().and_then(|s| Ulid::from_string(s).ok()) {
                return Ok(owner_ulid == context.user.id);
            }
        }

        Ok(false)
    }

    pub async fn create(&self, context: &AuthorizationContext) -> anyhow::Result<bool> {
        if context.roles.iter().any(|r| r.name == "author" || r.name == "admin") {
            return Ok(true);
        }

        if context.permissions.iter().any(|p| p.action == "create" && p.resource.as_ref() == Some(&"post".to_string())) {
            return Ok(true);
        }

        if let Some(can_create) = context.attributes.get("can_create_posts") {
            if can_create.as_bool() == Some(true) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn update(&self, context: &AuthorizationContext, _post: &Post) -> anyhow::Result<bool> {
        if context.roles.iter().any(|r| r.name == "admin") {
            return Ok(true);
        }

        if let Some(owner_id) = context.attributes.get("owner_id") {
            if let Some(owner_ulid) = owner_id.as_str().and_then(|s| Ulid::from_string(s).ok()) {
                if owner_ulid == context.user.id {
                    return Ok(true);
                }
            }
        }

        if context.roles.iter().any(|r| r.name == "editor") {
            if let Some(department) = context.attributes.get("department") {
                if let Some(post_department) = context.attributes.get("post_department") {
                    return Ok(department == post_department);
                }
            }
        }

        Ok(false)
    }

    pub async fn delete(&self, context: &AuthorizationContext, _post: &Post) -> anyhow::Result<bool> {
        if context.roles.iter().any(|r| r.name == "admin") {
            return Ok(true);
        }

        if let Some(owner_id) = context.attributes.get("owner_id") {
            if let Some(owner_ulid) = owner_id.as_str().and_then(|s| Ulid::from_string(s).ok()) {
                if owner_ulid == context.user.id {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub async fn publish(&self, context: &AuthorizationContext, _post: &Post) -> anyhow::Result<bool> {
        if context.roles.iter().any(|r| r.name == "admin" || r.name == "publisher") {
            return Ok(true);
        }

        if let Some(owner_id) = context.attributes.get("owner_id") {
            if let Some(owner_ulid) = owner_id.as_str().and_then(|s| Ulid::from_string(s).ok()) {
                if owner_ulid == context.user.id {
                    if let Some(can_publish) = context.attributes.get("can_publish_own_posts") {
                        return Ok(can_publish.as_bool() == Some(true));
                    }
                }
            }
        }

        if let Some(current_hour) = context.environment.get("current_hour") {
            if let Some(hour_str) = current_hour.as_str() {
                let hour: i32 = hour_str.parse().unwrap_or(0);
                if hour < 9 || hour > 17 {
                    return Ok(false);
                }
            }
        }

        Ok(false)
    }

    pub async fn moderate(&self, context: &AuthorizationContext) -> anyhow::Result<bool> {
        if context.roles.iter().any(|r| r.name == "admin" || r.name == "moderator") {
            return Ok(true);
        }

        if let Some(has_moderation_rights) = context.attributes.get("has_moderation_rights") {
            return Ok(has_moderation_rights.as_bool() == Some(true));
        }

        Ok(false)
    }

    pub async fn bulk_update(&self, context: &AuthorizationContext) -> anyhow::Result<bool> {
        if !context.roles.iter().any(|r| r.name == "admin") {
            return Ok(false);
        }

        if let Some(current_hour) = context.environment.get("current_hour") {
            if let Some(hour_str) = current_hour.as_str() {
                let hour: i32 = hour_str.parse().unwrap_or(0);
                if hour < 9 || hour > 17 {
                    return Ok(false);
                }
            }
        }

        if let Some(location) = context.environment.get("location") {
            if let Some(allowed_locations) = context.attributes.get("bulk_update_locations") {
                if let Some(locations_array) = allowed_locations.as_array() {
                    if !locations_array.iter().any(|l| l.as_str() == location.as_str()) {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }
}

#[async_trait]
impl PolicyTrait for PostPolicy {
    async fn authorize(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult> {
        self.base.authorize(context).await
    }

    async fn check_role_permissions(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult> {
        self.base.check_role_permissions(context).await
    }

    async fn evaluate_policies(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult> {
        self.base.evaluate_policies(context).await
    }

    async fn check_permission(&self, context: &AuthorizationContext, permission: &str) -> anyhow::Result<bool> {
        self.base.check_permission(context, permission).await
    }

    async fn check_role(&self, context: &AuthorizationContext, role: &str) -> anyhow::Result<bool> {
        self.base.check_role(context, role).await
    }

    async fn evaluate_condition(&self, condition: &str, context: &AuthorizationContext) -> anyhow::Result<bool> {
        self.base.evaluate_condition(condition, context).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::models::{role::Role, permission::Permission};
    use chrono::Utc;
    use serde_json::json;

    fn create_test_user() -> User {
        User::new(
            "John Doe".to_string(),
            "john@example.com".to_string(),
            "password".to_string()
        )
    }

    fn create_test_post() -> Post {
        Post::new(
            "Test Post".to_string(),
            "This is a test post".to_string(),
            Ulid::new(),
        )
    }

    #[tokio::test]
    async fn test_admin_can_do_everything() {
        let user = create_test_user();
        let post = create_test_post();
        let admin_role = Role::new("admin".to_string(), None, None);

        let context = AuthorizationContext {
            user: user.clone(),
            roles: vec![admin_role],
            permissions: vec![],
            attributes: HashMap::new(),
            resource_type: Some("post".to_string()),
            resource_id: Some(post.id),
            action: "delete".to_string(),
            environment: HashMap::new(),
        };

        let policy = PostPolicy::new();

        assert!(policy.view(&context, &post).await.unwrap());
        assert!(policy.create(&context).await.unwrap());
        assert!(policy.update(&context, &post).await.unwrap());
        assert!(policy.delete(&context, &post).await.unwrap());
        assert!(policy.publish(&context, &post).await.unwrap());
        assert!(policy.moderate(&context).await.unwrap());
    }

    #[tokio::test]
    async fn test_owner_can_modify_own_post() {
        let user = create_test_user();
        let post = create_test_post();

        let mut attributes = HashMap::new();
        attributes.insert("owner_id".to_string(), json!(user.id.to_string()));

        let context = AuthorizationContext {
            user: user.clone(),
            roles: vec![],
            permissions: vec![],
            attributes,
            resource_type: Some("post".to_string()),
            resource_id: Some(post.id),
            action: "update".to_string(),
            environment: HashMap::new(),
        };

        let policy = PostPolicy::new();

        assert!(policy.view(&context, &post).await.unwrap());
        assert!(policy.update(&context, &post).await.unwrap());
        assert!(policy.delete(&context, &post).await.unwrap());
    }

    #[tokio::test]
    async fn test_time_constraints_for_publishing() {
        let user = create_test_user();
        let post = create_test_post();

        let mut attributes = HashMap::new();
        attributes.insert("owner_id".to_string(), json!(user.id.to_string()));
        attributes.insert("can_publish_own_posts".to_string(), json!(true));

        let mut environment = HashMap::new();
        environment.insert("current_hour".to_string(), json!("22"));

        let context = AuthorizationContext {
            user: user.clone(),
            roles: vec![],
            permissions: vec![],
            attributes,
            resource_type: Some("post".to_string()),
            resource_id: Some(post.id),
            action: "publish".to_string(),
            environment,
        };

        let policy = PostPolicy::new();

        assert!(!policy.publish(&context, &post).await.unwrap());
    }

    #[tokio::test]
    async fn test_department_based_editing() {
        let user = create_test_user();
        let post = create_test_post();
        let editor_role = Role::new("editor".to_string(), None, None);

        let mut attributes = HashMap::new();
        attributes.insert("department".to_string(), json!("marketing"));
        attributes.insert("post_department".to_string(), json!("marketing"));

        let context = AuthorizationContext {
            user: user.clone(),
            roles: vec![editor_role],
            permissions: vec![],
            attributes,
            resource_type: Some("post".to_string()),
            resource_id: Some(post.id),
            action: "update".to_string(),
            environment: HashMap::new(),
        };

        let policy = PostPolicy::new();

        assert!(policy.update(&context, &post).await.unwrap());
    }
}