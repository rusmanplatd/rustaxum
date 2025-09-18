# Policy System - RBAC & ABAC Implementation

This directory contains a comprehensive authorization system that supports both Role-Based Access Control (RBAC) and Attribute-Based Access Control (ABAC) patterns.

## Overview

The policy system provides:

- **RBAC (Role-Based Access Control)**: Traditional role and permission-based authorization
- **ABAC (Attribute-Based Access Control)**: Fine-grained authorization based on attributes, context, and policies
- **Combined RBAC + ABAC**: Hybrid approach for maximum flexibility
- **Policy Evaluation Engine**: Dynamic policy condition evaluation
- **Context-Aware Authorization**: Time, location, and environment-based constraints

## Core Components

### 1. Policy Trait (`policy_trait.rs`)

The main trait that defines the authorization interface:

```rust
#[async_trait]
pub trait PolicyTrait: Send + Sync {
    async fn authorize(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult>;
    async fn authorize_rbac(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult>;
    async fn authorize_abac(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult>;
    // ... other methods
}
```

### 2. Base Policy (`base_policy.rs`)

A comprehensive implementation of the PolicyTrait that combines RBAC and ABAC:

- Role and permission checking
- Policy condition evaluation
- Resource ownership validation
- Time and location constraints

### 3. Policy Service (`policy_service.rs`)

High-level service for policy management and evaluation:

- User authorization with database integration
- Role and permission management
- Attribute management
- Policy evaluation with context building

### 4. Example Implementation (`post_policy.rs`)

A concrete example showing how to implement policies for a Post resource:

- View, create, update, delete operations
- Publishing and moderation workflows
- Department-based editing
- Time-constrained operations

## Key Features

### RBAC Features

- **Role Management**: Assign roles to users with hierarchical permissions
- **Permission System**: Fine-grained permissions with resource and action scoping
- **Resource Ownership**: Automatic ownership-based access control
- **Admin Overrides**: Administrative roles with elevated privileges

### ABAC Features

- **Attribute-Based Access**: Decision making based on user, resource, and environment attributes
- **Policy Conditions**: Complex condition evaluation with logical operators
- **Time Constraints**: Business hours and time-based restrictions
- **Location Restrictions**: Geographic or network-based access control
- **Multi-Factor Authorization**: Combining multiple authorization factors

### Policy Conditions

The system supports rich policy condition syntax:

```
user.role == "admin" AND time.hour in ["9", "10", "11", "12", "13", "14", "15", "16", "17"]
user.department == resource.department
resource.owner == user.id
```

## Usage Examples

### Basic RBAC Usage

```rust
use crate::app::policies::{policy_service::PolicyService, post_policy::PostPolicy};

let policy_service = PolicyService::new(pool);

// Check if user can perform action
let can_edit = policy_service.can_user(&user, "edit", Some("post".to_string()), Some(post_id)).await?;

// Check specific permission
let has_permission = policy_service.user_has_permission(&user, "create:post", Some("post".to_string())).await?;
```

### Advanced ABAC Usage

```rust
let mut environment = HashMap::new();
environment.insert("current_hour".to_string(), json!("14"));
environment.insert("location".to_string(), json!("office"));

let result = policy_service.authorize_user(
    &user,
    "publish",
    Some("post".to_string()),
    Some(post_id),
    Some(environment)
).await?;

if result.allowed {
    // Proceed with action
    println!("Authorized: {}", result.reason);
} else {
    // Deny access
    println!("Access denied: {}", result.reason);
}
```

### Custom Policy Implementation

```rust
use crate::app::policies::policy_trait::{PolicyTrait, AuthorizationContext, AuthorizationResult};

#[derive(Debug)]
pub struct CustomPolicy {
    base: BasePolicy,
}

#[async_trait]
impl PolicyTrait for CustomPolicy {
    async fn authorize(&self, context: &AuthorizationContext) -> anyhow::Result<AuthorizationResult> {
        // Custom authorization logic
        if self.check_custom_conditions(context).await? {
            Ok(AuthorizationResult::allow("Custom condition met".to_string()))
        } else {
            Ok(AuthorizationResult::deny("Custom condition not met".to_string()))
        }
    }

    // Implement other required methods...
}
```

## Database Schema

The policy system works with the following database tables:

- `roles` - Role definitions
- `permissions` - Permission definitions
- `user_roles` - User-role assignments
- `role_permissions` - Role-permission assignments
- `policies` - ABAC policy definitions
- `attributes` - User and resource attributes
- `subjects` - Policy subjects
- `resources` - Policy resources

## Running Examples

To see the policy system in action:

```bash
cargo run --bin policy_demo
```

This will run comprehensive examples demonstrating:

- RBAC with roles and permissions
- ABAC with attributes and conditions
- Time and location-based constraints
- Combined RBAC + ABAC scenarios
- Multi-factor authorization

## Testing

The policy implementations include comprehensive test suites:

```bash
cargo test policies
```

Tests cover:

- Role-based authorization scenarios
- Attribute-based policy evaluation
- Time and location constraints
- Ownership-based access control
- Department-based access
- Error conditions and edge cases

## Architecture Benefits

1. **Flexibility**: Support for both simple RBAC and complex ABAC scenarios
2. **Performance**: Efficient evaluation with minimal database queries
3. **Extensibility**: Easy to add new policy types and conditions
4. **Maintainability**: Clear separation between RBAC and ABAC logic
5. **Security**: Comprehensive validation and safe defaults (deny-by-default)
6. **Auditability**: Detailed authorization results with applied policies

## Best Practices

1. **Start Simple**: Begin with RBAC, add ABAC for complex scenarios
2. **Deny by Default**: Explicit permits are required for access
3. **Least Privilege**: Grant minimal required permissions
4. **Regular Audits**: Review roles, permissions, and policies regularly
5. **Test Coverage**: Maintain comprehensive test suites for authorization logic
6. **Performance**: Monitor policy evaluation performance in production
7. **Documentation**: Document custom policies and business rules clearly

This implementation provides a robust foundation for handling complex authorization requirements while maintaining simplicity for common use cases.