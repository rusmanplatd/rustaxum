pub mod diesel_ulid;
pub mod decimal_wrapper;
pub mod user;
pub mod migration;
pub mod oauth;
pub mod country;
pub mod province;
pub mod city;
pub mod district;
pub mod village;
pub mod role;
pub mod permission;
pub mod organization;
pub mod organization_position_level;
pub mod organization_position;
pub mod user_organization;
pub mod notification;
pub mod sys_model_has_permission;
pub mod sys_model_has_role;
pub mod activity_log;
pub mod session;
pub mod conversation;
pub mod message;
pub mod message_delivery_status;
pub mod message_reactions;
pub mod device;
pub mod device_capabilities;
pub mod conversation_participant;
pub mod prekey_bundle;
pub mod signal_session;
pub mod polls;
pub mod poll_votes;
pub mod jobs;
pub mod events;
pub mod security_incidents;
pub mod typing_indicators;
pub mod pinned_messages;
pub mod scheduled_messages;
pub mod message_mentions;
pub mod device_presence;
pub mod device_fingerprints;
pub mod oauth_pushed_requests;
pub mod oauth_ciba_requests;
pub mod oauth_ciba_auth_codes;
pub mod encrypted_backup_keys;
pub mod algorithm_compatibility_matrix;
pub mod forward_history;
pub mod message_device_keys;
pub mod device_push_tokens;
pub mod device_session_backups;

pub use diesel_ulid::DieselUlid;
pub use decimal_wrapper::DecimalWrapper;

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
    use crate::app::models::role::Role;
    use crate::app::models::HasModelType;

    /// Returns a list of all valid model types
    pub fn valid_model_types() -> Vec<&'static str> {
        vec![
            Role::model_type(),
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
    pub const ROLE: &str = "Role";
    pub const USER: &str = "User";
    pub const ORGANIZATION: &str = "Organization";
    pub const USER_ORGANIZATION: &str = "User_Organization";
}
// pub mod mfamethod;