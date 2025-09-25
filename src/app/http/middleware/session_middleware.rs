use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::app::services::session::{SessionManager, SessionStore};
use crate::config::session::SessionConfig;
use crate::database::DbPool;

pub async fn session_middleware(
    State(pool): State<DbPool>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Load session configuration
    let session_config = SessionConfig::from_env()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create session manager with database pool
    let session_manager = SessionManager::new(
        session_config.clone(),
        Some(&pool),
        None, // Redis pool not used yet
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let store = SessionStore::new(session_manager, session_config);

    // Extract session ID from cookies
    let headers = request.headers().clone();
    let session_id = store.extract_session_id_from_cookies(&headers);

    // Start the session
    if let Err(_) = store.start(session_id).await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Add session store to request extensions
    request.extensions_mut().insert(store.clone());

    // Process the request
    let mut response = next.run(request).await;

    // Save session data
    if let Err(_) = store.save().await {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Add session cookie to response if we have a session ID
    if let Some(session_id) = store.get_session_id().await {
        response = store.add_session_cookie_to_response(response, &session_id);
    }

    Ok(response)
}