pub mod diesel_ulid;
pub mod user;
pub mod migration;
pub mod oauth;
pub mod country;
pub mod province;
pub mod city;
pub mod role;
pub mod permission;
pub mod organization;
pub mod organization_position_level;
pub mod organization_position;
pub mod user_organization;
pub mod notification;
pub mod sys_model_has_permission;
pub mod sys_model_has_role;

pub use diesel_ulid::DieselUlid;

/// Trait for models that can be used in polymorphic relationships
/// Provides the model type name for the sys_model_has_roles table
pub trait HasModelType {
    /// Returns the model type name used in polymorphic relationships
    fn model_type() -> &'static str;
}

/// Trait for models that can have roles assigned to them
pub trait HasRoles: HasModelType {
    /// Returns the model's unique identifier as a string
    fn model_id(&self) -> String;
}

/// Helper functions for model type management
pub mod model_types {
    use crate::app::models::user::User;
    use crate::app::models::organization::Organization;
    use crate::app::models::user_organization::UserOrganization;
    use crate::app::models::HasModelType;

    /// Returns a list of all valid model types
    pub fn valid_model_types() -> Vec<&'static str> {
        vec![
            User::model_type(),
            Organization::model_type(),
            UserOrganization::model_type(),
        ]
    }

    /// Checks if a model type string is valid
    pub fn is_valid_model_type(model_type: &str) -> bool {
        valid_model_types().contains(&model_type)
    }

    /// Model type constants for easy access
    pub const USER: &str = "User";
    pub const ORGANIZATION: &str = "Organization";
    pub const USER_ORGANIZATION: &str = "User_Organization";
}