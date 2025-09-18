use anyhow::Result;
use ulid::Ulid;
use sqlx::PgPool;
use crate::app::models::role::{Role, CreateRole, UpdateRole};

pub struct RoleService;

impl RoleService {
    pub async fn create(_pool: &PgPool, _data: CreateRole) -> Result<Role> {
        todo!("Implement after migrations")
    }

    pub async fn find_by_id(_pool: &PgPool, _id: Ulid) -> Result<Option<Role>> {
        todo!("Implement after migrations")
    }

    pub async fn find_by_name(_pool: &PgPool, _name: &str, _guard_name: Option<&str>) -> Result<Option<Role>> {
        todo!("Implement after migrations")
    }

    pub async fn update(_pool: &PgPool, _id: Ulid, _data: UpdateRole) -> Result<Role> {
        todo!("Implement after migrations")
    }

    pub async fn delete(_pool: &PgPool, _id: Ulid) -> Result<()> {
        todo!("Implement after migrations")
    }

    pub async fn list(_pool: &PgPool, _limit: i64, _offset: i64) -> Result<Vec<Role>> {
        todo!("Implement after migrations")
    }

    pub async fn assign_to_user(_pool: &PgPool, _user_id: Ulid, _role_id: Ulid) -> Result<()> {
        todo!("Implement after migrations")
    }

    pub async fn remove_from_user(_pool: &PgPool, _user_id: Ulid, _role_id: Ulid) -> Result<()> {
        todo!("Implement after migrations")
    }

    pub async fn user_has_role(_pool: &PgPool, _user_id: Ulid, _role_name: &str, _guard_name: Option<&str>) -> Result<bool> {
        todo!("Implement after migrations")
    }

    pub async fn get_user_roles(_pool: &PgPool, _user_id: Ulid, _guard_name: Option<&str>) -> Result<Vec<Role>> {
        todo!("Implement after migrations")
    }
}