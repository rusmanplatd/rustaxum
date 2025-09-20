use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sqlx::PgPool;
use ulid::Ulid;

use crate::app::models::userorganization::UserOrganization;
use crate::app::http::requests::user_organization_requests::{
    CreateUserOrganizationRequest,
    UpdateUserOrganizationRequest,
};
use crate::query_builder::{QueryBuilder, QueryParams};

/// Get all user organization relationships with filtering and pagination
#[utoipa::path(
    get,
    path = "/api/user-organizations",
    tag = "User Organizations",
    summary = "List all user organization relationships",
    description = "Get all user organization relationships with optional filtering, sorting, and pagination",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination"),
        ("limit" = Option<u32>, Query, description = "Number of items per page (max 100)"),
        ("sort" = Option<String>, Query, description = "Sort field"),
        ("direction" = Option<String>, Query, description = "Sort direction (asc/desc)"),
    ),
    responses(
        (status = 200, description = "List of user organization relationships", body = Vec<crate::app::models::userorganization::UserOrganizationResponse>),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn index(
    State(pool): State<PgPool>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    // Authentication is handled by middleware

    let request = params.parse();
    let query_builder = QueryBuilder::<UserOrganization>::new(pool, request);

    match query_builder.paginate().await {
        Ok(result) => {
            (StatusCode::OK, Json(serde_json::json!(result))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch user organization relationships"
            }))).into_response()
        }
    }
}

/// Get a specific user organization relationship by ID
#[utoipa::path(
    get,
    path = "/api/user-organizations/{id}",
    tag = "User Organizations",
    summary = "Get user organization relationship by ID",
    description = "Retrieve a specific user organization relationship by its unique identifier",
    params(
        ("id" = String, Path, description = "User organization relationship unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "User organization relationship details", body = crate::app::models::userorganization::UserOrganizationResponse),
        (status = 404, description = "User organization relationship not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn show(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Authentication is handled by middleware
    let user_org_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid user organization ID format"
            }))).into_response();
        }
    };

    match sqlx::query_as::<_, UserOrganization>(
        "SELECT * FROM user_organizations WHERE id = $1"
    )
    .bind(user_org_id.to_string())
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(user_org)) => {
            (StatusCode::OK, Json(serde_json::json!(user_org.to_response()))).into_response()
        },
        Ok(None) => {
            (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "User organization relationship not found"
            }))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch user organization relationship"
            }))).into_response()
        }
    }
}

/// Create a new user organization relationship
#[utoipa::path(
    post,
    path = "/api/user-organizations",
    tag = "User Organizations",
    summary = "Create user organization relationship",
    description = "Create a new user organization relationship with validation",
    request_body = CreateUserOrganizationRequest,
    responses(
        (status = 201, description = "User organization relationship created successfully", body = crate::app::models::userorganization::UserOrganizationResponse),
        (status = 422, description = "Validation error", body = crate::app::http::form_request::ValidationErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn store(
    State(pool): State<PgPool>,
    request: CreateUserOrganizationRequest,
) -> impl IntoResponse {
    // Authentication is handled by middleware

    // Parse and validate IDs
    let user_id = match Ulid::from_string(&request.user_id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid user ID format"
            }))).into_response();
        }
    };

    let organization_id = match Ulid::from_string(&request.organization_id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid organization ID format"
            }))).into_response();
        }
    };

    let job_position_id = match Ulid::from_string(&request.job_position_id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid job position ID format"
            }))).into_response();
        }
    };

    // Create the user organization relationship
    let user_org = UserOrganization::new(
        user_id,
        organization_id,
        job_position_id,
        request.started_at,
    );

    // Insert into database
    match sqlx::query(
        r#"
        INSERT INTO user_organizations (id, user_id, organization_id, job_position_id, is_active, started_at, ended_at, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#
    )
    .bind(user_org.id.to_string())
    .bind(user_org.user_id.to_string())
    .bind(user_org.organization_id.to_string())
    .bind(user_org.job_position_id.to_string())
    .bind(user_org.is_active)
    .bind(user_org.started_at)
    .bind(user_org.ended_at)
    .bind(user_org.created_at)
    .bind(user_org.updated_at)
    .execute(&pool)
    .await
    {
        Ok(_) => {
            (StatusCode::CREATED, Json(serde_json::json!(user_org.to_response()))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to create user organization relationship"
            }))).into_response()
        }
    }
}

/// Update a user organization relationship
#[utoipa::path(
    put,
    path = "/api/user-organizations/{id}",
    tag = "User Organizations",
    summary = "Update user organization relationship",
    description = "Update an existing user organization relationship with validation",
    params(
        ("id" = String, Path, description = "User organization relationship unique identifier (ULID format)")
    ),
    request_body = UpdateUserOrganizationRequest,
    responses(
        (status = 200, description = "User organization relationship updated successfully", body = crate::app::models::userorganization::UserOrganizationResponse),
        (status = 404, description = "User organization relationship not found", body = crate::app::docs::ErrorResponse),
        (status = 422, description = "Validation error", body = crate::app::http::form_request::ValidationErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn update(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
    request: UpdateUserOrganizationRequest,
) -> impl IntoResponse {
    // Authentication is handled by middleware

    let user_org_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid user organization ID format"
            }))).into_response();
        }
    };

    // Fetch the existing user organization relationship
    let mut user_org = match sqlx::query_as::<_, UserOrganization>(
        "SELECT * FROM user_organizations WHERE id = $1"
    )
    .bind(user_org_id.to_string())
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(user_org)) => user_org,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "User organization relationship not found"
            }))).into_response();
        },
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch user organization relationship"
            }))).into_response();
        }
    };

    // Update fields if provided
    if let Some(organization_id_str) = &request.organization_id {
        match Ulid::from_string(organization_id_str) {
            Ok(org_id) => user_org.organization_id = org_id,
            Err(_) => {
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "error": "Invalid organization ID format"
                }))).into_response();
            }
        }
    }

    if let Some(job_position_id_str) = &request.job_position_id {
        match Ulid::from_string(job_position_id_str) {
            Ok(pos_id) => user_org.job_position_id = pos_id,
            Err(_) => {
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "error": "Invalid job position ID format"
                }))).into_response();
            }
        }
    }

    if let Some(is_active) = request.is_active {
        user_org.is_active = is_active;
    }

    if let Some(started_at) = request.started_at {
        user_org.started_at = started_at;
    }

    if let Some(ended_at) = request.ended_at {
        user_org.ended_at = Some(ended_at);
    }

    user_org.updated_at = chrono::Utc::now();

    // Update in database
    match sqlx::query(
        r#"
        UPDATE user_organizations
        SET organization_id = $2, job_position_id = $3, is_active = $4, started_at = $5, ended_at = $6, updated_at = $7
        WHERE id = $1
        "#
    )
    .bind(user_org.id.to_string())
    .bind(user_org.organization_id.to_string())
    .bind(user_org.job_position_id.to_string())
    .bind(user_org.is_active)
    .bind(user_org.started_at)
    .bind(user_org.ended_at)
    .bind(user_org.updated_at)
    .execute(&pool)
    .await
    {
        Ok(_) => {
            (StatusCode::OK, Json(serde_json::json!(user_org.to_response()))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to update user organization relationship"
            }))).into_response()
        }
    }
}

/// Delete a user organization relationship
#[utoipa::path(
    delete,
    path = "/api/user-organizations/{id}",
    tag = "User Organizations",
    summary = "Delete user organization relationship",
    description = "Delete an existing user organization relationship",
    params(
        ("id" = String, Path, description = "User organization relationship unique identifier (ULID format)")
    ),
    responses(
        (status = 204, description = "User organization relationship deleted successfully"),
        (status = 404, description = "User organization relationship not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn destroy(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Authentication is handled by middleware

    let user_org_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid user organization ID format"
            }))).into_response();
        }
    };

    // Check if exists first
    let exists = match sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_organizations WHERE id = $1"
    )
    .bind(user_org_id.to_string())
    .fetch_one(&pool)
    .await
    {
        Ok(count) => count > 0,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to check if user organization relationship exists"
            }))).into_response();
        }
    };

    if !exists {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "User organization relationship not found"
        }))).into_response();
    }

    // Delete from database
    match sqlx::query("DELETE FROM user_organizations WHERE id = $1")
        .bind(user_org_id.to_string())
        .execute(&pool)
        .await
    {
        Ok(_) => {
            (StatusCode::NO_CONTENT, Json(serde_json::json!({}))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to delete user organization relationship"
            }))).into_response()
        }
    }
}

/// Transfer user to different organization
#[utoipa::path(
    post,
    path = "/api/user-organizations/{id}/transfer",
    tag = "User Organizations",
    summary = "Transfer user to different organization",
    description = "Transfer a user to a different organization",
    params(
        ("id" = String, Path, description = "Current user organization relationship unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "User transferred successfully"),
        (status = 404, description = "User organization relationship not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn transfer(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Authentication is handled by middleware

    let user_org_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid user organization ID format"
            }))).into_response();
        }
    };

    // Extract new organization and job position IDs from payload
    let new_organization_id = match payload.get("organization_id").and_then(|v| v.as_str()) {
        Some(id_str) => match Ulid::from_string(id_str) {
            Ok(id) => id,
            Err(_) => {
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "error": "Invalid new organization ID format"
                }))).into_response();
            }
        },
        None => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "New organization ID is required"
            }))).into_response();
        }
    };

    let new_job_position_id = match payload.get("job_position_id").and_then(|v| v.as_str()) {
        Some(id_str) => match Ulid::from_string(id_str) {
            Ok(id) => id,
            Err(_) => {
                return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "error": "Invalid new job position ID format"
                }))).into_response();
            }
        },
        None => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "New job position ID is required"
            }))).into_response();
        }
    };

    // Fetch the current user organization relationship
    let mut user_org = match sqlx::query_as::<_, UserOrganization>(
        "SELECT * FROM user_organizations WHERE id = $1"
    )
    .bind(user_org_id.to_string())
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(user_org)) => user_org,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "User organization relationship not found"
            }))).into_response();
        },
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch user organization relationship"
            }))).into_response();
        }
    };

    // Perform the transfer
    match user_org.transfer_to_organization(&pool, new_organization_id, new_job_position_id).await {
        Ok(_) => {
            (StatusCode::OK, Json(serde_json::json!({
                "message": "User transferred successfully"
            }))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to transfer user"
            }))).into_response()
        }
    }
}

/// Activate user organization relationship
#[utoipa::path(
    post,
    path = "/api/user-organizations/{id}/activate",
    tag = "User Organizations",
    summary = "Activate user organization relationship",
    description = "Activate a deactivated user organization relationship",
    params(
        ("id" = String, Path, description = "User organization relationship unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "User organization relationship activated successfully", body = crate::app::models::userorganization::UserOrganizationResponse),
        (status = 404, description = "User organization relationship not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn activate(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Authentication is handled by middleware

    let user_org_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid user organization ID format"
            }))).into_response();
        }
    };

    // Fetch the user organization relationship
    let mut user_org = match sqlx::query_as::<_, UserOrganization>(
        "SELECT * FROM user_organizations WHERE id = $1"
    )
    .bind(user_org_id.to_string())
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(user_org)) => user_org,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "User organization relationship not found"
            }))).into_response();
        },
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch user organization relationship"
            }))).into_response();
        }
    };

    // Activate the relationship
    match user_org.activate(&pool).await {
        Ok(_) => {
            (StatusCode::OK, Json(serde_json::json!(user_org.to_response()))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to activate user organization relationship"
            }))).into_response()
        }
    }
}

/// Deactivate user organization relationship
#[utoipa::path(
    post,
    path = "/api/user-organizations/{id}/deactivate",
    tag = "User Organizations",
    summary = "Deactivate user organization relationship",
    description = "Deactivate an active user organization relationship",
    params(
        ("id" = String, Path, description = "User organization relationship unique identifier (ULID format)")
    ),
    responses(
        (status = 200, description = "User organization relationship deactivated successfully", body = crate::app::models::userorganization::UserOrganizationResponse),
        (status = 404, description = "User organization relationship not found", body = crate::app::docs::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::app::docs::ErrorResponse)
    )
)]
pub async fn deactivate(
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Authentication is handled by middleware

    let user_org_id = match Ulid::from_string(&id) {
        Ok(id) => id,
        Err(_) => {
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                "error": "Invalid user organization ID format"
            }))).into_response();
        }
    };

    // Fetch the user organization relationship
    let mut user_org = match sqlx::query_as::<_, UserOrganization>(
        "SELECT * FROM user_organizations WHERE id = $1"
    )
    .bind(user_org_id.to_string())
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(user_org)) => user_org,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "User organization relationship not found"
            }))).into_response();
        },
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to fetch user organization relationship"
            }))).into_response();
        }
    };

    // Deactivate the relationship
    match user_org.deactivate(&pool).await {
        Ok(_) => {
            (StatusCode::OK, Json(serde_json::json!(user_org.to_response()))).into_response()
        },
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to deactivate user organization relationship"
            }))).into_response()
        }
    }
}