// This file will be automatically updated when new seeders are generated
pub mod user_seeder;
pub mod database_seeder;
pub mod country_seeder;
pub mod province_seeder;
pub mod city_seeder;
// pub mod testseeder;
pub mod role_permission_seeder;
pub mod organization_seeder;
pub mod oauthscopeseeder;
pub use oauthscopeseeder::OAuthScopeSeeder;
pub mod organizationpositionlevelseeder;
pub use organizationpositionlevelseeder::OrganizationPositionLevelSeeder;
pub mod organizationpositionseeder;
pub use organizationpositionseeder::OrganizationPositionSeeder;
pub mod userorganizationseeder;
pub use userorganizationseeder::UserOrganizationSeeder;