use axum::{
    extract::{Json, State, Path},
    http::{StatusCode, HeaderMap},
    response::{IntoResponse, Json as ResponseJson},
};
use serde::{Deserialize, Serialize};
use crate::database::DbPool;
use ulid::Ulid;

use crate::app::services::oauth::ClientService;
use crate::app::services::auth_service::AuthService;
use crate::app::models::oauth::{CreateClient, UpdateClient};
use crate::app::utils::token_utils::TokenUtils;

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Deserialize)]
pub struct CreateClientRequest {
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub personal_access_client: Option<bool>,
    pub password_client: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateClientRequest {
    pub name: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    pub revoked: Option<bool>,
}

pub async fn create_client(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Json(payload): Json<CreateClientRequest>,
) -> impl IntoResponse {
    let user_id = match get_authenticated_user(&pool, &headers).await {
        Ok(user_id) => Some(user_id),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    let create_data = CreateClient {
        user_id,
        name: payload.name,
        redirect_uris: payload.redirect_uris,
        personal_access_client: payload.personal_access_client.unwrap_or(false),
        password_client: payload.password_client.unwrap_or(false),
    };

    match ClientService::create_client(&pool, create_data) {
        Ok(response) => (StatusCode::CREATED, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

pub async fn list_clients(
    State(pool): State<DbPool>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let user_id = match get_authenticated_user(&pool, &headers).await {
        Ok(user_id) => Some(user_id),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    match ClientService::list_clients(&pool, user_id) {
        Ok(clients) => (StatusCode::OK, ResponseJson(clients)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

pub async fn get_client(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(client_id): Path<String>,
) -> impl IntoResponse {
    let user_id = match get_authenticated_user(&pool, &headers).await {
        Ok(user_id) => user_id,
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    match ClientService::find_by_id(&pool, client_id) {
        Ok(Some(client)) => {
            // Check if user owns this client or if it's a system client
            if let Some(ref owner_id) = client.user_id {
                if owner_id != &user_id {
                    let error = ErrorResponse {
                        error: "Access denied".to_string(),
                    };
                    return (StatusCode::FORBIDDEN, ResponseJson(error)).into_response();
                }
            }
            (StatusCode::OK, ResponseJson(client.to_response())).into_response()
        },
        Ok(None) => {
            let error = ErrorResponse {
                error: "Client not found".to_string(),
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

pub async fn update_client(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(client_id): Path<String>,
    Json(payload): Json<UpdateClientRequest>,
) -> impl IntoResponse {
    let user_id = match get_authenticated_user(&pool, &headers).await {
        Ok(user_id) => user_id,
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    // Check if user owns this client
    match ClientService::find_by_id(&pool, client_id.clone()) {
        Ok(Some(client)) => {
            if let Some(ref owner_id) = client.user_id {
                if owner_id != &user_id {
                    let error = ErrorResponse {
                        error: "Access denied".to_string(),
                    };
                    return (StatusCode::FORBIDDEN, ResponseJson(error)).into_response();
                }
            }
        },
        Ok(None) => {
            let error = ErrorResponse {
                error: "Client not found".to_string(),
            };
            return (StatusCode::NOT_FOUND, ResponseJson(error)).into_response();
        },
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    }

    let update_data = UpdateClient {
        name: payload.name,
        redirect_uris: payload.redirect_uris,
        revoked: payload.revoked,
    };

    match ClientService::update_client(&pool, client_id, update_data) {
        Ok(response) => (StatusCode::OK, ResponseJson(response)).into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response()
        }
    }
}

pub async fn delete_client(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(client_id): Path<String>,
) -> impl IntoResponse {
    let user_id = match get_authenticated_user(&pool, &headers).await {
        Ok(user_id) => user_id,
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    let client_ulid = match Ulid::from_string(&client_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid client ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    // Check if user owns this client
    match ClientService::find_by_id(&pool, client_ulid) {
        Ok(Some(client)) => {
            if let Some(ref owner_id) = client.user_id {
                if owner_id != &user_id {
                    let error = ErrorResponse {
                        error: "Access denied".to_string(),
                    };
                    return (StatusCode::FORBIDDEN, ResponseJson(error)).into_response();
                }
            }
        },
        Ok(None) => {
            let error = ErrorResponse {
                error: "Client not found".to_string(),
            };
            return (StatusCode::NOT_FOUND, ResponseJson(error)).into_response();
        },
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    }

    match ClientService::delete_client(&pool, client_id) {
        Ok(_) => (StatusCode::NO_CONTENT, "").into_response(),
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response()
        }
    }
}

pub async fn regenerate_secret(
    State(pool): State<DbPool>,
    headers: HeaderMap,
    Path(client_id): Path<String>,
) -> impl IntoResponse {
    let user_id = match get_authenticated_user(&pool, &headers).await {
        Ok(user_id) => user_id,
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            return (StatusCode::UNAUTHORIZED, ResponseJson(error)).into_response();
        }
    };

    let client_ulid = match Ulid::from_string(&client_id) {
        Ok(id) => id,
        Err(_) => {
            let error = ErrorResponse {
                error: "Invalid client ID format".to_string(),
            };
            return (StatusCode::BAD_REQUEST, ResponseJson(error)).into_response();
        }
    };

    // Check if user owns this client
    match ClientService::find_by_id(&pool, client_ulid) {
        Ok(Some(client)) => {
            if let Some(ref owner_id) = client.user_id {
                if owner_id != &user_id {
                    let error = ErrorResponse {
                        error: "Access denied".to_string(),
                    };
                    return (StatusCode::FORBIDDEN, ResponseJson(error)).into_response();
                }
            }
        },
        Ok(None) => {
            let error = ErrorResponse {
                error: "Client not found".to_string(),
            };
            return (StatusCode::NOT_FOUND, ResponseJson(error)).into_response();
        },
        Err(e) => {
            let error = ErrorResponse {
                error: e.to_string(),
            };
            return (StatusCode::INTERNAL_SERVER_ERROR, ResponseJson(error)).into_response();
        }
    }

    match ClientService::regenerate_secret(&pool, client_ulid) {
        Ok(new_secret) => {
            #[derive(Serialize)]
            struct SecretResponse {
                secret: String,
                message: String,
            }

            let response = SecretResponse {
                secret: new_secret,
                message: "Client secret regenerated successfully".to_string(),
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

async fn get_authenticated_user(_pool: &DbPool, headers: &HeaderMap) -> anyhow::Result<String> {
    let auth_header = headers.get("authorization").and_then(|h| h.to_str().ok());
    let token = TokenUtils::extract_token_from_header(auth_header)?;
    let claims = AuthService::decode_token(token, "jwt-secret")?;

    Ok(claims.sub)
}