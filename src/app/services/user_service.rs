use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use crate::app::models::DieselUlid;
use crate::database::{DbPool};
use crate::schema::sys_users;
use crate::app::models::user::{User, CreateUser, UpdateUser};

pub struct UserService;

impl UserService {
    pub fn create_user(pool: &DbPool, data: CreateUser, created_by: Option<DieselUlid>) -> Result<User> {
        let mut conn = pool.get()?;

        let new_user = User::to_new_user(data.name, data.email, data.password, created_by);

        let created_user = diesel::insert_into(sys_users::table)
            .values(&new_user)
            .returning(User::as_select())
            .get_result::<User>(&mut conn)?;

        Ok(created_user)
    }

    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<User>> {
        let mut conn = pool.get()?;

        let result = sys_users::table
            .filter(sys_users::id.eq(id))
            .filter(sys_users::deleted_at.is_null())
            .select(User::as_select())
            .first::<User>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_email(pool: &DbPool, email: &str) -> Result<Option<User>> {
        let mut conn = pool.get()?;

        let result = sys_users::table
            .filter(sys_users::email.eq(email))
            .filter(sys_users::deleted_at.is_null())
            .select(User::as_select())
            .first::<User>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_reset_token(pool: &DbPool, token: &str) -> Result<Option<User>> {
        let mut conn = pool.get()?;

        let result = sys_users::table
            .filter(sys_users::password_reset_token.eq(token))
            .filter(sys_users::password_reset_expires_at.gt(Utc::now()))
            .filter(sys_users::deleted_at.is_null())
            .select(User::as_select())
            .first::<User>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn update_user(pool: &DbPool, id: String, data: UpdateUser, updated_by: Option<DieselUlid>) -> Result<User> {
        let mut conn = pool.get()?;

        let result = diesel::update(sys_users::table
            .filter(sys_users::id.eq(id.to_string()))
            .filter(sys_users::deleted_at.is_null()))
            .set((
                data.name.map(|n| sys_users::name.eq(n)),
                data.email.map(|e| sys_users::email.eq(e)),
                sys_users::updated_at.eq(Utc::now()),
                sys_users::updated_by.eq(updated_by),
            ))
            .returning(User::as_select())
            .get_result::<User>(&mut conn)?;

        Ok(result)
    }

    pub fn update_password(pool: &DbPool, id: DieselUlid, new_password: String, updated_by: Option<DieselUlid>) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table
            .filter(sys_users::id.eq(id.to_string()))
            .filter(sys_users::deleted_at.is_null()))
            .set((
                sys_users::password.eq(new_password),
                sys_users::updated_at.eq(Utc::now()),
                sys_users::updated_by.eq(updated_by),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn update_last_login(pool: &DbPool, id: DieselUlid) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table
            .filter(sys_users::id.eq(id.to_string()))
            .filter(sys_users::deleted_at.is_null()))
            .set((
                sys_users::last_login_at.eq(Some(Utc::now())),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn update_failed_attempts(pool: &DbPool, id: DieselUlid, attempts: i32, locked_until: Option<DateTime<Utc>>) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table
            .filter(sys_users::id.eq(id))
            .filter(sys_users::deleted_at.is_null()))
            .set((
                sys_users::failed_login_attempts.eq(attempts),
                sys_users::locked_until.eq(locked_until),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn reset_failed_attempts(pool: &DbPool, id: DieselUlid) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table
            .filter(sys_users::id.eq(id.to_string()))
            .filter(sys_users::deleted_at.is_null()))
            .set((
                sys_users::failed_login_attempts.eq(0),
                sys_users::locked_until.eq::<Option<DateTime<Utc>>>(None),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn update_password_reset_token(pool: &DbPool, id: DieselUlid, token: Option<String>, expires_at: Option<DateTime<Utc>>) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table
            .filter(sys_users::id.eq(id.to_string()))
            .filter(sys_users::deleted_at.is_null()))
            .set((
                sys_users::password_reset_token.eq(token),
                sys_users::password_reset_expires_at.eq(expires_at),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn update_refresh_token(pool: &DbPool, id: DieselUlid, token: Option<String>, expires_at: Option<DateTime<Utc>>) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table
            .filter(sys_users::id.eq(id.to_string()))
            .filter(sys_users::deleted_at.is_null()))
            .set((
                sys_users::refresh_token.eq(token),
                sys_users::refresh_token_expires_at.eq(expires_at),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn find_by_refresh_token(pool: &DbPool, token: &str) -> Result<Option<User>> {
        let mut conn = pool.get()?;

        let result = sys_users::table
            .filter(sys_users::refresh_token.eq(token))
            .filter(sys_users::refresh_token_expires_at.gt(Utc::now()))
            .filter(sys_users::deleted_at.is_null())
            .select(User::as_select())
            .first::<User>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn soft_delete(pool: &DbPool, id: String, deleted_by: Option<DieselUlid>) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table
            .filter(sys_users::id.eq(id))
            .filter(sys_users::deleted_at.is_null()))
            .set((
                sys_users::deleted_at.eq(Some(Utc::now())),
                sys_users::deleted_by.eq(deleted_by),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn restore(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table.filter(sys_users::id.eq(id)))
            .set((
                sys_users::deleted_at.eq::<Option<DateTime<Utc>>>(None),
                sys_users::deleted_by.eq::<Option<DieselUlid>>(None),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn hard_delete(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(sys_users::table.filter(sys_users::id.eq(id)))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn find_deleted(pool: &DbPool) -> Result<Vec<User>> {
        let mut conn = pool.get()?;

        let result = sys_users::table
            .filter(sys_users::deleted_at.is_not_null())
            .select(User::as_select())
            .load::<User>(&mut conn)?;

        Ok(result)
    }
}