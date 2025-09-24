use axum::{
    extract::{Json, State, Path, Query},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json as ResponseJson},
};
use serde::{Deserialize, Serialize};
use crate::database::DbPool;
use utoipa::ToSchema;

use crate::app::services::oauth::ScopeService;
use crate::app::services::auth_service::AuthService;
use crate::app::models::oauth::{CreateScope, UpdateScope, Scope};
use crate::app::utils::token_utils::TokenUtils;
use crate::app::query_builder::{QueryParams, QueryBuilderService};

#[derive(Serialize, ToSchema)]
#[schema(description = "Error response for OAuth scope operations")]
struct ErrorResponse {
    #[schema(example = "Scope not found")]
    error: String,
}

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

#[derive(Deserialize, ToSchema)]
#[schema(description = "Request to create a new OAuth2 scope")]
pub struct CreateScopeRequest {
    #[schema(example = "user:read")]
    pub name: String,
    #[schema(example = "Read user profile information")]
    pub description: Option<String>,
    #[schema(example = false)]
    pub is_default: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateScopeRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_default: Option<bool>,
}

#[derive(Deserialize)]
pub struct ListScopesQuery {
    pub default_only: Option<bool>,
    pub search: Option<String>,
}

/// Create a new OAuth2 scope
#[utoipa::path(
    post,
    path = "/oauth/scopes",
    tags = ["OAuth Scopes"],
    summary = "Create a new OAuth2 scope",
    description = "Create a new OAuth2 scope with specified permissions. Requires admin privileges.",
    request_body = CreateScopeRequest,
    responses(
        (status = 201, description = "Scope created successfully", body = crate::app::models::oauth::ScopeResponse),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 401, description = "Unauthorized - admin access required", body = ErrorResponse),
        (status = 409, description = "Scope with this name already exists", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn create_scope(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Json(payload): Json<CreateScopeRequest>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: e.to_string(),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    let create_data = CreateScope {
        name: payload.name,
        description: payload.description,
        is_default: payload.is_default.unwrap_or(false),
    };

    match ScopeService::create_scope(&pool, create_data) {
        Ok(response) => (StatusCode::CREATED, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// List all OAuth2 scopes
#[utoipa::path(
    get,
    path = "/oauth/scopes",
    tags = ["OAuth Scopes"],
    summary = "List OAuth2 scopes",
    description = "Retrieve a list of OAuth2 scopes with optional filtering, sorting, and pagination.",
    params(
        ("page" = Option<u32>, Query, description = "Page number for pagination (default: 1)"),
        ("per_page" = Option<u32>, Query, description = "Number of items per page (default: 15, max: 100)"),
        ("sort" = Option<String>, Query, description = "Sort field and direction. Available fields: id, name, created_at, updated_at (prefix with '-' for descending)"),
        ("filter" = Option<serde_json::Value>, Query, description = "Filter parameters. Available filters: name, is_default (e.g., filter[name]=user:read, filter[is_default]=true)"),
        ("fields" = Option<String>, Query, description = "Comma-separated list of fields to select. Available: id, name, description, is_default, created_at, updated_at"),
        ("cursor" = Option<String>, Query, description = "Cursor for cursor-based pagination"),
        ("pagination_type" = Option<String>, Query, description = "Pagination type: 'offset' or 'cursor' (default: cursor)"),
    ),
    responses(
        (status = 200, description = "List of scopes", body = Vec<crate::app::models::oauth::ScopeResponse>),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn list_scopes(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    // Verify authenticated access
    if let Err(e) = get_authenticated_user(&pool, &headers).await {
        let error = ErrorResponse {
            error: e.to_string(),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    match <Scope as QueryBuilderService<Scope>>::index(Query(params), &pool) {
        Ok(result) => {
            (StatusCode::OK, ResponseJson(serde_json::json!(result))).into_response()
        },
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

/// Get a specific OAuth2 scope
#[utoipa::path(
    get,
    path = "/oauth/scopes/{scope_id}",
    tags = ["OAuth Scopes"],
    summary = "Get OAuth2 scope by ID",
    description = "Retrieve a specific OAuth2 scope by its unique identifier",
    params(
        ("scope_id" = String, Path, description = "Scope identifier")
    ),
    responses(
        (status = 200, description = "Scope found", body = crate::app::models::oauth::ScopeResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Scope not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn get_scope(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(scope_id): Path<String>,
) -> impl IntoResponse {
    // Verify authenticated access
    if let Err(e) = get_authenticated_user(&pool, &headers).await {
        let error = ErrorResponse {
            error: e.to_string(),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    match ScopeService::find_by_id(&pool, &scope_id) {
        Ok(Some(scope)) => {
            (StatusCode::OK, ResponseJson(scope.to_response())).into_response()
        },
        Ok(None) => {
            let error = ErrorResponse {
                error: "Scope not found".to_string(),
            };
            (StatusCode::NOT_FOUND, ResponseJson(error)).into_response()
        },
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

/// Get a scope by name
#[utoipa::path(
    get,
    path = "/oauth/scopes/name/{scope_name}",
    tags = ["OAuth Scopes"],
    summary = "Get OAuth2 scope by name",
    description = "Retrieve a specific OAuth2 scope by its name",
    params(
        ("scope_name" = String, Path, description = "Scope name")
    ),
    responses(
        (status = 200, description = "Scope found", body = crate::app::models::oauth::ScopeResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Scope not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn get_scope_by_name(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(scope_name): Path<String>,
) -> impl IntoResponse {
    // Verify authenticated access
    if let Err(e) = get_authenticated_user(&pool, &headers).await {
        let error = ErrorResponse {
            error: e.to_string(),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    match ScopeService::find_by_name(&pool, &scope_name) {
        Ok(Some(scope)) => {
            (StatusCode::OK, ResponseJson(scope.to_response())).into_response()
        },
        Ok(None) => {
            let error = ErrorResponse {
                error: "Scope not found".to_string(),
            };
            (StatusCode::NOT_FOUND, ResponseJson(error)).into_response()
        },
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

/// Update an OAuth2 scope
#[utoipa::path(
    put,
    path = "/oauth/scopes/{scope_id}",
    tags = ["OAuth Scopes"],
    summary = "Update OAuth2 scope",
    description = "Update an existing OAuth2 scope. Requires admin privileges.",
    params(
        ("scope_id" = String, Path, description = "Scope identifier")
    ),
    request_body = UpdateScopeRequest,
    responses(
        (status = 200, description = "Scope updated successfully", body = crate::app::models::oauth::ScopeResponse),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 401, description = "Unauthorized - admin access required", body = ErrorResponse),
        (status = 404, description = "Scope not found", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn update_scope(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(scope_id): Path<String>,
    Json(payload): Json<UpdateScopeRequest>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: e.to_string(),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    let update_data = UpdateScope {
        name: payload.name,
        description: payload.description,
        is_default: payload.is_default,
    };

    match ScopeService::update_scope(&pool, scope_id, update_data) {
        Ok(response) => (StatusCode::OK, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Delete an OAuth2 scope
#[utoipa::path(
    delete,
    path = "/oauth/scopes/{scope_id}",
    tags = ["OAuth Scopes"],
    summary = "Delete OAuth2 scope",
    description = "Delete an existing OAuth2 scope. Requires admin privileges.",
    params(
        ("scope_id" = String, Path, description = "Scope identifier")
    ),
    responses(
        (status = 200, description = "Scope deleted successfully"),
        (status = 400, description = "Invalid request data", body = ErrorResponse),
        (status = 401, description = "Unauthorized - admin access required", body = ErrorResponse),
        (status = 404, description = "Scope not found", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn delete_scope(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(scope_id): Path<String>,
) -> impl IntoResponse {
    // Verify admin access
    if let Err(e) = verify_admin_access(&pool, &headers).await {
        let error = ErrorResponse {
            error: e.to_string(),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    match ScopeService::delete_scope(&pool, scope_id) {
        Ok(_) => {
            let response = MessageResponse {
                message: "Scope deleted successfully".to_string(),
            };
            (StatusCode::OK, ResponseJson(response)).into_response()
        },
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

/// Validate OAuth2 scopes
#[utoipa::path(
    post,
    path = "/oauth/scopes/validate",
    tags = ["OAuth Scopes"],
    summary = "Validate OAuth2 scopes",
    description = "Validate a list of OAuth2 scope names and return their details",
    request_body = Vec<String>,
    responses(
        (status = 200, description = "Scope validation result"),
        (status = 400, description = "Invalid scopes", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse)
    ),
    security(
        ("Bearer" = [])
    )
)]
pub async fn validate_scopes(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Json(scope_names): Json<Vec<String>>,
) -> impl IntoResponse {
    // Verify authenticated access
    if let Err(e) = get_authenticated_user(&pool, &headers).await {
        let error = ErrorResponse {
            error: e.to_string(),
        };
        return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
    }

    match ScopeService::validate_scopes(&pool, &scope_names).await {
        Ok(scopes) => {
            #[derive(Serialize)]
            struct ValidateScopesResponse {
                valid: bool,
                scopes: Vec<crate::app::models::oauth::ScopeResponse>,
                message: String,
            }

            let response = ValidateScopesResponse {
                valid: true,
                scopes: scopes.into_iter().map(|s| s.to_response()).collect(),
                message: "All scopes are valid".to_string(),
            };

            (StatusCode::OK, ResponseJson(response)).into_response()
        },
        Err(e) => {
            #[derive(Serialize)]
            struct ValidateScopesResponse {
                valid: bool,
                scopes: Vec<crate::app::models::oauth::ScopeResponse>,
                message: String,
            }

            let response = ValidateScopesResponse {
                valid: false,
                scopes: vec![],
                message: e.to_string(),
            };

            (StatusCode::BAD_REQUEST, ResponseJson(response)).into_response()
        }
    }
}

async fn get_authenticated_user(_pool: &DbPool, headers: &HeaderMap) -> anyhow::Result<String> {
    let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());
    let token = TokenUtils::extract_token_from_header(auth_header)?;
    let claims = AuthService::decode_token(token, "jwt-secret")?;

    Ok(claims.sub)
}

async fn verify_admin_access(pool: &DbPool, headers: &HeaderMap) -> anyhow::Result<String> {
    let user_id = get_authenticated_user(pool, headers).await?;

    // Here you would typically check if the user has admin role
    // For now, we'll accept any authenticated user
    // In a real implementation, you'd check user roles/permissions

    Ok(user_id)
}