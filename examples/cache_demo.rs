use rustaxum::cache::drivers::MemoryCache;
use rustaxum::cache::manager::{CacheDriver, CacheFacade};
use rustaxum::cache::Cache;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
struct User {
    id: i32,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Cache Demo - Laravel-style caching in Rust");
    println!("===============================================\n");

    // Create a memory cache instance
    let memory_cache = Arc::new(MemoryCache::new(Some("demo".to_string())));
    let cache = CacheDriver::Memory(memory_cache);

    // Example user
    let user = User {
        id: 1,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };

    println!("1. Testing basic cache operations:");
    println!("  📝 Storing user in cache...");

    // Store user with 1 hour TTL
    cache.put("user:1", &user, Some(Duration::from_secs(3600))).await?;

    // Retrieve user from cache
    let cached_user: Option<User> = cache.get("user:1").await?;
    println!("  ✅ Retrieved from cache: {:?}", cached_user);

    // Check if key exists
    let exists = cache.has("user:1").await?;
    println!("  🔍 Key exists: {}", exists);

    println!("\n2. Testing remember functionality:");

    // Remember pattern - get from cache or compute and store
    let user_from_remember = cache.remember(
        "user:2",
        Some(Duration::from_secs(3600)),
        || async {
            println!("  🔄 Loading user from 'database'...");
            tokio::time::sleep(Duration::from_millis(100)).await; // Simulate DB query
            Ok(User {
                id: 2,
                name: "Jane Smith".to_string(),
                email: "jane@example.com".to_string(),
            })
        }
    ).await?;

    println!("  ✅ User from remember: {:?}", user_from_remember);

    // Second call should return from cache (no database load)
    println!("  🔄 Calling remember again (should use cache)...");
    let cached_user_2 = cache.remember(
        "user:2",
        Some(Duration::from_secs(3600)),
        || async {
            println!("  ❌ This should not print - using cache!");
            Ok(User {
                id: 2,
                name: "Should not see this".to_string(),
                email: "nope@example.com".to_string(),
            })
        }
    ).await?;

    println!("  ✅ Second call result: {:?}", cached_user_2);

    println!("\n3. Testing increment operations:");

    // Increment operations
    let count1 = cache.increment("login_count", 1).await?;
    println!("  🔢 First increment: {}", count1);

    let count2 = cache.increment("login_count", 5).await?;
    println!("  🔢 Second increment (+5): {}", count2);

    let count3 = cache.decrement("login_count", 2).await?;
    println!("  🔢 Decrement (-2): {}", count3);

    println!("\n4. Testing multiple operations:");

    // Store multiple users
    let users = vec![
        ("user:3", &User { id: 3, name: "Bob".to_string(), email: "bob@example.com".to_string() }),
        ("user:4", &User { id: 4, name: "Alice".to_string(), email: "alice@example.com".to_string() }),
    ];

    cache.put_many(&users, Some(Duration::from_secs(3600))).await?;
    println!("  📝 Stored multiple users");

    // Retrieve multiple users
    let keys = vec!["user:3", "user:4", "user:5"]; // user:5 doesn't exist
    let results: Vec<(String, Option<User>)> = cache.many(&keys).await?;

    for (key, user_opt) in results {
        match user_opt {
            Some(user) => println!("  ✅ {}: {:?}", key, user),
            None => println!("  ❌ {}: Not found", key),
        }
    }

    println!("\n5. Testing add operation (only if key doesn't exist):");

    let added1 = cache.add("exclusive_key", &"first_value", Some(Duration::from_secs(3600))).await?;
    println!("  ➕ First add: {} (should be true)", added1);

    let added2 = cache.add("exclusive_key", &"second_value", Some(Duration::from_secs(3600))).await?;
    println!("  ➕ Second add: {} (should be false)", added2);

    let value: Option<String> = cache.get("exclusive_key").await?;
    println!("  🔍 Final value: {:?}", value);

    println!("\n6. Testing cache cleanup:");

    // Forget specific key
    let forgotten = cache.forget("user:1").await?;
    println!("  🗑️  Forgot user:1: {}", forgotten);

    let exists_after_forget = cache.has("user:1").await?;
    println!("  🔍 user:1 exists after forget: {}", exists_after_forget);

    // Flush all cache
    cache.flush().await?;
    println!("  🧹 Flushed all cache");

    let exists_after_flush = cache.has("user:2").await?;
    println!("  🔍 user:2 exists after flush: {}", exists_after_flush);

    println!("\n✨ Cache demo completed successfully!");
    println!("\n📚 Laravel-style methods available:");
    println!("  • cache.get(key) - Retrieve value");
    println!("  • cache.put(key, value, ttl) - Store value");
    println!("  • cache.forever(key, value) - Store permanently");
    println!("  • cache.remember(key, ttl, closure) - Get or compute & store");
    println!("  • cache.remember_forever(key, closure) - Get or compute & store forever");
    println!("  • cache.has(key) - Check if key exists");
    println!("  • cache.forget(key) - Remove key");
    println!("  • cache.flush() - Clear all cache");
    println!("  • cache.many(keys) - Get multiple values");
    println!("  • cache.put_many(values, ttl) - Store multiple values");
    println!("  • cache.increment(key, value) - Increment numeric value");
    println!("  • cache.decrement(key, value) - Decrement numeric value");
    println!("  • cache.add(key, value, ttl) - Add only if key doesn't exist");

    Ok(())
}