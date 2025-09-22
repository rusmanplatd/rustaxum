use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Rate limiting error
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded. Try again in {seconds} seconds")]
    Exceeded { seconds: u64 },
    #[error("Internal rate limiter error: {0}")]
    Internal(String),
}

/// Rate limiter bucket for tracking requests
#[derive(Debug, Clone)]
struct Bucket {
    tokens: u32,
    last_refill: Instant,
    max_tokens: u32,
    refill_rate: Duration,
}

impl Bucket {
    fn new(max_tokens: u32, refill_rate: Duration) -> Self {
        Self {
            tokens: max_tokens,
            last_refill: Instant::now(),
            max_tokens,
            refill_rate,
        }
    }

    fn try_consume(&mut self, tokens: u32) -> Result<(), RateLimitError> {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            Ok(())
        } else {
            let wait_time = self.refill_rate.as_secs();
            Err(RateLimitError::Exceeded { seconds: wait_time })
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);

        if elapsed >= self.refill_rate {
            let tokens_to_add = (elapsed.as_secs() / self.refill_rate.as_secs()) as u32;
            self.tokens = std::cmp::min(self.max_tokens, self.tokens + tokens_to_add);
            self.last_refill = now;
        }
    }
}

/// Token bucket rate limiter implementation
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
    max_tokens: u32,
    refill_rate: Duration,
    cleanup_interval: Duration,
    last_cleanup: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `max_requests` - Maximum number of requests allowed
    /// * `window` - Time window for the rate limit
    ///
    /// # Examples
    /// ```
    /// use std::time::Duration;
    /// use rustaxum::app::utils::RateLimiter;
    ///
    /// // Allow 10 requests per minute
    /// let limiter = RateLimiter::new(10, Duration::from_secs(60));
    /// ```
    pub fn new(max_requests: u32, window: Duration) -> Self {
        let refill_rate = Duration::from_secs(window.as_secs() / max_requests as u64);

        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            max_tokens: max_requests,
            refill_rate,
            cleanup_interval: Duration::from_secs(3600), // Cleanup every hour
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Check if a request is allowed for the given identifier
    pub fn check_rate_limit(&self, identifier: &str) -> Result<(), RateLimitError> {
        self.check_rate_limit_with_cost(identifier, 1)
    }

    /// Check rate limit with custom cost (number of tokens to consume)
    pub fn check_rate_limit_with_cost(&self, identifier: &str, cost: u32) -> Result<(), RateLimitError> {
        // Perform cleanup if needed
        self.cleanup_old_buckets();

        let mut buckets = self.buckets.lock()
            .map_err(|e| RateLimitError::Internal(format!("Mutex lock failed: {}", e)))?;

        let bucket = buckets
            .entry(identifier.to_string())
            .or_insert_with(|| Bucket::new(self.max_tokens, self.refill_rate));

        bucket.try_consume(cost)
    }

    /// Get current token count for an identifier (useful for debugging)
    pub fn get_tokens(&self, identifier: &str) -> u32 {
        if let Ok(mut buckets) = self.buckets.lock() {
            if let Some(bucket) = buckets.get_mut(identifier) {
                bucket.refill();
                return bucket.tokens;
            }
        }
        self.max_tokens // Return max if not found or error
    }

    /// Reset rate limit for a specific identifier
    pub fn reset(&self, identifier: &str) -> Result<()> {
        let mut buckets = self.buckets.lock()
            .map_err(|e| anyhow::anyhow!("Mutex lock failed: {}", e))?;

        if let Some(bucket) = buckets.get_mut(identifier) {
            bucket.tokens = bucket.max_tokens;
            bucket.last_refill = Instant::now();
        }

        Ok(())
    }

    /// Clean up old buckets that haven't been used recently
    fn cleanup_old_buckets(&self) {
        let mut last_cleanup = match self.last_cleanup.lock() {
            Ok(guard) => guard,
            Err(_) => return, // Skip cleanup if mutex is poisoned
        };

        let now = Instant::now();
        if now.duration_since(*last_cleanup) < self.cleanup_interval {
            return; // Not time for cleanup yet
        }

        if let Ok(mut buckets) = self.buckets.lock() {
            let cutoff = now - self.cleanup_interval;
            buckets.retain(|_, bucket| bucket.last_refill > cutoff);
        }

        *last_cleanup = now;
    }

    /// Get statistics about the rate limiter
    pub fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();

        if let Ok(buckets) = self.buckets.lock() {
            stats.insert("total_buckets".to_string(), serde_json::json!(buckets.len()));
            stats.insert("max_tokens".to_string(), serde_json::json!(self.max_tokens));
            stats.insert("refill_rate_secs".to_string(), serde_json::json!(self.refill_rate.as_secs()));
        }

        stats
    }
}

/// Predefined rate limiters for common use cases
impl RateLimiter {
    /// Create a rate limiter for web push subscriptions (10 per hour per user)
    pub fn for_web_push_subscription() -> Self {
        Self::new(10, Duration::from_secs(3600))
    }

    /// Create a rate limiter for sending notifications (100 per hour per user)
    pub fn for_notification_sending() -> Self {
        Self::new(100, Duration::from_secs(3600))
    }

    /// Create a rate limiter for API endpoints (60 per minute per IP)
    pub fn for_api_endpoints() -> Self {
        Self::new(60, Duration::from_secs(60))
    }

    /// Create a rate limiter for authentication attempts (5 per minute per IP)
    pub fn for_auth_attempts() -> Self {
        Self::new(5, Duration::from_secs(60))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_rate_limiting() {
        let limiter = RateLimiter::new(3, Duration::from_secs(60));

        // First 3 requests should succeed
        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_ok());

        // 4th request should fail
        assert!(limiter.check_rate_limit("user1").is_err());
    }

    #[test]
    fn test_different_identifiers() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));

        // Different users should have separate buckets
        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_err());

        // user2 should still have their full quota
        assert!(limiter.check_rate_limit("user2").is_ok());
        assert!(limiter.check_rate_limit("user2").is_ok());
        assert!(limiter.check_rate_limit("user2").is_err());
    }

    #[test]
    fn test_token_cost() {
        let limiter = RateLimiter::new(5, Duration::from_secs(60));

        // Consume 3 tokens at once
        assert!(limiter.check_rate_limit_with_cost("user1", 3).is_ok());

        // Should have 2 tokens left
        assert_eq!(limiter.get_tokens("user1"), 2);

        // Should be able to consume 2 more
        assert!(limiter.check_rate_limit_with_cost("user1", 2).is_ok());

        // Should be out of tokens now
        assert!(limiter.check_rate_limit("user1").is_err());
    }

    #[test]
    fn test_reset() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));

        // Exhaust the bucket
        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_err());

        // Reset and try again
        limiter.reset("user1").unwrap();
        assert!(limiter.check_rate_limit("user1").is_ok());
        assert!(limiter.check_rate_limit("user1").is_ok());
    }

    #[test]
    fn test_predefined_limiters() {
        let web_push_limiter = RateLimiter::for_web_push_subscription();
        let api_limiter = RateLimiter::for_api_endpoints();

        // Test that they're created with different limits
        assert_ne!(web_push_limiter.max_tokens, api_limiter.max_tokens);
    }
}