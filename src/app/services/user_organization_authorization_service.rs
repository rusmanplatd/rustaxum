use std::collections::HashMap;
use anyhow::Result;
use sqlx::PgPool;
use ulid::Ulid;
use serde_json::{json, Value};

use crate::app::models::{
    user::User,
    userorganization::UserOrganization,
    organization::Organization,
    role::Role,
    permission::Permission,
};
use crate::app::policies::{
    user_organization_policy::UserOrganizationPolicy,
    policy_trait::{PolicyTrait, AuthorizationContext, AuthorizationResult},
};

#[derive(Debug)]
pub struct UserOrganizationAuthorizationService {
    policy: UserOrganizationPolicy,
    pool: PgPool,
}

impl UserOrganizationAuthorizationService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            policy: UserOrganizationPolicy::new(),
            pool,
        }
    }

    /// Authorize a user action on a UserOrganization resource
    pub async fn authorize(
        &self,
        user: &User,
        action: &str,
        user_org: Option<&UserOrganization>,
        organization: Option<&Organization>,
    ) -> Result<AuthorizationResult> {
        // Get user roles and permissions (simplified for now)
        let roles = self.get_user_roles(user.id).await?;
        let permissions = self.get_user_permissions(user.id).await?;

        // Create authorization context
        let context = self.policy.create_context(
            user.clone(),
            roles,
            permissions,
            user_org,
            organization,
            action.to_string(),
        ).await;

        // Perform authorization
        self.policy.authorize(&context).await
    }

    /// Check if user can view any user organization relationships
    pub async fn can_view_any(&self, user: &User) -> Result<bool> {
        let result = self.authorize(user, "view_any", None, None).await?;
        Ok(result.allowed)
    }

    /// Check if user can view specific user organization relationship
    pub async fn can_view(&self, user: &User, user_org: &UserOrganization) -> Result<bool> {
        let organization = self.get_organization(user_org.organization_id).await.ok();
        let result = self.authorize(user, "view", Some(user_org), organization.as_ref()).await?;
        Ok(result.allowed)
    }

    /// Check if user can create user organization relationships
    pub async fn can_create(&self, user: &User, organization: Option<&Organization>) -> Result<bool> {
        let result = self.authorize(user, "create", None, organization).await?;
        Ok(result.allowed)
    }

    /// Check if user can update user organization relationships
    pub async fn can_update(&self, user: &User, user_org: &UserOrganization) -> Result<bool> {
        let organization = self.get_organization(user_org.organization_id).await.ok();
        let result = self.authorize(user, "update", Some(user_org), organization.as_ref()).await?;
        Ok(result.allowed)
    }

    /// Check if user can delete user organization relationships
    pub async fn can_delete(&self, user: &User, user_org: &UserOrganization) -> Result<bool> {
        let organization = self.get_organization(user_org.organization_id).await.ok();
        let result = self.authorize(user, "delete", Some(user_org), organization.as_ref()).await?;
        Ok(result.allowed)
    }

    /// Check if user can transfer someone to different organization
    pub async fn can_transfer(&self, user: &User, user_org: &UserOrganization) -> Result<bool> {
        let organization = self.get_organization(user_org.organization_id).await.ok();
        let result = self.authorize(user, "transfer", Some(user_org), organization.as_ref()).await?;
        Ok(result.allowed)
    }

    /// Check if user can activate/deactivate user organization relationships
    pub async fn can_activate_deactivate(&self, user: &User, user_org: &UserOrganization) -> Result<bool> {
        let organization = self.get_organization(user_org.organization_id).await.ok();
        let result = self.authorize(user, "activate", Some(user_org), organization.as_ref()).await?;
        Ok(result.allowed)
    }

    /// Check role-based permissions for specific organization
    pub async fn check_role_in_organization(
        &self,
        user_id: Ulid,
        organization_id: Ulid,
        role_name: &str,
    ) -> Result<bool> {
        // Simplified check - in a real implementation, you'd query the user_organization_roles table
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM roles r
            INNER JOIN user_organization_roles uor ON r.id = uor.role_id
            INNER JOIN user_organizations uo ON uor.user_organization_id = uo.id
            WHERE uo.user_id = $1 AND uo.organization_id = $2 AND r.name = $3 AND uo.is_active = true
            "#
        )
        .bind(user_id.to_string())
        .bind(organization_id.to_string())
        .bind(role_name)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        Ok(count > 0)
    }

    /// Check permission-based access for specific organization
    pub async fn check_permission_in_organization(
        &self,
        user_id: Ulid,
        organization_id: Ulid,
        permission_name: &str,
    ) -> Result<bool> {
        // Simplified check - in a real implementation, you'd query through role_permissions
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM permissions p
            INNER JOIN role_permissions rp ON p.id = rp.permission_id
            INNER JOIN roles r ON rp.role_id = r.id
            INNER JOIN user_organization_roles uor ON r.id = uor.role_id
            INNER JOIN user_organizations uo ON uor.user_organization_id = uo.id
            WHERE uo.user_id = $1 AND uo.organization_id = $2 AND p.name = $3 AND uo.is_active = true
            "#
        )
        .bind(user_id.to_string())
        .bind(organization_id.to_string())
        .bind(permission_name)
        .fetch_one(&self.pool)
        .await
        .unwrap_or(0);

        Ok(count > 0)
    }

    /// Get user's hierarchical access level in organization
    pub async fn get_user_organization_level(&self, _user_id: Ulid, organization_id: Ulid) -> Result<Option<u8>> {
        // Get the organization type to determine hierarchy level
        let org_type: Option<String> = sqlx::query_scalar(
            "SELECT type FROM organizations WHERE id = $1"
        )
        .bind(organization_id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(org_type) = org_type {
            let level = match org_type.as_str() {
                "holding" => 1,
                "subsidiary" => 2,
                "boc" | "bod" => 3,
                "division" => 4,
                "department" => 5,
                "branch" => 6,
                "subbranch" => 7,
                "section" => 8,
                _ => 0,
            };
            Ok(Some(level))
        } else {
            Ok(None)
        }
    }

    /// Check if user can access target organization based on hierarchy
    pub async fn can_access_organization_hierarchy(
        &self,
        _user_id: Ulid,
        user_organization_id: Ulid,
        target_organization_id: Ulid,
        access_type: HierarchyAccessType,
    ) -> Result<bool> {
        match access_type {
            HierarchyAccessType::Same => {
                Ok(user_organization_id == target_organization_id)
            }
            HierarchyAccessType::Subordinate => {
                // Check if target is under user's organization in hierarchy
                let count: i64 = sqlx::query_scalar(
                    r#"
                    WITH RECURSIVE org_hierarchy AS (
                        SELECT id, parent_id FROM organizations WHERE id = $1
                        UNION ALL
                        SELECT o.id, o.parent_id
                        FROM organizations o
                        INNER JOIN org_hierarchy oh ON o.parent_id = oh.id
                    )
                    SELECT COUNT(*) FROM org_hierarchy WHERE id = $2
                    "#
                )
                .bind(user_organization_id.to_string())
                .bind(target_organization_id.to_string())
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

                Ok(count > 0)
            }
            HierarchyAccessType::Superior => {
                // Check if target is above user's organization in hierarchy
                let count: i64 = sqlx::query_scalar(
                    r#"
                    WITH RECURSIVE parent_hierarchy AS (
                        SELECT id, parent_id FROM organizations WHERE id = $1
                        UNION ALL
                        SELECT o.id, o.parent_id
                        FROM organizations o
                        INNER JOIN parent_hierarchy ph ON ph.parent_id = o.id
                    )
                    SELECT COUNT(*) FROM parent_hierarchy WHERE id = $2
                    "#
                )
                .bind(user_organization_id.to_string())
                .bind(target_organization_id.to_string())
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);

                Ok(count > 0)
            }
        }
    }

    /// Assign role to user in organization
    pub async fn assign_role(
        &self,
        user_organization_id: Ulid,
        role_id: Ulid,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO user_organization_roles (id, user_organization_id, role_id, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            ON CONFLICT (user_organization_id, role_id) DO NOTHING
            "#
        )
        .bind(Ulid::new().to_string())
        .bind(user_organization_id.to_string())
        .bind(role_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Remove role from user in organization
    pub async fn remove_role(
        &self,
        user_organization_id: Ulid,
        role_id: Ulid,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM user_organization_roles WHERE user_organization_id = $1 AND role_id = $2"
        )
        .bind(user_organization_id.to_string())
        .bind(role_id.to_string())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Helper methods

    async fn get_user_roles(&self, _user_id: Ulid) -> Result<Vec<Role>> {
        // Simplified - return empty vec for now
        // In a real implementation, you'd query user's global roles and organization-specific roles
        Ok(vec![])
    }

    async fn get_user_permissions(&self, _user_id: Ulid) -> Result<Vec<Permission>> {
        // Simplified - return empty vec for now
        // In a real implementation, you'd query user's permissions through roles
        Ok(vec![])
    }

    async fn get_organization(&self, organization_id: Ulid) -> Result<Organization> {
        let org = sqlx::query_as::<_, Organization>(
            "SELECT id, name, type, parent_id, code, description, is_active, created_at, updated_at FROM organizations WHERE id = $1"
        )
        .bind(organization_id.to_string())
        .fetch_one(&self.pool)
        .await?;

        Ok(org)
    }
}

#[derive(Debug, Clone)]
pub enum HierarchyAccessType {
    Same,        // Same organization level
    Subordinate, // Organizations below in hierarchy
    Superior,    // Organizations above in hierarchy
}

/// ABAC attribute builder for UserOrganization authorization
#[derive(Debug)]
pub struct UserOrganizationAttributeBuilder {
    attributes: HashMap<String, Value>,
}

impl UserOrganizationAttributeBuilder {
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }

    pub fn with_user_organization(mut self, user_org: &UserOrganization) -> Self {
        self.attributes.extend(user_org.get_abac_attributes());
        self
    }

    pub fn with_organization_hierarchy(mut self, level: u8, parent_count: u8) -> Self {
        self.attributes.insert("organization_level".to_string(), json!(level));
        self.attributes.insert("hierarchy_depth".to_string(), json!(parent_count));
        self
    }

    pub fn with_temporal_constraints(mut self, business_hours_only: bool, exclude_weekends: bool) -> Self {
        self.attributes.insert("business_hours_only".to_string(), json!(business_hours_only));
        self.attributes.insert("exclude_weekends".to_string(), json!(exclude_weekends));
        self
    }

    pub fn with_custom_attribute(mut self, key: String, value: Value) -> Self {
        self.attributes.insert(key, value);
        self
    }

    pub fn build(self) -> HashMap<String, Value> {
        self.attributes
    }
}

impl Default for UserOrganizationAttributeBuilder {
    fn default() -> Self {
        Self::new()
    }
}