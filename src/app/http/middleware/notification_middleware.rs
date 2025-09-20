use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tokio::time::Instant;
use tracing::{info, warn, error};

use crate::app::notifications::channels::ChannelManager;

/// Middleware state for notification handling
#[derive(Debug, Clone)]
pub struct NotificationMiddlewareState {
    pub channel_manager: Arc<ChannelManager>,
    pub auto_dispatch_events: bool,
    pub auto_process_jobs: bool,
    pub metrics_enabled: bool,
}

impl NotificationMiddlewareState {
    pub async fn new() -> Self {
        Self {
            channel_manager: Arc::new(ChannelManager::new().await),
            auto_dispatch_events: true,
            auto_process_jobs: true,
            metrics_enabled: true,
        }
    }

    pub fn with_auto_dispatch(mut self, enabled: bool) -> Self {
        self.auto_dispatch_events = enabled;
        self
    }

    pub fn with_auto_jobs(mut self, enabled: bool) -> Self {
        self.auto_process_jobs = enabled;
        self
    }

    pub fn with_metrics(mut self, enabled: bool) -> Self {
        self.metrics_enabled = enabled;
        self
    }
}

/// Middleware for handling notifications and events in HTTP requests
pub async fn notification_middleware(
    State(state): State<Arc<NotificationMiddlewareState>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let start_time = if state.metrics_enabled {
        Some(Instant::now())
    } else {
        None
    };

    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path();

    // Log the request
    info!("Processing request: {} {}", method, path);

    // Process the request
    let response = next.run(request).await;

    // Record metrics if enabled
    if let Some(start) = start_time {
        let duration = start.elapsed();
        info!("Request {} {} completed in {:?}", method, path, duration);
    }

    // Handle post-request notifications based on response status
    let status = response.status();
    if state.auto_dispatch_events {
        handle_response_events(&state, &method, path, status).await;
    }

    Ok(response)
}

/// Handle events based on HTTP response
async fn handle_response_events(
    state: &NotificationMiddlewareState,
    method: &axum::http::Method,
    path: &str,
    status: StatusCode,
) {
    // Create events based on specific endpoints and status codes
    match (method.as_str(), path, status.as_u16()) {
        ("POST", "/api/users", 201) => {
            info!("User created successfully - would trigger UserRegisteredEvent");
            // In a real app, you'd extract user data from the response and create the event
        },
        ("POST", "/api/orders", 201) => {
            info!("Order created successfully - would trigger OrderCreatedEvent");
        },
        ("PUT", "/api/orders/*/ship", 200) => {
            info!("Order shipped successfully - would trigger OrderShippedEvent");
        },
        ("POST", "/api/payments", 402) => {
            warn!("Payment failed - would trigger PaymentFailedEvent");
        },
        (_, _, code) if code >= 500 => {
            error!("Server error {} on {} {} - would trigger SystemErrorEvent", code, method, path);
        },
        _ => {
            // No specific event handling for this request
        }
    }
}

/// Middleware for event correlation and tracking
pub async fn event_correlation_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Generate correlation ID for request tracking
    let correlation_id = ulid::Ulid::new().to_string();

    // Add correlation ID to tracing context
    let span = tracing::info_span!("request", correlation_id = %correlation_id);
    let _enter = span.enter();

    info!("Request started with correlation ID: {}", correlation_id);

    // Process request
    let mut response = next.run(request).await;

    // Add correlation ID to response headers
    response.headers_mut().insert(
        "X-Correlation-ID",
        correlation_id.parse().unwrap_or_else(|_| {
            warn!("Failed to parse correlation ID as header value");
            "invalid".parse().unwrap()
        })
    );

    info!("Request completed with correlation ID: {}", correlation_id);

    Ok(response)
}

/// Middleware for rate limiting notifications
pub async fn notification_rate_limit_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // In a real implementation, you would:
    // 1. Check rate limits for the user/IP
    // 2. Track notification sending rates
    // 3. Apply backpressure if limits are exceeded

    let response = next.run(request).await;

    // For now, just log that rate limiting would be applied
    if response.status().is_success() {
        // Check if this is a notification-triggering endpoint
        // Apply rate limiting logic here
    }

    Ok(response)
}

/// Error handling middleware for notification failures
pub async fn notification_error_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let response = next.run(request).await;

    // If there's an error, we might want to send notifications
    if response.status().is_server_error() {
        warn!("Server error occurred - consider sending admin notification");

        // In a real app, you might:
        // 1. Send error notifications to administrators
        // 2. Log errors for monitoring
        // 3. Create incident events
    }

    Ok(response)
}

/// Middleware builder for notification-related middleware
pub struct NotificationMiddlewareBuilder {
    state: NotificationMiddlewareState,
}

impl NotificationMiddlewareBuilder {
    pub async fn new() -> Self {
        Self {
            state: NotificationMiddlewareState::new().await,
        }
    }

    pub fn with_auto_dispatch(mut self, enabled: bool) -> Self {
        self.state = self.state.with_auto_dispatch(enabled);
        self
    }

    pub fn with_auto_jobs(mut self, enabled: bool) -> Self {
        self.state = self.state.with_auto_jobs(enabled);
        self
    }

    pub fn with_metrics(mut self, enabled: bool) -> Self {
        self.state = self.state.with_metrics(enabled);
        self
    }

    pub fn build(self) -> Arc<NotificationMiddlewareState> {
        Arc::new(self.state)
    }
}

/// Helper functions for creating notification middleware layers
pub mod layers {

    /// Create a complete notification middleware stack
    pub async fn notification_stack() {
        // Simplified for now - individual middleware functions can be used separately
        tracing::info!("Notification middleware stack initialized");
    }

    /// Create a basic notification middleware layer
    pub async fn basic_notifications() {
        // Simplified for now
        tracing::info!("Basic notification middleware initialized");
    }

    /// Create correlation middleware
    pub fn correlation() {
        // Simplified for now
        tracing::info!("Correlation middleware initialized");
    }
}