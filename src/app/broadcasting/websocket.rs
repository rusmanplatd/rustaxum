use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State, Path, Query,
    },
    response::Response,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{info, warn, error};

/// WebSocket connection manager for broadcasting
#[derive(Debug, Clone)]
pub struct WebSocketManager {
    /// Channel broadcasters for each channel
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<BroadcastMessage>>>>,
    /// Connected clients for each channel
    connections: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub channel: String,
    pub event: String,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    pub channel: Option<String>,
    pub auth_token: Option<String>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Subscribe to a channel and get a receiver
    pub async fn subscribe(&self, channel: &str) -> broadcast::Receiver<BroadcastMessage> {
        let mut channels = self.channels.write().await;
        let sender = channels.entry(channel.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(1000);
                info!("Created new broadcast channel: {}", channel);
                tx
            });

        sender.subscribe()
    }

    /// Broadcast a message to a channel
    pub async fn broadcast(&self, message: BroadcastMessage) -> Result<()> {
        let channels = self.channels.read().await;
        if let Some(sender) = channels.get(&message.channel) {
            match sender.send(message.clone()) {
                Ok(receiver_count) => {
                    info!("Broadcasted to channel '{}' with {} receivers", message.channel, receiver_count);
                }
                Err(_) => {
                    warn!("No receivers for channel '{}'", message.channel);
                }
            }
        } else {
            warn!("Channel '{}' does not exist", message.channel);
        }
        Ok(())
    }

    /// Add a connection to a channel
    pub async fn add_connection(&self, channel: &str, connection_id: String) {
        let mut connections = self.connections.write().await;
        connections.entry(channel.to_string())
            .or_insert_with(Vec::new)
            .push(connection_id);
    }

    /// Remove a connection from a channel
    pub async fn remove_connection(&self, channel: &str, connection_id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(channel_connections) = connections.get_mut(channel) {
            channel_connections.retain(|id| id != connection_id);
            if channel_connections.is_empty() {
                connections.remove(channel);
            }
        }
    }

    /// Get connection count for a channel
    pub async fn connection_count(&self, channel: &str) -> usize {
        let connections = self.connections.read().await;
        connections.get(channel).map(|v| v.len()).unwrap_or(0)
    }

    /// Get all active channels
    pub async fn active_channels(&self) -> Vec<String> {
        let connections = self.connections.read().await;
        connections.keys().cloned().collect()
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle WebSocket upgrade
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WebSocketQuery>,
    State(manager): State<Arc<WebSocketManager>>,
) -> Response {
    let channel = params.channel.unwrap_or_else(|| "general".to_string());

    // In a real app, you'd validate the auth_token here
    if let Some(_token) = params.auth_token {
        // Validate token and get user permissions
        info!("WebSocket connection with auth token for channel: {}", channel);
    }

    ws.on_upgrade(move |socket| handle_socket(socket, channel, manager))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, channel: String, manager: Arc<WebSocketManager>) {
    let connection_id = ulid::Ulid::new().to_string();
    info!("New WebSocket connection {} for channel: {}", connection_id, channel);

    // Add connection to manager
    manager.add_connection(&channel, connection_id.clone()).await;

    // Subscribe to channel broadcasts
    let mut receiver = manager.subscribe(&channel).await;

    // Split the socket into sender and receiver
    let (mut sender, mut receiver_ws) = socket.split();

    // Send welcome message
    let welcome_msg = BroadcastMessage {
        channel: channel.clone(),
        event: "connected".to_string(),
        data: serde_json::json!({
            "connection_id": connection_id,
            "message": "Connected to channel successfully"
        }),
        timestamp: chrono::Utc::now(),
    };

    if let Ok(welcome_json) = serde_json::to_string(&welcome_msg) {
        let _ = sender.send(Message::Text(welcome_json)).await;
    }

    // Handle incoming messages from client
    let manager_clone = manager.clone();
    let channel_clone = channel.clone();
    let connection_id_clone = connection_id.clone();

    let receive_task = tokio::spawn(async move {
        while let Some(msg) = receiver_ws.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    info!("Received message from {}: {}", connection_id_clone, text);

                    // Handle client messages (e.g., join different channels, send messages)
                    if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                        handle_client_message(client_msg, &manager_clone, &channel_clone).await;
                    }
                }
                Ok(Message::Binary(_)) => {
                    warn!("Binary messages not supported");
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection {} closed by client", connection_id_clone);
                    break;
                }
                Err(e) => {
                    error!("WebSocket error for connection {}: {}", connection_id_clone, e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Handle outgoing broadcasts to client
    let send_task = tokio::spawn(async move {
        while let Ok(broadcast_msg) = receiver.recv().await {
            if let Ok(json) = serde_json::to_string(&broadcast_msg) {
                if sender.send(Message::Text(json)).await.is_err() {
                    error!("Failed to send message to connection {}", connection_id);
                    break;
                }
            }
        }
    });

    // Wait for either task to complete (connection closed or error)
    tokio::select! {
        _ = receive_task => {},
        _ = send_task => {},
    }

    // Clean up connection
    manager.remove_connection(&channel, &connection_id).await;
    info!("WebSocket connection {} disconnected from channel: {}", connection_id, channel);
}

#[derive(Debug, Deserialize)]
struct ClientMessage {
    action: String,
    channel: Option<String>,
    data: Option<serde_json::Value>,
}

async fn handle_client_message(
    msg: ClientMessage,
    manager: &WebSocketManager,
    current_channel: &str,
) {
    match msg.action.as_str() {
        "ping" => {
            // Handle ping/pong for connection keep-alive
            let pong_msg = BroadcastMessage {
                channel: current_channel.to_string(),
                event: "pong".to_string(),
                data: serde_json::json!({"timestamp": chrono::Utc::now()}),
                timestamp: chrono::Utc::now(),
            };
            let _ = manager.broadcast(pong_msg).await;
        }
        "get_stats" => {
            // Send channel statistics
            let stats = serde_json::json!({
                "channel": current_channel,
                "connections": manager.connection_count(current_channel).await,
                "active_channels": manager.active_channels().await
            });

            let stats_msg = BroadcastMessage {
                channel: current_channel.to_string(),
                event: "stats".to_string(),
                data: stats,
                timestamp: chrono::Utc::now(),
            };
            let _ = manager.broadcast(stats_msg).await;
        }
        _ => {
            warn!("Unknown client action: {}", msg.action);
        }
    }
}

/// Create WebSocket routes
pub fn websocket_routes() -> Router<Arc<WebSocketManager>> {
    Router::new()
        .route("/ws", get(websocket_handler))
        .route("/ws/:channel", get(websocket_handler_with_channel))
}

/// WebSocket handler with channel parameter
async fn websocket_handler_with_channel(
    ws: WebSocketUpgrade,
    Path(channel): Path<String>,
    Query(params): Query<WebSocketQuery>,
    State(manager): State<Arc<WebSocketManager>>,
) -> Response {
    // Use channel from path, or fall back to query parameter
    let final_channel = if channel.is_empty() {
        params.channel.unwrap_or_else(|| "general".to_string())
    } else {
        channel
    };

    if let Some(_token) = params.auth_token {
        info!("WebSocket connection with auth token for channel: {}", final_channel);
    }

    ws.on_upgrade(move |socket| handle_socket(socket, final_channel, manager))
}

/// Create a complete WebSocket server
pub async fn create_websocket_server(port: u16) -> Result<()> {
    let manager = Arc::new(WebSocketManager::new());

    let app = websocket_routes()
        .with_state(manager.clone())
        .route("/health", get(|| async { "WebSocket server is running" }));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    info!("WebSocket server starting on port {}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Global WebSocket manager instance
static WS_MANAGER: tokio::sync::OnceCell<Arc<WebSocketManager>> = tokio::sync::OnceCell::const_new();

/// Get the global WebSocket manager
pub async fn websocket_manager() -> Arc<WebSocketManager> {
    WS_MANAGER.get_or_init(|| async {
        Arc::new(WebSocketManager::new())
    }).await.clone()
}

/// Broadcast a message using the global WebSocket manager
pub async fn websocket_broadcast(message: BroadcastMessage) -> Result<()> {
    let manager = websocket_manager().await;
    manager.broadcast(message).await
}