use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use ulid::Ulid;
use crate::app::models::{
    user::User,
    role::Role,
    permission::Permission,
    attribute::Attribute,
    policy::Policy,
};
use crate::app::policies::policy_trait::{
    PolicyTrait, AuthorizationContext, AuthorizationResult
};
use crate::app::policies::base_policy::BasePolicy;

#[derive(Debug)]
pub struct PolicyService {
    pool: PgPool,
}

impl PolicyService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn authorize_user(
        &self,
        user: &User,
        action: &str,
        resource_type: Option<String>,
        resource_id: Option<Ulid>,
        environment: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<AuthorizationResult> {
        let context = self.build_authorization_context(
            user, action, resource_type, resource_id, environment
        ).await?;

        let policies = self.get_applicable_policies(&context).await?;
        let policy = BasePolicy::new().with_policies(policies);

        policy.authorize(&context).await
    }

    pub async fn can_user(
        &self,
        user: &User,
        permission: &str,
        resource_type: Option<String>,
        resource_id: Option<Ulid>,
    ) -> anyhow::Result<bool> {
        let result = self.authorize_user(user, permission, resource_type, resource_id, None).await?;
        Ok(result.allowed)
    }

    pub async fn user_has_role(&self, user: &User, role_name: &str) -> anyhow::Result<bool> {
        let roles = self.get_user_roles(user).await?;
        Ok(roles.iter().any(|role| role.name == role_name))
    }

    pub async fn user_has_permission(
        &self,
        user: &User,
        permission_name: &str,
        resource_type: Option<String>,
    ) -> anyhow::Result<bool> {
        let permissions = self.get_user_permissions(user).await?;

        Ok(permissions.iter().any(|perm| {
            perm.name == permission_name &&
            (resource_type.is_none() || perm.resource.as_ref() == resource_type.as_ref())
        }))
    }

    pub async fn get_user_roles(&self, user: &User) -> anyhow::Result<Vec<Role>> {
        let roles = sqlx::query_as::<_, Role>(
            r#"
            SELECT r.* FROM roles r
            INNER JOIN user_roles ur ON r.id = ur.role_id
            WHERE ur.user_id = $1
            "#
        )
        .bind(user.id.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(roles)
    }

    pub async fn get_user_permissions(&self, user: &User) -> anyhow::Result<Vec<Permission>> {
        let permissions = sqlx::query_as::<_, Permission>(
            r#"
            SELECT DISTINCT p.* FROM permissions p
            INNER JOIN role_permissions rp ON p.id = rp.permission_id
            INNER JOIN user_roles ur ON rp.role_id = ur.role_id
            WHERE ur.user_id = $1
            "#
        )
        .bind(user.id.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(permissions)
    }

    pub async fn get_user_attributes(&self, user: &User) -> anyhow::Result<HashMap<String, Value>> {
        let attributes = sqlx::query_as::<_, Attribute>(
            r#"
            SELECT * FROM attributes
            WHERE subject_type = 'user' AND subject_id = $1
            "#
        )
        .bind(user.id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let mut attr_map = HashMap::new();
        for attr in attributes {
            attr_map.insert(attr.name, attr.value);
        }

        Ok(attr_map)
    }

    pub async fn get_resource_attributes(
        &self,
        resource_type: &str,
        resource_id: Ulid,
    ) -> anyhow::Result<HashMap<String, Value>> {
        let attributes = sqlx::query_as::<_, Attribute>(
            r#"
            SELECT * FROM attributes
            WHERE resource_type = $1 AND resource_id = $2
            "#
        )
        .bind(resource_type)
        .bind(resource_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        let mut attr_map = HashMap::new();
        for attr in attributes {
            attr_map.insert(attr.name, attr.value);
        }

        Ok(attr_map)
    }

    async fn build_authorization_context(
        &self,
        user: &User,
        action: &str,
        resource_type: Option<String>,
        resource_id: Option<Ulid>,
        environment: Option<HashMap<String, Value>>,
    ) -> anyhow::Result<AuthorizationContext> {
        let roles = self.get_user_roles(user).await?;
        let permissions = self.get_user_permissions(user).await?;
        let mut attributes = self.get_user_attributes(user).await?;

        if let (Some(res_type), Some(res_id)) = (&resource_type, resource_id) {
            let resource_attrs = self.get_resource_attributes(res_type, res_id).await?;
            attributes.extend(resource_attrs);
        }

        let environment = environment.unwrap_or_else(|| {
            let mut env = HashMap::new();
            env.insert("current_time".to_string(), Value::String(chrono::Utc::now().format("%H").to_string()));
            env.insert("current_date".to_string(), Value::String(chrono::Utc::now().format("%Y-%m-%d").to_string()));
            env
        });

        Ok(AuthorizationContext {
            user: user.clone(),
            roles,
            permissions,
            attributes,
            resource_type,
            resource_id,
            action: action.to_string(),
            environment,
        })
    }

    async fn get_applicable_policies(&self, _context: &AuthorizationContext) -> anyhow::Result<Vec<Policy>> {
        let policies = sqlx::query_as::<_, Policy>(
            r#"
            SELECT * FROM policies
            WHERE is_active = true
            ORDER BY priority DESC, created_at ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(policies)
    }

    pub async fn assign_role_to_user(&self, user_id: Ulid, role_id: Ulid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO user_roles (id, user_id, role_id, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            ON CONFLICT (user_id, role_id) DO NOTHING
            "#
        )
        .bind(Ulid::new().to_string())
        .bind(user_id.to_string())
        .bind(role_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn remove_role_from_user(&self, user_id: Ulid, role_id: Ulid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM user_roles
            WHERE user_id = $1 AND role_id = $2
            "#
        )
        .bind(user_id.to_string())
        .bind(role_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn assign_permission_to_role(&self, role_id: Ulid, permission_id: Ulid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO role_permissions (id, role_id, permission_id, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            ON CONFLICT (role_id, permission_id) DO NOTHING
            "#
        )
        .bind(Ulid::new().to_string())
        .bind(role_id.to_string())
        .bind(permission_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn remove_permission_from_role(&self, role_id: Ulid, permission_id: Ulid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            DELETE FROM role_permissions
            WHERE role_id = $1 AND permission_id = $2
            "#
        )
        .bind(role_id.to_string())
        .bind(permission_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn set_user_attribute(
        &self,
        user_id: Ulid,
        name: &str,
        value: Value,
        attribute_type: crate::app::models::attribute::AttributeType,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO attributes (id, name, attribute_type, value, subject_type, subject_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, 'user', $5, NOW(), NOW())
            ON CONFLICT (name, subject_type, subject_id) DO UPDATE SET
                value = EXCLUDED.value,
                attribute_type = EXCLUDED.attribute_type,
                updated_at = NOW()
            "#
        )
        .bind(Ulid::new().to_string())
        .bind(name)
        .bind(attribute_type.as_str())
        .bind(&value)
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn set_resource_attribute(
        &self,
        resource_type: &str,
        resource_id: Ulid,
        name: &str,
        value: Value,
        attribute_type: crate::app::models::attribute::AttributeType,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO attributes (id, name, attribute_type, value, subject_type, resource_type, resource_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, 'resource', $5, $6, NOW(), NOW())
            ON CONFLICT (name, resource_type, resource_id) DO UPDATE SET
                value = EXCLUDED.value,
                attribute_type = EXCLUDED.attribute_type,
                updated_at = NOW()
            "#
        )
        .bind(Ulid::new().to_string())
        .bind(name)
        .bind(attribute_type.as_str())
        .bind(&value)
        .bind(resource_type)
        .bind(resource_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}