# Laravel-style Cache System for Rust

This implementation provides a Laravel-inspired cache system with support for multiple drivers, async operations, and familiar Laravel methods.

## Features

- **Multiple Drivers**: Memory and Redis cache drivers
- **Laravel-style API**: Familiar methods like `put()`, `get()`, `remember()`, `forget()`, etc.
- **Async/Await**: Full async support throughout
- **JSON Serialization**: Automatic serialization/deserialization with serde
- **TTL Support**: Time-to-live for cache entries
- **Batch Operations**: Store/retrieve multiple values at once
- **Increment/Decrement**: Atomic counter operations
- **Facade Pattern**: Easy global access with `CacheFacade`

## Configuration

Add to your `.env` file:

```env
# Cache configuration
CACHE_DRIVER=redis          # or "memory"
CACHE_PREFIX=myapp
REDIS_HOST=redis
REDIS_PORT=6379
REDIS_PASSWORD=
REDIS_DATABASE=0
```

## Usage Examples

### Basic Cache Operations

```rust
use crate::cache::{Cache, manager::{CacheDriver, CacheFacade}};
use serde::{Serialize, Deserialize};
use std::time::Duration;

#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    email: String,
}

// Using facade for simple operations
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let user = User {
        id: 1,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };

    // Store with 1 hour TTL
    CacheFacade::put("user:1", &user, Some(Duration::from_secs(3600))).await?;

    // Retrieve from cache
    let cached_user: Option<User> = CacheFacade::get("user:1").await?;

    // Check if exists
    let exists = CacheFacade::has("user:1").await?;

    // Remove from cache
    let removed = CacheFacade::forget("user:1").await?;

    Ok(())
}
```

### Remember Pattern (Laravel-style)

```rust
// Get from cache or compute and store if not found
let user = CacheFacade::remember(
    "user:123",
    Some(Duration::from_secs(3600)),
    || async {
        // This closure only runs if the key is not in cache
        database::load_user(123).await
    }
).await?;

// Store forever (no TTL)
let user = CacheFacade::remember_forever(
    "user:123",
    || async {
        database::load_user(123).await
    }
).await?;
```

### Using Specific Cache Stores

```rust
use crate::cache::manager::cache;

// Use specific cache store
let redis_cache = cache("redis").await?;
redis_cache.put("key", &value, Some(Duration::from_secs(300))).await?;

let memory_cache = cache("memory").await?;
memory_cache.put("key", &value, None).await?; // No TTL
```

### Increment/Decrement Operations

```rust
// Increment login counter
let count = CacheFacade::increment("login_count", 1).await?;

// Decrement with custom value
let remaining = CacheFacade::decrement("api_calls", 5).await?;
```

### Batch Operations

```rust
// Store multiple values
let users = vec![
    ("user:1", &user1),
    ("user:2", &user2),
];
CacheFacade::put_many(&users, Some(Duration::from_secs(3600))).await?;

// Get multiple values
let keys = vec!["user:1", "user:2", "user:3"];
let results: Vec<(String, Option<User>)> = CacheFacade::many(&keys).await?;

for (key, user_opt) in results {
    match user_opt {
        Some(user) => println!("Found {}: {:?}", key, user),
        None => println!("Key {} not found", key),
    }
}
```

### Add Operation (Only if Key Doesn't Exist)

```rust
// Try to acquire a lock
let acquired = CacheFacade::add(
    "processing_lock",
    &"locked",
    Some(Duration::from_secs(300))
).await?;

if acquired {
    // We got the lock, proceed with processing
    process_data().await?;
    CacheFacade::forget("processing_lock").await?;
} else {
    // Someone else is already processing
    println!("Already being processed");
}
```

## Available Methods

### Core Cache Methods

- `get<T>(key)` - Retrieve a value
- `put<T>(key, value, ttl)` - Store a value with optional TTL
- `forever<T>(key, value)` - Store a value permanently
- `has(key)` - Check if key exists
- `forget(key)` - Remove a key
- `flush()` - Clear all cache

### Laravel-style Remember Methods

- `remember<T>(key, ttl, closure)` - Get from cache or compute and store
- `remember_forever<T>(key, closure)` - Same but store permanently

### Batch Operations

- `many<T>(keys)` - Get multiple values
- `put_many<T>(values, ttl)` - Store multiple values

### Numeric Operations

- `increment(key, value)` - Increment a numeric value
- `decrement(key, value)` - Decrement a numeric value

### Conditional Operations

- `add<T>(key, value, ttl)` - Add only if key doesn't exist

## Cache Drivers

### Memory Cache

- Fast in-memory storage
- Automatic cleanup of expired entries
- Perfect for single-instance applications
- Data lost on application restart

### Redis Cache

- Persistent storage
- Shared between application instances
- Network-based (slightly slower than memory)
- Survives application restarts

## Error Handling

All cache operations return `Result<T, anyhow::Error>`. Common error types:

- `CacheError::Serialization` - JSON serialization failed
- `CacheError::Deserialization` - JSON deserialization failed
- `CacheError::Connection` - Redis connection issues
- `CacheError::Operation` - Cache operation failed

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CACHE_DRIVER` | `memory` | Cache driver to use |
| `CACHE_PREFIX` | - | Key prefix for all cache operations |
| `REDIS_HOST` | `127.0.0.1` | Redis host |
| `REDIS_PORT` | `6379` | Redis port |
| `REDIS_PASSWORD` | - | Redis password |
| `REDIS_DATABASE` | `0` | Redis database number |
| `REDIS_URL` | - | Full Redis URL (overrides individual settings) |

## Docker Setup

The cache system works out of the box with the provided Docker Compose setup. Redis is automatically configured and available.

## Testing

The cache system includes comprehensive tests for both memory and Redis drivers. Run tests with:

```bash
cargo test cache
```

Note: Redis tests require a running Redis instance. Use Docker Compose for development:

```bash
docker-compose up -d redis
cargo test cache
```