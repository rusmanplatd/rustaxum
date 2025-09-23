use anyhow::Result;
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use crate::app::models::{DieselUlid, HasModelType};
use crate::app::models::activity_log::{ActivityLog, NewActivityLog};
use crate::database::DbPool;
use crate::schema::activity_log;

pub struct ActivityLogService {
    pool: Option<DbPool>,
}

impl ActivityLogService {
    pub fn new() -> Self {
        Self { pool: None }
    }

    pub fn with_pool(pool: DbPool) -> Self {
        Self { pool: Some(pool) }
    }

    pub async fn create(&self, new_activity: NewActivityLog) -> Result<ActivityLog> {
        let pool = self.get_pool().await?;
        let mut conn = pool.get()?;

        let activity = diesel::insert_into(activity_log::table)
            .values(&new_activity)
            .returning(ActivityLog::as_select())
            .get_result::<ActivityLog>(&mut conn)?;

        Ok(activity)
    }

    pub async fn create_batch(&self, activities: Vec<NewActivityLog>) -> Result<Vec<ActivityLog>> {
        let pool = self.get_pool().await?;
        let mut conn = pool.get()?;

        let batch_uuid = Uuid::new_v4().to_string();
        let activities_with_batch: Vec<NewActivityLog> = activities
            .into_iter()
            .map(|mut activity| {
                activity.batch_uuid = Some(batch_uuid.clone());
                activity
            })
            .collect();

        let created_activities = diesel::insert_into(activity_log::table)
            .values(&activities_with_batch)
            .returning(ActivityLog::as_select())
            .get_results::<ActivityLog>(&mut conn)?;

        Ok(created_activities)
    }

    pub async fn find_by_correlation_id(&self, correlation_id: DieselUlid) -> Result<Vec<ActivityLog>> {
        let pool = self.get_pool().await?;
        let mut conn = pool.get()?;

        let activities = activity_log::table
            .filter(activity_log::correlation_id.eq(correlation_id))
            .order(activity_log::created_at.desc())
            .select(ActivityLog::as_select())
            .load::<ActivityLog>(&mut conn)?;

        Ok(activities)
    }

    pub async fn find_by_batch_uuid(&self, batch_uuid: &str) -> Result<Vec<ActivityLog>> {
        let pool = self.get_pool().await?;
        let mut conn = pool.get()?;

        let activities = activity_log::table
            .filter(activity_log::batch_uuid.eq(batch_uuid))
            .order(activity_log::created_at.desc())
            .select(ActivityLog::as_select())
            .load::<ActivityLog>(&mut conn)?;

        Ok(activities)
    }

    pub async fn find_by_subject<T: HasModelType>(&self, subject_id: &str) -> Result<Vec<ActivityLog>> {
        let pool = self.get_pool().await?;
        let mut conn = pool.get()?;

        let activities = activity_log::table
            .filter(activity_log::subject_type.eq(T::model_type()))
            .filter(activity_log::subject_id.eq(subject_id))
            .order(activity_log::created_at.desc())
            .select(ActivityLog::as_select())
            .load::<ActivityLog>(&mut conn)?;

        Ok(activities)
    }

    pub async fn find_by_causer<T: HasModelType>(&self, causer_id: &str) -> Result<Vec<ActivityLog>> {
        let pool = self.get_pool().await?;
        let mut conn = pool.get()?;

        let activities = activity_log::table
            .filter(activity_log::causer_type.eq(T::model_type()))
            .filter(activity_log::causer_id.eq(causer_id))
            .order(activity_log::created_at.desc())
            .select(ActivityLog::as_select())
            .load::<ActivityLog>(&mut conn)?;

        Ok(activities)
    }

    pub async fn find_by_log_name(&self, log_name: &str) -> Result<Vec<ActivityLog>> {
        let pool = self.get_pool().await?;
        let mut conn = pool.get()?;

        let activities = activity_log::table
            .filter(activity_log::log_name.eq(log_name))
            .order(activity_log::created_at.desc())
            .select(ActivityLog::as_select())
            .load::<ActivityLog>(&mut conn)?;

        Ok(activities)
    }

    pub async fn find_by_event(&self, event: &str) -> Result<Vec<ActivityLog>> {
        let pool = self.get_pool().await?;
        let mut conn = pool.get()?;

        let activities = activity_log::table
            .filter(activity_log::event.eq(event))
            .order(activity_log::created_at.desc())
            .select(ActivityLog::as_select())
            .load::<ActivityLog>(&mut conn)?;

        Ok(activities)
    }

    pub async fn find_with_properties(&self, properties: Value) -> Result<Vec<ActivityLog>> {
        let pool = self.get_pool().await?;
        let mut conn = pool.get()?;

        let activities = activity_log::table
            .filter(activity_log::properties.eq(properties))
            .order(activity_log::created_at.desc())
            .select(ActivityLog::as_select())
            .load::<ActivityLog>(&mut conn)?;

        Ok(activities)
    }

    pub async fn find_in_log(&self, log_name: &str) -> ActivityLogQueryBuilder {
        ActivityLogQueryBuilder::new(self.get_pool().await.ok(), Some(log_name.to_string()))
    }

    pub async fn query(&self) -> ActivityLogQueryBuilder {
        ActivityLogQueryBuilder::new(self.get_pool().await.ok(), None)
    }

    async fn get_pool(&self) -> Result<DbPool> {
        if let Some(pool) = &self.pool {
            Ok(pool.clone())
        } else {
            let config = crate::config::Config::load()?;
            let pool = crate::database::create_pool(&config)?;
            Ok(pool)
        }
    }
}

impl Default for ActivityLogService {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ActivityLogQueryBuilder {
    pool: Option<DbPool>,
    log_name: Option<String>,
    subject_type: Option<String>,
    subject_id: Option<String>,
    causer_type: Option<String>,
    causer_id: Option<String>,
    correlation_id: Option<DieselUlid>,
    batch_uuid: Option<String>,
    event: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

impl ActivityLogQueryBuilder {
    pub fn new(pool: Option<DbPool>, log_name: Option<String>) -> Self {
        Self {
            pool,
            log_name,
            subject_type: None,
            subject_id: None,
            causer_type: None,
            causer_id: None,
            correlation_id: None,
            batch_uuid: None,
            event: None,
            limit: None,
            offset: None,
        }
    }

    pub fn log_name(mut self, log_name: &str) -> Self {
        self.log_name = Some(log_name.to_string());
        self
    }

    pub fn caused_by<T: HasModelType>(mut self, causer_id: &str) -> Self {
        self.causer_type = Some(T::model_type().to_string());
        self.causer_id = Some(causer_id.to_string());
        self
    }

    pub fn performed_on<T: HasModelType>(mut self, subject_id: &str) -> Self {
        self.subject_type = Some(T::model_type().to_string());
        self.subject_id = Some(subject_id.to_string());
        self
    }

    pub fn with_correlation_id(mut self, correlation_id: DieselUlid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn in_batch(mut self, batch_uuid: &str) -> Self {
        self.batch_uuid = Some(batch_uuid.to_string());
        self
    }

    pub fn for_event(mut self, event: &str) -> Self {
        self.event = Some(event.to_string());
        self
    }

    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub async fn get(self) -> Result<Vec<ActivityLog>> {
        let pool = self.get_pool().await?;
        let mut conn = pool.get()?;

        let mut query = activity_log::table.into_boxed();

        if let Some(log_name) = &self.log_name {
            query = query.filter(activity_log::log_name.eq(log_name));
        }

        if let Some(subject_type) = &self.subject_type {
            query = query.filter(activity_log::subject_type.eq(subject_type));
        }

        if let Some(subject_id) = &self.subject_id {
            query = query.filter(activity_log::subject_id.eq(subject_id));
        }

        if let Some(causer_type) = &self.causer_type {
            query = query.filter(activity_log::causer_type.eq(causer_type));
        }

        if let Some(causer_id) = &self.causer_id {
            query = query.filter(activity_log::causer_id.eq(causer_id));
        }

        if let Some(correlation_id) = &self.correlation_id {
            query = query.filter(activity_log::correlation_id.eq(correlation_id));
        }

        if let Some(batch_uuid) = &self.batch_uuid {
            query = query.filter(activity_log::batch_uuid.eq(batch_uuid));
        }

        if let Some(event) = &self.event {
            query = query.filter(activity_log::event.eq(event));
        }

        query = query.order(activity_log::created_at.desc());

        if let Some(limit) = self.limit {
            query = query.limit(limit);
        }

        if let Some(offset) = self.offset {
            query = query.offset(offset);
        }

        let activities = query
            .select(ActivityLog::as_select())
            .load::<ActivityLog>(&mut conn)?;

        Ok(activities)
    }

    pub async fn first(self) -> Result<Option<ActivityLog>> {
        let activities = self.limit(1).get().await?;
        Ok(activities.into_iter().next())
    }

    pub async fn count(self) -> Result<i64> {
        let pool = self.get_pool().await?;
        let mut conn = pool.get()?;

        let mut query = activity_log::table.into_boxed();

        if let Some(log_name) = &self.log_name {
            query = query.filter(activity_log::log_name.eq(log_name));
        }

        if let Some(subject_type) = &self.subject_type {
            query = query.filter(activity_log::subject_type.eq(subject_type));
        }

        if let Some(subject_id) = &self.subject_id {
            query = query.filter(activity_log::subject_id.eq(subject_id));
        }

        if let Some(causer_type) = &self.causer_type {
            query = query.filter(activity_log::causer_type.eq(causer_type));
        }

        if let Some(causer_id) = &self.causer_id {
            query = query.filter(activity_log::causer_id.eq(causer_id));
        }

        if let Some(correlation_id) = &self.correlation_id {
            query = query.filter(activity_log::correlation_id.eq(correlation_id));
        }

        if let Some(batch_uuid) = &self.batch_uuid {
            query = query.filter(activity_log::batch_uuid.eq(batch_uuid));
        }

        if let Some(event) = &self.event {
            query = query.filter(activity_log::event.eq(event));
        }

        let total = query.count().get_result::<i64>(&mut conn)?;

        Ok(total)
    }

    async fn get_pool(&self) -> Result<DbPool> {
        if let Some(pool) = &self.pool {
            Ok(pool.clone())
        } else {
            let config = crate::config::Config::load()?;
            let pool = crate::database::create_pool(&config)?;
            Ok(pool)
        }
    }
}

// Helper functions for quick logging
pub async fn activity(description: &str) -> Result<ActivityLog> {
    ActivityLog::builder()
        .description(description)
        .log()
        .await
}

pub async fn activity_for_log(log_name: &str, description: &str) -> Result<ActivityLog> {
    ActivityLog::for_log(log_name)
        .description(description)
        .log()
        .await
}

pub async fn activity_with_correlation(
    description: &str,
    correlation_id: DieselUlid
) -> Result<ActivityLog> {
    ActivityLog::with_correlation_id(correlation_id)
        .description(description)
        .log()
        .await
}