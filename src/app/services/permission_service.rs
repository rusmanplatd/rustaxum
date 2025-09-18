use anyhow::Result;
use ulid::Ulid;
use sqlx::PgPool;
use crate::app::models::permission::{Permission, CreatePermission, UpdatePermission};

pub struct PermissionService;

impl PermissionService {
    pub async fn create(_pool: &PgPool, _data: CreatePermission) -> Result<Permission> {
        todo!("Implement after migrations")
    }

    pub async fn find_by_id(_pool: &PgPool, _id: Ulid) -> Result<Option<Permission>> {
        todo!("Implement after migrations")
    }

    pub async fn find_by_name(_pool: &PgPool, _name: &str, _guard_name: Option<&str>) -> Result<Option<Permission>> {
        todo!("Implement after migrations")
    }

    pub async fn update(_pool: &PgPool, _id: Ulid, _data: UpdatePermission) -> Result<Permission> {
        todo!("Implement after migrations")
    }

    pub async fn delete(_pool: &PgPool, _id: Ulid) -> Result<()> {
        todo!("Implement after migrations")
    }

    pub async fn list(_pool: &PgPool, _limit: i64, _offset: i64) -> Result<Vec<Permission>> {
        todo!("Implement after migrations")
    }

    pub async fn assign_to_role(_pool: &PgPool, _role_id: Ulid, _permission_id: Ulid) -> Result<()> {
        todo!("Implement after migrations")
    }

    pub async fn remove_from_role(_pool: &PgPool, _role_id: Ulid, _permission_id: Ulid) -> Result<()> {
        todo!("Implement after migrations")
    }

    pub async fn role_has_permission(_pool: &PgPool, _role_id: Ulid, _permission_name: &str, _guard_name: Option<&str>) -> Result<bool> {
        todo!("Implement after migrations")
    }

    pub async fn user_has_permission(_pool: &PgPool, _user_id: Ulid, _permission_name: &str, _guard_name: Option<&str>) -> Result<bool> {
        todo!("Implement after migrations")
    }

    pub async fn get_role_permissions(_pool: &PgPool, _role_id: Ulid, _guard_name: Option<&str>) -> Result<Vec<Permission>> {
        todo!("Implement after migrations")
    }

    pub async fn get_user_permissions(_pool: &PgPool, _user_id: Ulid, _guard_name: Option<&str>) -> Result<Vec<Permission>> {
        todo!("Implement after migrations")
    }
}