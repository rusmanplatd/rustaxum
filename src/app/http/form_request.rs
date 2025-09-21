use std::collections::HashMap;
use axum::{
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::app::validation::{ValidationRules, ValidationErrors, make_validator};

/// Response format for validation errors
#[derive(Serialize, ToSchema)]
pub struct ValidationErrorResponse {
    pub message: String,
    pub errors: HashMap<String, HashMap<String, String>>,
}

impl IntoResponse for ValidationErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::UNPROCESSABLE_ENTITY, Json(self)).into_response()
    }
}

/// Trait for form request validation similar to Laravel's FormRequest
#[async_trait]
pub trait FormRequest: for<'de> Deserialize<'de> + Serialize + Send + Sync + 'static {
    /// Define validation rules for the request
    fn rules() -> ValidationRules;

    /// Define custom validation messages (optional)
    fn messages() -> HashMap<&'static str, &'static str> {
        HashMap::new()
    }

    /// Define custom attribute names for error messages (optional)
    fn attributes() -> HashMap<&'static str, &'static str> {
        HashMap::new()
    }

    /// Authorize the request (optional, default: true)
    fn authorize(&self) -> bool {
        true
    }

    /// Prepare the data for validation (optional)
    fn prepare_for_validation(&mut self) {}

    /// Handle a failed validation attempt (optional)
    fn failed_validation(&self, errors: ValidationErrors) -> ValidationErrorResponse {
        ValidationErrorResponse {
            message: errors.message.clone(),
            errors: errors.errors,
        }
    }

    /// Handle a failed authorization attempt (optional)
    fn failed_authorization(&self) -> impl IntoResponse {
        (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "message": "This action is unauthorized."
        })))
    }

    /// Validate the request data
    async fn validate_data(&self, data: Value) -> Result<(), ValidationErrors> {
        let rules = Self::rules();
        let validator = make_validator(data, rules);
        validator.validate().await
    }
}

/// Helper function to validate and extract form request data
pub async fn extract_form_request<T>(req: Request) -> Result<T, ValidationErrorResponse>
where
    T: FormRequest + Serialize,
{
    let Json(mut payload): Json<T> = Json::from_request(req, &())
        .await
        .map_err(|_| ValidationErrorResponse {
            message: "Invalid JSON format.".to_string(),
            errors: HashMap::new(),
        })?;

    // Check authorization first
    if !payload.authorize() {
        return Err(ValidationErrorResponse {
            message: "This action is unauthorized.".to_string(),
            errors: HashMap::new(),
        });
    }

    // Prepare data for validation
    payload.prepare_for_validation();

    // Convert to JSON for validation
    let json_data = serde_json::to_value(&payload)
        .map_err(|_| ValidationErrorResponse {
            message: "Failed to serialize data for validation.".to_string(),
            errors: HashMap::new(),
        })?;

    // Validate the data
    if let Err(validation_errors) = payload.validate_data(json_data).await {
        return Err(payload.failed_validation(validation_errors));
    }

    Ok(payload)
}

/// Macro to implement FromRequest for FormRequest types
#[macro_export]
macro_rules! impl_form_request_extractor {
    ($name:ty) => {
        impl<S> axum::extract::FromRequest<S> for $name
        where
            S: Send + Sync,
        {
            type Rejection = $crate::app::http::form_request::ValidationErrorResponse;

            fn from_request(
                req: axum::extract::Request,
                state: &S,
            ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
                async move {
                let axum::Json(mut payload): axum::Json<$name> = axum::Json::from_request(req, state)
                    .await
                    .map_err(|_| $crate::app::http::form_request::ValidationErrorResponse {
                        message: "Invalid JSON format.".to_string(),
                        errors: std::collections::HashMap::new(),
                    })?;

                // Check authorization first
                if !<$name as $crate::app::http::form_request::FormRequest>::authorize(&payload) {
                    return Err($crate::app::http::form_request::ValidationErrorResponse {
                        message: "This action is unauthorized.".to_string(),
                        errors: std::collections::HashMap::new(),
                    });
                }

                // Prepare data for validation
                <$name as $crate::app::http::form_request::FormRequest>::prepare_for_validation(&mut payload);

                // Convert to JSON for validation
                let json_data = serde_json::to_value(&payload)
                    .map_err(|_| $crate::app::http::form_request::ValidationErrorResponse {
                        message: "Failed to serialize data for validation.".to_string(),
                        errors: std::collections::HashMap::new(),
                    })?;

                // Validate the data
                if let Err(validation_errors) = <$name as $crate::app::http::form_request::FormRequest>::validate_data(&payload, json_data).await {
                    return Err(<$name as $crate::app::http::form_request::FormRequest>::failed_validation(&payload, validation_errors));
                }

                Ok(payload)
                }
            }
        }
    };
}

/// Macro to easily create FormRequest implementations
#[macro_export]
macro_rules! form_request {
    (
        $name:ident {
            $($field:ident: $field_type:ty),* $(,)?
        },
        rules: {
            $($rule_field:literal => [$($rule:expr_2021),* $(,)?]),* $(,)?
        }
        $(, messages: {
            $($msg_key:literal => $msg_value:literal),* $(,)?
        })?
        $(, attributes: {
            $($attr_key:literal => $attr_value:literal),* $(,)?
        })?
        $(, authorize: $auth_fn:expr_2021)?
    ) => {
        #[derive(serde::Deserialize, serde::Serialize)]
        pub struct $name {
            $(pub $field: $field_type,)*
        }

        #[async_trait::async_trait]
        impl $crate::app::http::form_request::FormRequest for $name {
            fn rules() -> $crate::app::validation::ValidationRules {
                $crate::validation_rules! {
                    $(
                        $rule_field => [$($rule),*]
                    ),*
                }
            }

            $(
                fn messages() -> std::collections::HashMap<&'static str, &'static str> {
                    let mut messages = std::collections::HashMap::new();
                    $(
                        messages.insert($msg_key, $msg_value);
                    )*
                    messages
                }
            )?

            $(
                fn attributes() -> std::collections::HashMap<&'static str, &'static str> {
                    let mut attributes = std::collections::HashMap::new();
                    $(
                        attributes.insert($attr_key, $attr_value);
                    )*
                    attributes
                }
            )?

            $(
                fn authorize(&self) -> bool {
                    $auth_fn(self)
                }
            )?
        }

        $crate::impl_form_request_extractor!($name);
    };
}

#[cfg(test)]
mod tests {
    use crate::validation_rules;

    use super::*;

    // Test FormRequest implementation
    #[derive(Deserialize, Serialize)]
    struct TestFormRequest {
        name: String,
        email: String,
        age: u32,
    }

    #[async_trait]
    impl FormRequest for TestFormRequest {
        fn rules() -> ValidationRules {
            validation_rules! {
                "name" => ["required", "string", "min:2"],
                "email" => ["required", "email"],
                "age" => ["required", "integer"]
            }
        }

        fn messages() -> HashMap<&'static str, &'static str> {
            let mut messages = HashMap::new();
            messages.insert("name.required", "Name is required");
            messages.insert("email.email", "Email must be valid");
            messages
        }
    }

    #[test]
    fn test_validation_error_response_serialization() {
        let mut field_errors = HashMap::new();
        field_errors.insert("required".to_string(), "Email is required".to_string());

        let mut errors = HashMap::new();
        errors.insert("email".to_string(), field_errors);

        let response = ValidationErrorResponse {
            message: "Validation failed".to_string(),
            errors,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Validation failed"));
        assert!(json.contains("Email is required"));
    }

    #[test]
    fn test_form_request_rules() {
        let rules = TestFormRequest::rules();
        assert!(rules.contains_key("name"));
        assert!(rules.contains_key("email"));
        assert!(rules.contains_key("age"));
    }

    #[test]
    fn test_form_request_messages() {
        let messages = TestFormRequest::messages();
        assert_eq!(messages.get("name.required"), Some(&"Name is required"));
        assert_eq!(messages.get("email.email"), Some(&"Email must be valid"));
    }
}