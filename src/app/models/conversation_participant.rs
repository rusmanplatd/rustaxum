use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use crate::schema::conversation_participants;
use super::DieselUlid;
use chrono::{DateTime, Utc};
use crate::app::models::{HasModelType, activity_log::HasId};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = conversation_participants)]
#[diesel(primary_key(id))]
pub struct ConversationParticipant {
    pub id: DieselUlid,
    pub conversation_id: DieselUlid,
    pub user_id: DieselUlid,
    pub role: String,
    pub is_active: bool,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub last_read_message_id: Option<DieselUlid>,
    pub last_read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParticipantRole {
    Owner,
    Admin,
    Member,
    Moderator,
}

impl From<String> for ParticipantRole {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "owner" => ParticipantRole::Owner,
            "admin" => ParticipantRole::Admin,
            "member" => ParticipantRole::Member,
            "moderator" => ParticipantRole::Moderator,
            _ => ParticipantRole::Member,
        }
    }
}

impl From<ParticipantRole> for String {
    fn from(pr: ParticipantRole) -> Self {
        match pr {
            ParticipantRole::Owner => "owner".to_string(),
            ParticipantRole::Admin => "admin".to_string(),
            ParticipantRole::Member => "member".to_string(),
            ParticipantRole::Moderator => "moderator".to_string(),
        }
    }
}

impl ConversationParticipant {
    pub fn role_enum(&self) -> ParticipantRole {
        self.role.clone().into()
    }

    pub fn is_owner(&self) -> bool {
        matches!(self.role_enum(), ParticipantRole::Owner)
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.role_enum(), ParticipantRole::Admin | ParticipantRole::Owner)
    }

    pub fn can_moderate(&self) -> bool {
        matches!(
            self.role_enum(),
            ParticipantRole::Owner | ParticipantRole::Admin | ParticipantRole::Moderator
        )
    }
    
    pub fn new(conversation_id: DieselUlid, user_id: DieselUlid, role: ParticipantRole) -> Self {
        let now = Utc::now();
        ConversationParticipant {
            id: DieselUlid::new(),
            conversation_id,
            user_id,
            role: role.into(),
            is_active: true,
            joined_at: now,
            left_at: None,
            last_read_message_id: None,
            last_read_at: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    pub fn member(conversation_id: DieselUlid, user_id: DieselUlid) -> Self {
        ConversationParticipant::new(conversation_id, user_id, ParticipantRole::Member)
    }

    pub fn owner(conversation_id: DieselUlid, user_id: DieselUlid) -> Self {
        ConversationParticipant::new(conversation_id, user_id, ParticipantRole::Owner)
    }

    pub fn admin(conversation_id: DieselUlid, user_id: DieselUlid) -> Self {
        ConversationParticipant::new(conversation_id, user_id, ParticipantRole::Admin)
    }
}

impl HasModelType for ConversationParticipant {
    fn model_type() -> &'static str {
        "ConversationParticipant"
    }
}

impl HasId for ConversationParticipant {
    fn id(&self) -> String {
        self.id.to_string()
    }
}