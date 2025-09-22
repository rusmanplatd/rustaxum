pub mod validator;
pub mod errors;
pub mod rules;

pub use validator::{Validator, make_validator};
pub use errors::{ValidationError, ValidationErrors};
pub use rules::Rule;

use std::collections::HashMap;
use serde_json::Value;
use crate::database::DbPool;

pub type ValidationRules = HashMap<String, Vec<String>>;
pub type ValidationData = HashMap<String, Value>;

// Trait for easy validation integration
pub trait Validatable: serde::Serialize {
    fn validation_rules() -> ValidationRules;

    async fn validate(&self) -> Result<(), ValidationErrors> {
        self.validate_with_db(None).await
    }

    async fn validate_with_db(&self, db: Option<DbPool>)  -> Result<(), ValidationErrors> {
        let data = serde_json::to_value(self)
            .map_err(|_| {
                let mut errors = ValidationErrors::new();
                errors.add("_json", "serialization", "Failed to serialize data for validation");
                errors.finalize();
                errors
            })?;

        let mut validator = make_validator(data, Self::validation_rules());

        if let Some(db) = db {
            validator = validator.with_db(db);
        }

        validator.validate().await
    }
}

// Macro to help implement validation rules easily
#[macro_export]
macro_rules! validation_rules {
    ($($field:expr_2021 => [$($rule:expr_2021),* $(,)?]),* $(,)?) => {
        {
            let mut rules = std::collections::HashMap::new();
            $(
                rules.insert($field.to_string(), vec![$($rule.to_string()),*]);
            )*
            rules
        }
    };
}

// Helper functions for common validation scenarios
pub async fn validate_json(data: serde_json::Value, rules: ValidationRules) -> Result<(), ValidationErrors> {
    // tokio::runtime::Runtime::new().unwrap().block_on(async {
    // })
    make_validator(data, rules).validate().await
}

pub async fn validate_json_async(data: serde_json::Value, rules: ValidationRules) -> Result<(), ValidationErrors> {
    make_validator(data, rules).validate().await
}

pub async fn validate_json_with_db(data: serde_json::Value, rules: ValidationRules, db: DbPool) -> Result<(), ValidationErrors> {
    make_validator(data, rules).with_db(db).validate().await
}
