use anyhow::Result;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use crate::app::models::DieselUlid;
use crate::database::{DbPool};
use crate::schema::sys_users;
use crate::app::models::user::{User, CreateUser, UpdateUser};

pub struct UserService;

impl UserService {
    pub fn create_user(_pool: &DbPool, data: CreateUser) -> Result<User> {
        let user = User::new(data.name, data.email, data.password);
        Ok(user)
    }

    pub fn create_user_record(pool: &DbPool, user: User) -> Result<User> {
        let mut conn = pool.get()?;

        let new_user = diesel::insert_into(sys_users::table)
            .values((
                sys_users::id.eq(user.id),
                sys_users::name.eq(&user.name),
                sys_users::email.eq(&user.email),
                sys_users::password.eq(&user.password),
                sys_users::created_at.eq(user.created_at),
                sys_users::updated_at.eq(user.updated_at),
            ))
            .returning(User::as_select())
            .get_result::<User>(&mut conn)?;

        Ok(new_user)
    }

    pub fn find_by_id(pool: &DbPool, id: String) -> Result<Option<User>> {
        let mut conn = pool.get()?;

        let result = sys_users::table
            .filter(sys_users::id.eq(id))
            .select(User::as_select())
            .first::<User>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_email(pool: &DbPool, email: &str) -> Result<Option<User>> {
        let mut conn = pool.get()?;

        let result = sys_users::table
            .filter(sys_users::email.eq(email))
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
            .select(User::as_select())
            .first::<User>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn update_user(pool: &DbPool, id: String, data: UpdateUser) -> Result<User> {
        let mut conn = pool.get()?;

        let result = diesel::update(sys_users::table.filter(sys_users::id.eq(id.to_string())))
            .set((
                data.name.map(|n| sys_users::name.eq(n)),
                data.email.map(|e| sys_users::email.eq(e)),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .returning(User::as_select())
            .get_result::<User>(&mut conn)?;

        Ok(result)
    }

    pub fn update_password(pool: &DbPool, id: DieselUlid, new_password: String) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table.filter(sys_users::id.eq(id.to_string())))
            .set((
                sys_users::password.eq(new_password),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn update_last_login(pool: &DbPool, id: DieselUlid) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table.filter(sys_users::id.eq(id.to_string())))
            .set((
                sys_users::last_login_at.eq(Some(Utc::now())),
                sys_users::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)?;

        Ok(())
    }

    pub fn update_failed_attempts(pool: &DbPool, id: String, attempts: i32, locked_until: Option<DateTime<Utc>>) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::update(sys_users::table.filter(sys_users::id.eq(id.to_string())))
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

        diesel::update(sys_users::table.filter(sys_users::id.eq(id.to_string())))
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

        diesel::update(sys_users::table.filter(sys_users::id.eq(id.to_string())))
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

        diesel::update(sys_users::table.filter(sys_users::id.eq(id.to_string())))
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
            .select(User::as_select())
            .first::<User>(&mut conn)
            .optional()?;

        Ok(result)
    }

    pub fn delete_user(pool: &DbPool, id: String) -> Result<()> {
        let mut conn = pool.get()?;

        diesel::delete(sys_users::table.filter(sys_users::id.eq(id.to_string())))
            .execute(&mut conn)?;

        Ok(())
    }
}