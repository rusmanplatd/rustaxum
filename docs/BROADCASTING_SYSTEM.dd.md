# Broadcasting System Documentation

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Configuration](#configuration)
4. [Broadcasting Drivers](#broadcasting-drivers)
5. [Events and Broadcasting](#events-and-broadcasting)
6. [WebSocket Implementation](#websocket-implementation)
7. [Channel Management](#channel-management)
8. [Authentication & Authorization](#authentication--authorization)
9. [API Reference](#api-reference)
10. [Artisan Commands](#artisan-commands)
11. [Usage Examples](#usage-examples)
12. [Testing](#testing)
13. [Production Deployment](#production-deployment)
14. [Troubleshooting](#troubleshooting)

## Overview

The RustAxum Broadcasting System provides real-time communication capabilities similar to Laravel's Broadcasting feature. It enables server-to-client communication through WebSockets, event broadcasting, and multi-driver support for scalable real-time applications.

### Key Features

- **Multi-Driver Architecture**: WebSocket, Redis, and Log drivers
- **Event Integration**: Automatic broadcasting when events are dispatched
- **Channel-Based Communication**: Public, private, and presence channels
- **JWT Authentication**: Secure WebSocket connections
- **Laravel-Compatible API**: Familiar traits and patterns
- **Real-time Monitoring**: Connection and channel statistics
- **Production Ready**: Comprehensive error handling and logging

## Architecture

### Component Overview

```txt
┌─────────────────────────────────────────────────────────────┐
│                    Broadcasting System                      │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │  Event System   │  │  Broadcast Mgr  │  │   Drivers    │ │
│  │                 │  │                 │  │              │ │
│  │ • ShouldBcast   │  │ • Driver Mgmt   │  │ • WebSocket  │ │
│  │ • Auto Dispatch │  │ • Channel Mgmt  │  │ • Redis      │ │
│  │ • Queue Support │  │ • Auth/AuthZ    │  │ • Log        │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
├─────────────────────────────────────────────────────────────┤
│                    WebSocket Layer                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │ Connection Mgmt │  │ Channel Router  │  │ Auth Guard   │ │
│  │                 │  │                 │  │              │ │
│  │ • Client Pool   │  │ • Channel Join  │  │ • JWT Verify │ │
│  │ • Heartbeat     │  │ • Broadcasting  │  │ • Permissions│ │
│  │ • Cleanup       │  │ • Presence      │  │ • Rate Limit │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### File Structure

```txt
src/app/broadcasting/
├── mod.rs                 # Core traits and broadcast manager
├── websocket.rs           # WebSocket implementation
├── helpers.rs             # Convenience functions and macros
└── ...

src/app/events/
├── mod.rs                 # Event system with broadcast integration
└── ...

src/cli/commands/
├── broadcast.rs           # Artisan commands for broadcasting
└── ...

src/config/
├── broadcasting.rs        # Broadcasting configuration
└── ...
```

## Configuration

### Environment Variables

Add these environment variables to your `.env` file:

```env
# Broadcasting Configuration
BROADCAST_DRIVER=websocket
BROADCAST_WEBSOCKET_ENABLED=true
BROADCAST_WEBSOCKET_PORT=8080
BROADCAST_REDIS_ENABLED=false
BROADCAST_REDIS_HOST=localhost
BROADCAST_REDIS_PORT=6379
BROADCAST_REDIS_PASSWORD=
BROADCAST_REDIS_DATABASE=0
BROADCAST_CHANNELS_PREFIX=
```

### Configuration Structure

```rust
// src/config/broadcasting.rs
#[derive(Debug, Clone)]
pub struct BroadcastingConfig {
    pub default_driver: String,
    pub websocket_enabled: bool,
    pub websocket_port: u16,
    pub redis_enabled: bool,
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_password: Option<String>,
    pub redis_database: u8,
    pub channels_prefix: String,
}
```

## Broadcasting Drivers

### WebSocket Driver

The primary driver for real-time browser communication.

**Features:**

- Real-time bidirectional communication
- Channel-based message routing
- JWT authentication support
- Connection management and cleanup
- Automatic reconnection handling

**Use Cases:**

- Real-time notifications
- Live chat applications
- Dashboard updates
- Collaborative editing

### Redis Driver

For distributed systems and horizontal scaling.

**Features:**

- Pub/Sub messaging
- Cross-server broadcasting
- Message persistence options
- Cluster support

**Use Cases:**

- Multi-server deployments
- Microservices communication
- Message queuing
- Event sourcing

### Log Driver

For development and debugging.

**Features:**

- Console output
- File logging
- Message inspection
- Testing support

**Use Cases:**

- Development debugging
- Testing environments
- Message auditing
- Performance monitoring

## Events and Broadcasting

### ShouldBroadcast Trait

Events that implement the `ShouldBroadcast` trait are automatically broadcast when dispatched:

```rust
use crate::app::events::{Event, ShouldBroadcast};
use crate::app::broadcasting::Broadcastable;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegisteredEvent {
    pub user_id: String,
    pub email: String,
    pub name: String,
    // ... other fields
}

impl Event for UserRegisteredEvent {
    fn event_name(&self) -> &'static str {
        "UserRegistered"
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

impl ShouldBroadcast for UserRegisteredEvent {
    fn broadcast_on(&self) -> Vec<String> {
        vec!["user.registered".to_string()]
    }

    fn broadcast_as(&self) -> Option<String> {
        Some("UserRegistered".to_string())
    }

    fn broadcast_with(&self) -> serde_json::Value {
        serde_json::json!({
            "user_id": self.user_id,
            "name": self.name,
            "timestamp": chrono::Utc::now()
        })
    }
}

impl Broadcastable for UserRegisteredEvent {
    fn broadcast_channel(&self) -> String {
        "user.registered".to_string()
    }

    fn broadcast_data(&self) -> serde_json::Value {
        self.broadcast_with()
    }
}
```

### Automatic Broadcasting

When an event is dispatched, the system automatically checks if it implements `ShouldBroadcast` and broadcasts it:

```rust
// Event is automatically broadcast when dispatched
let event = UserRegisteredEvent::new("user123", "user@example.com", "John Doe");
event.dispatch().await?;
```

## WebSocket Implementation

### Connection Flow

1. **Client Connection**: Client connects to WebSocket endpoint
2. **Authentication**: JWT token validation (if required)
3. **Channel Join**: Client joins specified channel
4. **Message Routing**: Messages routed based on channel
5. **Disconnection**: Cleanup and resource deallocation

### WebSocket Endpoints

```txt
GET /ws?channel=general&auth_token=jwt_token
GET /ws/{channel}?auth_token=jwt_token
```

### Client-Side JavaScript Example

```javascript
// Connect to WebSocket
const ws = new WebSocket(
  "ws://localhost:3000/ws?channel=general&auth_token=" + authToken
);

// Handle connection
ws.onopen = function (event) {
  console.log("Connected to WebSocket");
};

// Handle messages
ws.onmessage = function (event) {
  const message = JSON.parse(event.data);
  console.log("Received:", message);

  switch (message.event) {
    case "notification":
      showNotification(message.data);
      break;
    case "UserRegistered":
      handleUserRegistered(message.data);
      break;
  }
};

// Send ping to keep connection alive
setInterval(() => {
  ws.send(
    JSON.stringify({
      action: "ping",
      data: { timestamp: Date.now() },
    })
  );
}, 30000);
```

### Message Format

All WebSocket messages follow this structure:

```json
{
  "channel": "general",
  "event": "notification",
  "data": {
    "title": "Welcome!",
    "message": "You have successfully registered",
    "timestamp": "2023-12-07T10:30:00Z"
  },
  "timestamp": "2023-12-07T10:30:00Z"
}
```

## Channel Management

### Channel Types

#### Public Channels

Open to all users without authentication:

```txt
general
public
announcements
```

#### Private Channels

Require authentication and authorization:

```txt
user.{user_id}
admin.notifications
team.{team_id}
org.{organization_id}
```

#### Presence Channels

Track who is currently in the channel:

```txt
presence.chat.{room_id}
presence.game.{game_id}
```

### Channel Authorization

```rust
// Check if user can access a specific channel
async fn can_access_channel(user_info: &WebSocketUserInfo, channel: &str) -> bool {
    match channel {
        "general" | "public" => true,
        "notifications" => true, // All authenticated users
        "admin" => user_info.roles.contains(&"admin".to_string()),
        _ => {
            // User-specific channels
            if let Some(user_id) = channel.strip_prefix("user.") {
                return user_info.user_id == user_id;
            }

            // Team channels
            if let Some(team_id) = channel.strip_prefix("team.") {
                return user_has_team_access(&user_info.user_id, team_id).await;
            }

            false
        }
    }
}
```

## Authentication & Authorization

### JWT Authentication

WebSocket connections can be secured using JWT tokens:

```rust
// Token validation in WebSocket handler
if let Some(token) = params.auth_token {
    match validate_websocket_token(&token, &channel).await {
        Ok(user_info) => {
            // Connection authorized
        }
        Err(e) => {
            // Connection denied
            return ws.on_upgrade(move |socket| handle_unauthorized_socket(socket));
        }
    }
}
```

### Token Format

JWT tokens should include these claims:

```json
{
  "sub": "user_id",
  "aud": ["websocket"],
  "exp": 1701234567,
  "iat": 1701230967,
  "roles": ["user", "admin"],
  "permissions": ["read_notifications", "admin_channels"]
}
```

## API Reference

### Broadcasting Helpers

#### Direct Broadcasting

```rust
use crate::app::broadcasting::helpers::*;

// Broadcast to specific channel
broadcast_to_channel("general", "announcement", serde_json::json!({
    "message": "System maintenance in 5 minutes"
})).await?;

// Broadcast to user
broadcast_to_user("user123", "notification", serde_json::json!({
    "title": "Welcome!",
    "message": "Registration successful"
})).await?;

// Broadcast to organization
broadcast_to_organization("org456", "update", serde_json::json!({
    "type": "policy_change",
    "message": "New privacy policy available"
})).await?;
```

#### Fluent API

```rust
use crate::app::broadcasting::helpers::broadcast;

// Using fluent interface
broadcast()
    .to_user("user123")
    .event("notification")
    .with(serde_json::json!({
        "title": "Order Status",
        "message": "Your order has been shipped!"
    }))
    .send().await?;

// Multiple channels
broadcast()
    .channels(vec!["admin".to_string(), "moderators".to_string()])
    .event("alert")
    .with(serde_json::json!({
        "level": "warning",
        "message": "High CPU usage detected"
    }))
    .send().await?;
```

#### Facade Pattern

```rust
use crate::app::broadcasting::helpers::BroadcastFacade;

// Laravel-style facade
BroadcastFacade::to_user("user123")
    .event("notification")
    .with(notification_data)
    .send().await?;

BroadcastFacade::system_alert("critical", "Database connection lost", true).await?;
```

### Macros

```rust
// Broadcast macro
broadcast!(event).await?;

// Broadcast to channel
broadcast!("general", "announcement", data).await?;

// Broadcast to user
broadcast_to_user!("user123", "notification", data).await?;

// Notify user
notify_user!("user123", "Welcome", "Registration successful").await?;
notify_user!("user123", "Order Update", "Shipped", "https://track.example.com").await?;
```

## Artisan Commands

### Available Commands

```bash
# Test broadcasting
cargo run --bin artisan -- broadcast test --channel general --message "Hello World"

# Start WebSocket server
cargo run --bin artisan -- broadcast websocket --port 8080

# Show system statistics
cargo run --bin artisan -- broadcast stats

# Send ping messages
cargo run --bin artisan -- broadcast ping --channel general --interval 5

# List active channels
cargo run --bin artisan -- broadcast channels

# Send user notification
cargo run --bin artisan -- broadcast notify:user user123 "Welcome" "Registration successful" --action-url "/dashboard"

# System alert
cargo run --bin artisan -- broadcast system:alert warning "Maintenance starting soon" --action-required

# Monitor activity
cargo run --bin artisan -- broadcast monitor --duration 60
```

### Command Examples

#### Testing Broadcasting

```bash
# Basic test
$ cargo run --bin artisan -- broadcast test
🚀 Testing broadcast to channel: general
✅ Broadcast test completed successfully

# Custom channel and message
$ cargo run --bin artisan -- broadcast test --channel "notifications" --message "Custom test message"
🚀 Testing broadcast to channel: notifications
✅ Broadcast test completed successfully
```

#### WebSocket Server

```bash
$ cargo run --bin artisan -- broadcast websocket --port 8080
🌐 Starting WebSocket server on port 8080
WebSocket server starting on port 8080
```

#### Statistics

```bash
$ cargo run --bin artisan -- broadcast stats
📊 Broadcasting System Statistics
================================
📡 WebSocket Channels:
  • general: 3 connections
  • user.123: 1 connections
  • admin: 2 connections

🔧 Broadcast Configuration:
  • Default Driver: Available
  • Total Active Channels: 3
```

## Usage Examples

### Real-time Notifications

```rust
// Service layer
impl UserService {
    pub async fn register_user(&self, user_data: CreateUserRequest) -> Result<User> {
        // Create user in database
        let user = self.create_user(user_data).await?;

        // Dispatch event (automatically broadcasts)
        UserRegisteredEvent::new(
            user.id.clone(),
            user.email.clone(),
            user.name.clone()
        ).dispatch().await?;

        // Send welcome notification
        BroadcastFacade::notify_user(
            &user.id,
            "Welcome!",
            "Your account has been created successfully",
            Some("/dashboard")
        ).await?;

        Ok(user)
    }
}
```

### Live Chat Implementation

```rust
#[derive(Debug, Clone, Serialize)]
pub struct ChatMessageEvent {
    pub room_id: String,
    pub user_id: String,
    pub username: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

impl Event for ChatMessageEvent {
    fn event_name(&self) -> &'static str { "ChatMessage" }
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

impl Broadcastable for ChatMessageEvent {
    fn broadcast_channel(&self) -> String {
        format!("chat.{}", self.room_id)
    }

    fn broadcast_data(&self) -> serde_json::Value {
        serde_json::json!({
            "user_id": self.user_id,
            "username": self.username,
            "message": self.message,
            "timestamp": self.timestamp
        })
    }
}

// Usage in controller
pub async fn send_message(
    State(pool): State<DbPool>,
    Json(request): Json<SendMessageRequest>
) -> impl IntoResponse {
    // Save message to database
    let message = save_chat_message(&pool, &request).await?;

    // Broadcast to room
    ChatMessageEvent {
        room_id: request.room_id,
        user_id: request.user_id,
        username: request.username,
        message: request.message,
        timestamp: Utc::now(),
    }.dispatch().await?;

    Json(message)
}
```

### System Monitoring

```rust
pub struct SystemMonitorService;

impl SystemMonitorService {
    pub async fn check_system_health(&self) -> Result<()> {
        let cpu_usage = self.get_cpu_usage().await?;
        let memory_usage = self.get_memory_usage().await?;

        if cpu_usage > 80.0 {
            BroadcastFacade::system_alert(
                "warning",
                &format!("High CPU usage: {:.1}%", cpu_usage),
                false
            ).await?;
        }

        if memory_usage > 90.0 {
            BroadcastFacade::system_alert(
                "critical",
                &format!("Critical memory usage: {:.1}%", memory_usage),
                true
            ).await?;
        }

        Ok(())
    }
}
```

### Dashboard Updates

```rust
pub async fn update_dashboard_metrics() -> Result<()> {
    let metrics = collect_metrics().await?;

    // Broadcast to admin dashboard
    broadcast()
        .channel("admin.dashboard")
        .event("metrics_update")
        .with(serde_json::json!({
            "cpu": metrics.cpu_usage,
            "memory": metrics.memory_usage,
            "disk": metrics.disk_usage,
            "active_users": metrics.active_users,
            "requests_per_minute": metrics.requests_per_minute,
            "timestamp": Utc::now()
        }))
        .send().await?;

    Ok(())
}
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::events::EventFacade;

    #[tokio::test]
    async fn test_event_broadcasting() {
        // Enable event faking
        EventFacade::fake().await;

        // Dispatch event
        let event = UserRegisteredEvent::new(
            "test_user".to_string(),
            "test@example.com".to_string(),
            "Test User".to_string()
        );
        event.dispatch().await.unwrap();

        // Assert event was dispatched
        assert!(EventFacade::assert_dispatched("UserRegistered").await);
    }

    #[tokio::test]
    async fn test_broadcast_helper() {
        let result = broadcast_to_channel(
            "test_channel",
            "test_event",
            serde_json::json!({"message": "test"})
        ).await;

        assert!(result.is_ok());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_websocket_connection() {
    let server = create_test_server().await;
    let client = connect_websocket_client(&server.addr).await;

    // Send test message
    client.send_text(r#"{"action": "ping"}"#).await;

    // Receive response
    let response = client.receive_text().await;
    let parsed: serde_json::Value = serde_json::from_str(&response).unwrap();

    assert_eq!(parsed["event"], "pong");
}
```

### Load Testing

```bash
# Using wscat for WebSocket testing
npm install -g wscat

# Connect and test
wscat -c ws://localhost:3000/ws?channel=test

# Send test messages
{"action": "ping", "data": {"test": true}}
```

## Production Deployment

### Docker Configuration

```dockerfile
# Dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rustaxum /usr/local/bin/
COPY --from=builder /app/target/release/artisan /usr/local/bin/
EXPOSE 3000 8080
CMD ["rustaxum"]
```

### Docker Compose

```yaml
# docker-compose.prod.yml
version: "3.8"
services:
  app:
    build: .
    ports:
      - "3000:3000"
      - "8080:8080"
    environment:
      - BROADCAST_DRIVER=redis
      - BROADCAST_REDIS_HOST=redis
      - BROADCAST_WEBSOCKET_ENABLED=true
    depends_on:
      - redis
      - postgres

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - app
```

### Nginx Configuration

```nginx
# nginx.conf
upstream app {
    server app:3000;
}

upstream websocket {
    server app:8080;
}

server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://app;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /ws {
        proxy_pass http://websocket;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

### Environment Configuration

```env
# Production environment
RUST_ENV=production
RUST_LOG=info

# Broadcasting
BROADCAST_DRIVER=redis
BROADCAST_REDIS_HOST=redis
BROADCAST_REDIS_PORT=6379
BROADCAST_REDIS_PASSWORD=your_redis_password
BROADCAST_WEBSOCKET_ENABLED=true
BROADCAST_WEBSOCKET_PORT=8080

# Security
JWT_SECRET=your_very_secure_jwt_secret_here
CORS_ALLOWED_ORIGINS=https://your-domain.com

# Database
DATABASE_URL=postgresql://username:password@postgres:5432/database
```

### Monitoring

```bash
# Monitor broadcasting stats
cargo run --bin artisan -- broadcast stats

# Monitor system activity
cargo run --bin artisan -- broadcast monitor --duration 300

# Check active connections
curl http://localhost:3000/api/broadcast/stats
```

### Scaling Considerations

1. **Horizontal Scaling**: Use Redis driver for multi-instance deployments
2. **Load Balancing**: Sticky sessions for WebSocket connections
3. **Resource Limits**: Configure connection limits and timeouts
4. **Monitoring**: Set up metrics and alerting
5. **SSL/TLS**: Enable secure WebSocket connections (WSS)

## Troubleshooting

### Common Issues

#### WebSocket Connection Fails

**Problem**: Clients cannot connect to WebSocket

```txt
WebSocket connection failed: Error 403 Forbidden
```

**Solutions**:

1. Check JWT token validity
2. Verify channel permissions
3. Check CORS configuration
4. Ensure WebSocket server is running

#### Broadcasting Not Working

**Problem**: Events are dispatched but not broadcast

```txt
Event UserRegistered was dispatched but no broadcast received
```

**Solutions**:

1. Verify event implements `Broadcastable` trait
2. Check broadcasting driver configuration
3. Ensure WebSocket clients are connected
4. Review event dispatcher integration

#### High Memory Usage

**Problem**: WebSocket connections consuming too much memory

```txt
Memory usage increasing with each connection
```

**Solutions**:

1. Implement connection limits
2. Add proper cleanup on disconnect
3. Configure heartbeat/ping timeouts
4. Monitor and restart if needed

#### Redis Connection Issues

**Problem**: Redis driver not working

```txt
Failed to connect to Redis: Connection refused
```

**Solutions**:

1. Verify Redis server is running
2. Check connection credentials
3. Test network connectivity
4. Review firewall settings

### Debug Commands

```bash
# Test WebSocket connectivity
wscat -c ws://localhost:3000/ws?channel=test

# Check broadcasting configuration
cargo run --bin artisan -- broadcast stats

# Monitor real-time activity
cargo run --bin artisan -- broadcast monitor --duration 60

# Test specific channel broadcasting
cargo run --bin artisan -- broadcast test --channel "user.123" --message "Debug test"
```

### Logging

Enable detailed logging for debugging:

```env
RUST_LOG=debug,rustaxum::app::broadcasting=trace
```

### Performance Monitoring

```rust
// Add metrics collection
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref WEBSOCKET_CONNECTIONS: Counter = register_counter!(
        "websocket_connections_total",
        "Total number of WebSocket connections"
    ).unwrap();

    static ref BROADCAST_DURATION: Histogram = register_histogram!(
        "broadcast_duration_seconds",
        "Time spent broadcasting messages"
    ).unwrap();
}
```

## Advanced Features

### Custom Drivers

Implement custom broadcasting drivers:

```rust
use async_trait::async_trait;
use crate::app::broadcasting::BroadcastDriver;

pub struct CustomDriver {
    // Driver-specific configuration
}

#[async_trait]
impl BroadcastDriver for CustomDriver {
    async fn broadcast(&self, channel: &str, data: serde_json::Value) -> Result<()> {
        // Custom broadcasting logic
        Ok(())
    }

    async fn broadcast_private(&self, channel: &str, data: serde_json::Value) -> Result<()> {
        // Custom private broadcasting logic
        Ok(())
    }

    fn driver_name(&self) -> &'static str {
        "custom"
    }
}
```

### Message Queuing

Integrate with job queues for reliable delivery:

```rust
impl ShouldQueue for BroadcastableEvent {
    fn queue_connection(&self) -> Option<&str> {
        Some("broadcasts")
    }

    fn queue(&self) -> Option<&str> {
        Some("high_priority")
    }

    fn tries(&self) -> Option<u32> {
        Some(3)
    }

    fn timeout(&self) -> Option<chrono::Duration> {
        Some(chrono::Duration::minutes(5))
    }
}
```

### Presence Channels

Track online users:

```rust
pub struct PresenceChannel {
    users: Arc<RwLock<HashMap<String, UserInfo>>>,
}

impl PresenceChannel {
    pub async fn join(&self, user_id: String, user_info: UserInfo) {
        let mut users = self.users.write().await;
        users.insert(user_id, user_info);

        // Broadcast user joined
        self.broadcast_presence_update().await;
    }

    pub async fn leave(&self, user_id: &str) {
        let mut users = self.users.write().await;
        users.remove(user_id);

        // Broadcast user left
        self.broadcast_presence_update().await;
    }
}
```

---

## Contributing

When contributing to the broadcasting system:

1. **Follow Patterns**: Use existing traits and patterns
2. **Add Tests**: Include unit and integration tests
3. **Document Changes**: Update this documentation
4. **Performance**: Consider impact on real-time performance
5. **Security**: Review authentication and authorization

## License

This broadcasting system is part of the RustAxum framework and follows the same license terms.

---

_Last updated: 26 September 2025_
