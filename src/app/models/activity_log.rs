use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::app::models::{DieselUlid, HasModelType};

#[derive(Queryable, Selectable, Insertable, Serialize, Deserialize, Debug, Clone, ToSchema)]
#[diesel(table_name = crate::schema::activity_log)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ActivityLog {
    pub id: DieselUlid,
    pub log_name: Option<String>,
    pub description: String,
    pub subject_type: Option<String>,
    pub subject_id: Option<String>,
    pub causer_type: Option<String>,
    pub causer_id: Option<String>,
    pub properties: Option<Value>,
    pub correlation_id: Option<DieselUlid>,
    pub batch_uuid: Option<String>,
    pub event: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl HasModelType for ActivityLog {
    fn model_type() -> &'static str {
        "ActivityLog"
    }
}

impl crate::app::query_builder::Queryable for ActivityLog {
    fn table_name() -> &'static str {
        "activity_log"
    }

    fn allowed_filters() -> Vec<&'static str> {
        vec![
            "id",
            "log_name",
            "description",
            "subject_type",
            "subject_id",
            "causer_type",
            "causer_id",
            "correlation_id",
            "batch_uuid",
            "event",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_sorts() -> Vec<&'static str> {
        vec![
            "id",
            "log_name",
            "description",
            "subject_type",
            "subject_id",
            "causer_type",
            "causer_id",
            "event",
            "created_at",
            "updated_at",
        ]
    }

    fn allowed_fields() -> Vec<&'static str> {
        vec![
            "id",
            "log_name",
            "description",
            "subject_type",
            "subject_id",
            "causer_type",
            "causer_id",
            "properties",
            "correlation_id",
            "batch_uuid",
            "event",
            "created_at",
            "updated_at",
        ]
    }

    fn default_sort() -> Option<(&'static str, crate::app::query_builder::SortDirection)> {
        Some(("created_at", crate::app::query_builder::SortDirection::Desc))
    }
}

// Implement the query builder service for ActivityLog
crate::impl_query_builder_service!(ActivityLog);

impl ActivityLog {
    pub fn builder() -> ActivityLogBuilder {
        ActivityLogBuilder::new()
    }

    pub fn for_log(log_name: &str) -> ActivityLogBuilder {
        ActivityLogBuilder::new().log_name(log_name)
    }

    pub fn caused_by<T: HasModelType + ?Sized>(causer: &T) -> ActivityLogBuilder
    where
        T: HasId,
    {
        ActivityLogBuilder::new()
            .causer_type(T::model_type())
            .causer_id(&causer.id())
    }

    pub fn performed_on<T: HasModelType + ?Sized>(subject: &T) -> ActivityLogBuilder
    where
        T: HasId,
    {
        ActivityLogBuilder::new()
            .subject_type(T::model_type())
            .subject_id(&subject.id())
    }

    pub fn with_correlation_id(correlation_id: DieselUlid) -> ActivityLogBuilder {
        ActivityLogBuilder::new().correlation_id(correlation_id)
    }

    pub fn in_batch(batch_uuid: &str) -> ActivityLogBuilder {
        ActivityLogBuilder::new().batch_uuid(batch_uuid)
    }
}

pub trait HasId {
    fn id(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct ActivityLogBuilder {
    log_name: Option<String>,
    description: Option<String>,
    subject_type: Option<String>,
    subject_id: Option<String>,
    causer_type: Option<String>,
    causer_id: Option<String>,
    properties: Option<Value>,
    correlation_id: Option<DieselUlid>,
    batch_uuid: Option<String>,
    event: Option<String>,
}

impl ActivityLogBuilder {
    pub fn new() -> Self {
        Self {
            log_name: None,
            description: None,
            subject_type: None,
            subject_id: None,
            causer_type: None,
            causer_id: None,
            properties: None,
            correlation_id: None,
            batch_uuid: None,
            event: None,
        }
    }

    pub fn log_name(mut self, log_name: &str) -> Self {
        self.log_name = Some(log_name.to_string());
        self
    }

    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn subject_type(mut self, subject_type: &str) -> Self {
        self.subject_type = Some(subject_type.to_string());
        self
    }

    pub fn subject_id(mut self, subject_id: &str) -> Self {
        self.subject_id = Some(subject_id.to_string());
        self
    }

    pub fn causer_type(mut self, causer_type: &str) -> Self {
        self.causer_type = Some(causer_type.to_string());
        self
    }

    pub fn causer_id(mut self, causer_id: &str) -> Self {
        self.causer_id = Some(causer_id.to_string());
        self
    }

    pub fn with_properties(mut self, properties: Value) -> Self {
        self.properties = Some(properties);
        self
    }

    pub fn with_property<T: Serialize>(mut self, key: &str, value: T) -> Self {
        let mut props = self.properties.unwrap_or(Value::Object(serde_json::Map::new()));
        if let Value::Object(ref mut map) = props {
            map.insert(key.to_string(), serde_json::to_value(value).unwrap_or(Value::Null));
        }
        self.properties = Some(props);
        self
    }

    pub fn correlation_id(mut self, correlation_id: DieselUlid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn batch_uuid(mut self, batch_uuid: &str) -> Self {
        self.batch_uuid = Some(batch_uuid.to_string());
        self
    }

    pub fn event(mut self, event: &str) -> Self {
        self.event = Some(event.to_string());
        self
    }

    pub fn performed_on<T: HasModelType + HasId + ?Sized>(mut self, subject: &T) -> Self {
        self.subject_type = Some(T::model_type().to_string());
        self.subject_id = Some(subject.id());
        self
    }

    pub fn caused_by<T: HasModelType + HasId + ?Sized>(mut self, causer: &T) -> Self {
        self.causer_type = Some(T::model_type().to_string());
        self.causer_id = Some(causer.id());
        self
    }

    pub fn build(self) -> Result<ActivityLog, &'static str> {
        let description = self.description.ok_or("Description is required")?;
        let now = chrono::Utc::now();

        Ok(ActivityLog {
            id: DieselUlid::new(),
            log_name: self.log_name,
            description,
            subject_type: self.subject_type,
            subject_id: self.subject_id,
            causer_type: self.causer_type,
            causer_id: self.causer_id,
            properties: self.properties,
            correlation_id: self.correlation_id,
            batch_uuid: self.batch_uuid,
            event: self.event,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn log(self) -> Result<ActivityLog, anyhow::Error> {
        let activity_service = crate::app::services::activity_log_service::ActivityLogService::new();
        activity_service.create(self.build().map_err(anyhow::Error::msg)?).await
    }
}

impl Default for ActivityLogBuilder {
    fn default() -> Self {
        Self::new()
    }
}