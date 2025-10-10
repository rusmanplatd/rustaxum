use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::schema::prekey_bundles;
use super::DieselUlid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = prekey_bundles)]
#[diesel(primary_key(id))]
pub struct PrekeyBundle {
    pub id: DieselUlid,
    pub device_id: DieselUlid,
    pub user_id: DieselUlid,
    pub prekey_id: i32,
    pub prekey_public: String,
    pub is_used: bool,
    pub used_at: Option<DateTime<Utc>>,
    pub used_by_user_id: Option<DieselUlid>,
    pub used_by_device_id: Option<DieselUlid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl PrekeyBundle {
    pub fn is_available(&self) -> bool {
        !self.is_used
    }
}

impl PrekeyBundle {
    pub fn new(
        device_id: DieselUlid,
        user_id: DieselUlid,
        prekey_id: i32,
        prekey_public: String,
    ) -> Self {
        let now = chrono::Utc::now();
        PrekeyBundle {
            id: DieselUlid::new(),
            device_id,
            user_id,
            prekey_id,
            prekey_public,
            is_used: false,
            used_at: None,
            used_by_user_id: None,
            used_by_device_id: None,
            created_at: now,
            updated_at: now,
        }
    }
}