use serde::{Deserialize, Serialize};
use crate::app::models::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResource {
    pub id: String,
    // Add resource fields here
}

impl UserResource {
    pub fn from_model(model: User) -> Self {
        Self {
            id: model.id,
            // Map model fields to resource fields
        }
    }

    pub fn collection(models: Vec<User>) -> Vec<Self> {
        models.into_iter().map(Self::from_model).collect()
    }
}
