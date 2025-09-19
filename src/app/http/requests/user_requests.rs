use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use utoipa::ToSchema;

use crate::app::http::form_request::FormRequest;
use crate::app::validation::{Rule, required, email, min, max, string, numeric, in_list};
use crate::impl_form_request_extractor;

/// Update user profile form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateUserRequest {
    /// User's full name (2-100 characters)
    #[schema(example = "John Doe")]
    pub name: String,
    /// User's email address
    #[schema(example = "john@example.com")]
    pub email: String,
    /// User's biography (max 500 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Software developer with 5 years experience")]
    pub bio: Option<String>,
    /// User's phone number (10-15 digits)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "+1234567890")]
    pub phone: Option<String>,
    /// User's age (13-120)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 25)]
    pub age: Option<u32>,
}

#[async_trait]
impl FormRequest for UpdateUserRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("name", vec![required(), string(), min(2), max(100)]);
        rules.insert("email", vec![required(), email()]);
        rules.insert("bio", vec![string(), max(500)]);
        rules.insert("phone", vec![string(), min(10), max(15)]);
        rules.insert("age", vec![numeric(), min(13), max(120)]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("name.required", "Name is required");
        messages.insert("name.min", "Name must be at least 2 characters");
        messages.insert("name.max", "Name cannot exceed 100 characters");
        messages.insert("email.required", "Email is required");
        messages.insert("email.email", "Email must be a valid email address");
        messages.insert("bio.max", "Bio cannot exceed 500 characters");
        messages.insert("phone.min", "Phone number must be at least 10 digits");
        messages.insert("phone.max", "Phone number cannot exceed 15 digits");
        messages.insert("age.min", "Age must be at least 13");
        messages.insert("age.max", "Age cannot exceed 120");
        messages
    }

    fn attributes() -> HashMap<&'static str, &'static str> {
        let mut attributes = HashMap::new();
        attributes.insert("bio", "biography");
        attributes.insert("phone", "phone number");
        attributes
    }
}

impl_form_request_extractor!(UpdateUserRequest);

/// Search users form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct SearchUsersRequest {
    /// Search query (2-100 characters)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "john")]
    pub query: Option<String>,
    /// Page number (default: 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 1)]
    pub page: Option<u32>,
    /// Items per page (1-100, default: 15)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = 15)]
    pub per_page: Option<u32>,
    /// Sort field (name, email, created_at, updated_at)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "created_at")]
    pub sort_by: Option<String>,
    /// Sort direction (asc, desc)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "desc")]
    pub sort_direction: Option<String>,
}

#[async_trait]
impl FormRequest for SearchUsersRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("query", vec![string(), min(2), max(100)]);
        rules.insert("page", vec![numeric(), min(1)]);
        rules.insert("per_page", vec![numeric(), min(1), max(100)]);
        rules.insert("sort_by", vec![string(), in_list(vec!["name", "email", "created_at", "updated_at"])]);
        rules.insert("sort_direction", vec![string(), in_list(vec!["asc", "desc"])]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("query.min", "Search query must be at least 2 characters");
        messages.insert("query.max", "Search query cannot exceed 100 characters");
        messages.insert("page.min", "Page must be at least 1");
        messages.insert("per_page.min", "Per page must be at least 1");
        messages.insert("per_page.max", "Per page cannot exceed 100");
        messages.insert("sort_by.in", "Sort by must be one of: name, email, created_at, updated_at");
        messages.insert("sort_direction.in", "Sort direction must be either asc or desc");
        messages
    }

    fn prepare_for_validation(&mut self) {
        // Set default values
        if self.page.is_none() {
            self.page = Some(1);
        }
        if self.per_page.is_none() {
            self.per_page = Some(15);
        }
        if self.sort_by.is_none() {
            self.sort_by = Some("created_at".to_string());
        }
        if self.sort_direction.is_none() {
            self.sort_direction = Some("desc".to_string());
        }
    }
}

impl_form_request_extractor!(SearchUsersRequest);

/// Create contact form request
#[derive(Deserialize, Serialize, ToSchema)]
pub struct ContactFormRequest {
    /// Contact name (2-100 characters)
    #[schema(example = "John Doe")]
    pub name: String,
    /// Contact email address
    #[schema(example = "john@example.com")]
    pub email: String,
    /// Message subject (5-200 characters)
    #[schema(example = "Inquiry about your services")]
    pub subject: String,
    /// Message content (10-2000 characters)
    #[schema(example = "I would like to know more about your services...")]
    pub message: String,
}

#[async_trait]
impl FormRequest for ContactFormRequest {
    fn rules() -> HashMap<&'static str, Vec<Rule>> {
        let mut rules = HashMap::new();
        rules.insert("name", vec![required(), string(), min(2), max(100)]);
        rules.insert("email", vec![required(), email()]);
        rules.insert("subject", vec![required(), string(), min(5), max(200)]);
        rules.insert("message", vec![required(), string(), min(10), max(2000)]);
        rules
    }

    fn messages() -> HashMap<&'static str, &'static str> {
        let mut messages = HashMap::new();
        messages.insert("name.required", "Name is required");
        messages.insert("name.min", "Name must be at least 2 characters");
        messages.insert("name.max", "Name cannot exceed 100 characters");
        messages.insert("email.required", "Email is required");
        messages.insert("email.email", "Email must be a valid email address");
        messages.insert("subject.required", "Subject is required");
        messages.insert("subject.min", "Subject must be at least 5 characters");
        messages.insert("subject.max", "Subject cannot exceed 200 characters");
        messages.insert("message.required", "Message is required");
        messages.insert("message.min", "Message must be at least 10 characters");
        messages.insert("message.max", "Message cannot exceed 2000 characters");
        messages
    }
}

impl_form_request_extractor!(ContactFormRequest);