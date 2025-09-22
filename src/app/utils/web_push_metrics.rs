use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Web push metrics and monitoring system
#[derive(Debug, Clone)]
pub struct WebPushMetrics {
    data: Arc<RwLock<MetricsData>>,
}

#[derive(Debug, Default)]
struct MetricsData {
    total_notifications_sent: u64,
    total_notifications_failed: u64,
    total_subscriptions_created: u64,
    total_subscriptions_deleted: u64,
    response_times: Vec<Duration>,
    error_counts: HashMap<String, u64>,
    status_code_counts: HashMap<u16, u64>,
    daily_stats: HashMap<String, DailyStats>, // Date -> Stats
    hourly_stats: HashMap<String, HourlyStats>, // Hour -> Stats
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,
    pub notifications_sent: u64,
    pub notifications_failed: u64,
    pub subscriptions_created: u64,
    pub subscriptions_deleted: u64,
    pub average_response_time_ms: f64,
    pub success_rate: f64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct HourlyStats {
    pub hour: String,
    pub notifications_sent: u64,
    pub notifications_failed: u64,
    pub peak_requests_per_minute: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebPushStatsSnapshot {
    pub total_notifications_sent: u64,
    pub total_notifications_failed: u64,
    pub total_subscriptions_created: u64,
    pub total_subscriptions_deleted: u64,
    pub success_rate: f64,
    pub average_response_time_ms: f64,
    pub last_24h_notifications: u64,
    pub last_24h_failures: u64,
    pub top_error_types: Vec<(String, u64)>,
    pub status_code_distribution: HashMap<u16, u64>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl WebPushMetrics {
    /// Create a new metrics instance
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(MetricsData::default())),
        }
    }

    /// Record a successful notification send
    pub async fn record_notification_success(&self, response_time: Duration) {
        let mut data = self.data.write().await;
        data.total_notifications_sent += 1;
        data.response_times.push(response_time);

        // Keep only last 1000 response times to prevent memory growth
        if data.response_times.len() > 1000 {
            data.response_times.drain(0..100);
        }

        self.update_daily_stats(&mut data, true, response_time).await;
        self.update_hourly_stats(&mut data, true).await;
    }

    /// Record a failed notification send
    pub async fn record_notification_failure(&self, error_type: &str, status_code: Option<u16>) {
        let mut data = self.data.write().await;
        data.total_notifications_failed += 1;

        // Count error types
        *data.error_counts.entry(error_type.to_string()).or_insert(0) += 1;

        // Count status codes if available
        if let Some(code) = status_code {
            *data.status_code_counts.entry(code).or_insert(0) += 1;
        }

        self.update_daily_stats(&mut data, false, Duration::from_millis(0)).await;
        self.update_hourly_stats(&mut data, false).await;
    }

    /// Record subscription creation
    pub async fn record_subscription_created(&self) {
        let mut data = self.data.write().await;
        data.total_subscriptions_created += 1;

        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let daily_stats = data.daily_stats.entry(today.clone()).or_default();
        daily_stats.date = today;
        daily_stats.subscriptions_created += 1;
    }

    /// Record subscription deletion
    pub async fn record_subscription_deleted(&self) {
        let mut data = self.data.write().await;
        data.total_subscriptions_deleted += 1;

        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let daily_stats = data.daily_stats.entry(today.clone()).or_default();
        daily_stats.date = today;
        daily_stats.subscriptions_deleted += 1;
    }

    /// Get current metrics snapshot
    pub async fn get_snapshot(&self) -> WebPushStatsSnapshot {
        let data = self.data.read().await;

        let total_notifications = data.total_notifications_sent + data.total_notifications_failed;
        let success_rate = if total_notifications > 0 {
            (data.total_notifications_sent as f64 / total_notifications as f64) * 100.0
        } else {
            0.0
        };

        let average_response_time_ms = if !data.response_times.is_empty() {
            data.response_times.iter().map(|d| d.as_millis() as f64).sum::<f64>()
                / data.response_times.len() as f64
        } else {
            0.0
        };

        // Calculate last 24h stats
        let yesterday = chrono::Utc::now() - chrono::Duration::days(1);
        let yesterday_str = yesterday.format("%Y-%m-%d").to_string();
        let today_str = chrono::Utc::now().format("%Y-%m-%d").to_string();

        let last_24h_notifications = [yesterday_str.as_str(), today_str.as_str()]
            .iter()
            .filter_map(|date| data.daily_stats.get(*date))
            .map(|stats| stats.notifications_sent)
            .sum();

        let last_24h_failures = [yesterday_str.as_str(), today_str.as_str()]
            .iter()
            .filter_map(|date| data.daily_stats.get(*date))
            .map(|stats| stats.notifications_failed)
            .sum();

        // Get top error types
        let mut top_errors: Vec<(String, u64)> = data.error_counts.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        top_errors.sort_by(|a, b| b.1.cmp(&a.1));
        top_errors.truncate(10);

        WebPushStatsSnapshot {
            total_notifications_sent: data.total_notifications_sent,
            total_notifications_failed: data.total_notifications_failed,
            total_subscriptions_created: data.total_subscriptions_created,
            total_subscriptions_deleted: data.total_subscriptions_deleted,
            success_rate,
            average_response_time_ms,
            last_24h_notifications,
            last_24h_failures,
            top_error_types: top_errors,
            status_code_distribution: data.status_code_counts.clone(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get daily statistics for the last N days
    pub async fn get_daily_stats(&self, days: usize) -> Vec<DailyStats> {
        let data = self.data.read().await;
        let mut stats = Vec::new();

        for i in 0..days {
            let date = (chrono::Utc::now() - chrono::Duration::days(i as i64))
                .format("%Y-%m-%d")
                .to_string();

            if let Some(daily_stat) = data.daily_stats.get(&date) {
                stats.push(daily_stat.clone());
            } else {
                stats.push(DailyStats {
                    date,
                    ..Default::default()
                });
            }
        }

        stats.reverse(); // Oldest first
        stats
    }

    /// Clean up old metrics data to prevent memory growth
    pub async fn cleanup_old_data(&self) {
        let mut data = self.data.write().await;
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(30);

        // Remove daily stats older than 30 days
        data.daily_stats.retain(|date_str, _| {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                let date_time = date.and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_local_timezone(chrono::Utc)
                    .unwrap();
                date_time > cutoff_date
            } else {
                false
            }
        });

        // Remove hourly stats older than 7 days
        let cutoff_hour = chrono::Utc::now() - chrono::Duration::days(7);
        data.hourly_stats.retain(|hour_str, _| {
            if let Ok(hour) = chrono::DateTime::parse_from_rfc3339(hour_str) {
                hour.with_timezone(&chrono::Utc) > cutoff_hour
            } else {
                false
            }
        });
    }

    async fn update_daily_stats(&self, data: &mut MetricsData, success: bool, _response_time: Duration) {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let daily_stats = data.daily_stats.entry(today.clone()).or_default();
        daily_stats.date = today;

        if success {
            daily_stats.notifications_sent += 1;
        } else {
            daily_stats.notifications_failed += 1;
        }

        // Update success rate
        let total = daily_stats.notifications_sent + daily_stats.notifications_failed;
        if total > 0 {
            daily_stats.success_rate = (daily_stats.notifications_sent as f64 / total as f64) * 100.0;
        }

        // Update average response time
        if success && !data.response_times.is_empty() {
            daily_stats.average_response_time_ms = data.response_times.iter()
                .map(|d| d.as_millis() as f64)
                .sum::<f64>() / data.response_times.len() as f64;
        }
    }

    async fn update_hourly_stats(&self, data: &mut MetricsData, success: bool) {
        let now = chrono::Utc::now();
        let hour_key = now.format("%Y-%m-%dT%H:00:00Z").to_string();
        let hourly_stats = data.hourly_stats.entry(hour_key.clone()).or_default();
        hourly_stats.hour = hour_key;

        if success {
            hourly_stats.notifications_sent += 1;
        } else {
            hourly_stats.notifications_failed += 1;
        }
    }
}

impl Default for WebPushMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Global metrics instance (singleton pattern)
static METRICS: std::sync::OnceLock<WebPushMetrics> = std::sync::OnceLock::new();

/// Get the global metrics instance
pub fn get_metrics() -> &'static WebPushMetrics {
    METRICS.get_or_init(WebPushMetrics::new)
}

/// Initialize metrics system
pub fn init_metrics() -> &'static WebPushMetrics {
    get_metrics()
}

/// Helper functions for common metric operations
pub async fn record_success(response_time: Duration) {
    get_metrics().record_notification_success(response_time).await;
}

pub async fn record_failure(error_type: &str, status_code: Option<u16>) {
    get_metrics().record_notification_failure(error_type, status_code).await;
}

pub async fn record_subscription_created() {
    get_metrics().record_subscription_created().await;
}

pub async fn record_subscription_deleted() {
    get_metrics().record_subscription_deleted().await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_metrics_recording() {
        let metrics = WebPushMetrics::new();

        // Record some success
        metrics.record_notification_success(Duration::from_millis(150)).await;
        metrics.record_notification_success(Duration::from_millis(200)).await;

        // Record some failures
        metrics.record_notification_failure("timeout", Some(408)).await;
        metrics.record_notification_failure("auth_failed", Some(401)).await;

        let snapshot = metrics.get_snapshot().await;

        assert_eq!(snapshot.total_notifications_sent, 2);
        assert_eq!(snapshot.total_notifications_failed, 2);
        assert_eq!(snapshot.success_rate, 50.0);
        assert!(snapshot.average_response_time_ms > 0.0);
        assert_eq!(snapshot.top_error_types.len(), 2);
    }

    #[tokio::test]
    async fn test_subscription_tracking() {
        let metrics = WebPushMetrics::new();

        metrics.record_subscription_created().await;
        metrics.record_subscription_created().await;
        metrics.record_subscription_deleted().await;

        let snapshot = metrics.get_snapshot().await;

        assert_eq!(snapshot.total_subscriptions_created, 2);
        assert_eq!(snapshot.total_subscriptions_deleted, 1);
    }

    #[tokio::test]
    async fn test_daily_stats() {
        let metrics = WebPushMetrics::new();

        metrics.record_notification_success(Duration::from_millis(100)).await;
        metrics.record_notification_failure("test", None).await;

        let daily_stats = metrics.get_daily_stats(1).await;

        assert_eq!(daily_stats.len(), 1);
        assert_eq!(daily_stats[0].notifications_sent, 1);
        assert_eq!(daily_stats[0].notifications_failed, 1);
        assert_eq!(daily_stats[0].success_rate, 50.0);
    }
}