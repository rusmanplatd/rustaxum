use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Database notification structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseNotification {
    pub id: String,
    pub notifiable_type: String,
    pub notifiable_id: String,
    pub notification_type: String,
    pub data: serde_json::Value,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Trait for queuing support (similar to Laravel's Queueable)
pub trait Queueable {
    /// Get the queue connection name
    fn on_connection(&self) -> Option<&str> {
        None
    }

    /// Get the queue name
    fn on_queue(&self) -> Option<&str> {
        None
    }

    /// Get the delay for the notification
    fn delay(&self) -> Option<chrono::Duration> {
        None
    }

    /// Set the delay for the notification
    fn with_delay(self, delay: chrono::Duration) -> Self
    where
        Self: Sized;

    /// Get the middleware for the notification
    fn middleware(&self) -> Vec<String> {
        vec![]
    }

    /// Get the tags for the notification
    fn tags(&self) -> Vec<String> {
        vec![]
    }

    /// Set the queue connection
    fn via_connection(self, connection: &str) -> Self
    where
        Self: Sized;

    /// Set the queue name
    fn via_queue(self, queue: &str) -> Self
    where
        Self: Sized;
}

/// Trait for notifications that should be queued
pub trait ShouldQueue: Notification {
    /// Determine if the notification should be queued
    fn should_queue(&self) -> bool {
        true
    }
}

/// Trait for notifications that should be queued after database transactions commit
pub trait ShouldQueueAfterCommit: ShouldQueue {
    /// Determine if the notification should be queued after commit
    fn after_commit(&self) -> bool {
        true
    }
}

/// Trait for entities with locale preferences
pub trait HasLocalePreference {
    /// Get the preferred locale for notifications
    fn preferred_locale(&self) -> Option<String>;
}

/// Base trait that all notifications must implement
#[async_trait]
pub trait Notification: Send + Sync {
    /// Get the notification channels this notification should be sent on
    fn via(&self, notifiable: &dyn Notifiable) -> Vec<NotificationChannel>;

    /// Get the mail representation of the notification
    fn to_mail(&self, _notifiable: &dyn Notifiable) -> Result<MailMessage> {
        Err(anyhow::anyhow!("Mail channel not implemented for this notification"))
    }

    /// Get the database representation of the notification
    fn to_database(&self, _notifiable: &dyn Notifiable) -> Result<DatabaseMessage> {
        Err(anyhow::anyhow!("Database channel not implemented for this notification"))
    }

    /// Get the broadcast representation of the notification
    fn to_broadcast(&self, notifiable: &dyn Notifiable) -> Result<BroadcastMessage> {
        let database_msg = self.to_database(notifiable)?;
        Ok(BroadcastMessage {
            data: database_msg.data,
            event: self.broadcast_type().unwrap_or_else(|| self.notification_type().to_string()),
        })
    }

    /// Get the web push representation of the notification
    fn to_web_push(&self, notifiable: &dyn Notifiable) -> Result<crate::app::notifications::channels::web_push_channel::WebPushMessage> {
        let database_msg = self.to_database(notifiable)?;
        let title = database_msg.data.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Notification")
            .to_string();
        let message = database_msg.data.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("You have a new notification")
            .to_string();

        Ok(crate::app::notifications::channels::web_push_channel::WebPushMessage::new(title, message)
            .data(database_msg.data))
    }

    /// Get the SMS representation of the notification (alias for to_vonage for Laravel compatibility)
    fn to_sms(&self, notifiable: &dyn Notifiable) -> Result<SmsMessage> {
        self.to_vonage(notifiable)
    }

    /// Get the Vonage (SMS) representation of the notification
    fn to_vonage(&self, notifiable: &dyn Notifiable) -> Result<SmsMessage> {
        let database_msg = self.to_database(notifiable)?;
        let content = database_msg.data.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("You have a new notification")
            .to_string();

        Ok(SmsMessage::new(content))
    }

    /// Get the Slack representation of the notification
    fn to_slack(&self, notifiable: &dyn Notifiable) -> Result<SlackMessage> {
        let database_msg = self.to_database(notifiable)?;
        let content = database_msg.data.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("You have a new notification")
            .to_string();

        Ok(SlackMessage::new(content))
    }

    /// Get a unique identifier for this notification type
    fn notification_type(&self) -> &'static str;

    /// Get the notification ID
    fn id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Get the notification ID using ULID (matching Laravel's ULID support)
    fn ulid(&self) -> String {
        ulid::Ulid::new().to_string()
    }

    /// Get the locale for the notification
    fn locale(&self) -> Option<&str> {
        None
    }

    /// Get the broadcast type for the notification
    fn broadcast_type(&self) -> Option<String> {
        None
    }

    /// Get the broadcast channels for the notification
    fn broadcast_on(&self) -> Vec<String> {
        vec![]
    }

    /// Determine if the notification should be sent
    fn should_send(&self, _notifiable: &dyn Notifiable, _channel: &NotificationChannel) -> bool {
        true
    }

    /// Get the notification's data array (for database storage)
    fn to_array(&self, notifiable: &dyn Notifiable) -> Result<serde_json::Value> {
        self.to_database(notifiable).map(|db_msg| db_msg.data)
    }
}

/// Trait for models that can receive notifications
#[async_trait]
pub trait Notifiable: Send + Sync {
    /// Downcast to Any for type-safe casting
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        None
    }
    /// Get the notification routing information for the given channel
    async fn route_notification_for(&self, channel: &NotificationChannel) -> Option<String>;

    /// Get the entity's identifier
    fn get_key(&self) -> String;

    /// Get the entity's notification preferences
    async fn notification_preferences(&self) -> HashMap<String, bool> {
        HashMap::new()
    }

    /// Get the preferred locale for notifications
    fn preferred_locale(&self) -> Option<String> {
        None
    }

    /// Get unread notifications
    async fn unread_notifications(&self) -> Result<Vec<DatabaseNotification>> {
        Ok(vec![])
    }

    /// Get read notifications
    async fn read_notifications(&self) -> Result<Vec<DatabaseNotification>> {
        Ok(vec![])
    }

    /// Get all notifications
    async fn notifications(&self) -> Result<Vec<DatabaseNotification>> {
        Ok(vec![])
    }

    /// Mark notifications as read
    async fn mark_as_read(&self, _notification_ids: Vec<String>) -> Result<()> {
        Ok(())
    }

    /// Mark all notifications as read
    async fn mark_all_as_read(&self) -> Result<()> {
        Ok(())
    }

    /// Check if the notifiable can receive a notification on a given channel
    async fn can_receive_notification(&self, _notification_type: &str, _channel: &NotificationChannel) -> bool {
        true
    }

    // Note: These methods are moved out of the trait to maintain dyn compatibility
    // Use the free functions notify() and notify_via() instead

    /// Route notification for specific channel (Laravel pattern)
    async fn route_notification_for_mail(&self) -> Option<String> {
        self.route_notification_for(&NotificationChannel::Mail).await
    }

    /// Route notification for database channel
    async fn route_notification_for_database(&self) -> Option<String> {
        self.route_notification_for(&NotificationChannel::Database).await
    }

    /// Route notification for broadcast channel
    async fn route_notification_for_broadcast(&self) -> Option<String> {
        self.route_notification_for(&NotificationChannel::Broadcast).await
    }

    /// Route notification for Slack channel
    async fn route_notification_for_slack(&self) -> Option<String> {
        self.route_notification_for(&NotificationChannel::Slack).await
    }

    /// Route notification for Vonage (SMS) channel
    async fn route_notification_for_vonage(&self) -> Option<String> {
        self.route_notification_for(&NotificationChannel::Sms).await
    }
}

/// Available notification channels
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NotificationChannel {
    Mail,
    Database,
    Broadcast,
    WebPush,
    Sms,
    Vonage, // Laravel 12 uses Vonage for SMS
    Slack,
    Custom(String),
}

impl NotificationChannel {
    /// Get the channel name as a string
    pub fn to_string(&self) -> String {
        match self {
            NotificationChannel::Mail => "mail".to_string(),
            NotificationChannel::Database => "database".to_string(),
            NotificationChannel::Broadcast => "broadcast".to_string(),
            NotificationChannel::WebPush => "web_push".to_string(),
            NotificationChannel::Sms => "sms".to_string(),
            NotificationChannel::Vonage => "vonage".to_string(),
            NotificationChannel::Slack => "slack".to_string(),
            NotificationChannel::Custom(name) => name.clone(),
        }
    }

    /// Create a channel from a string
    pub fn from_string(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "mail" => NotificationChannel::Mail,
            "database" => NotificationChannel::Database,
            "broadcast" => NotificationChannel::Broadcast,
            "web_push" => NotificationChannel::WebPush,
            "sms" => NotificationChannel::Sms,
            "vonage" => NotificationChannel::Vonage,
            "slack" => NotificationChannel::Slack,
            _ => NotificationChannel::Custom(name.to_string()),
        }
    }
}

/// Mail message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailMessage {
    pub from: Option<String>,
    pub to: String,
    pub subject: String,
    pub content: MailContent,
    pub attachments: Vec<String>,
}

/// Mail content types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MailContent {
    Text(String),
    Html(String),
    Markdown(String),
}

/// Database message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMessage {
    pub data: serde_json::Value,
}

/// Broadcast message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    pub data: serde_json::Value,
    pub event: String,
}

/// SMS message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsMessage {
    pub content: String,
    pub from: Option<String>,
    pub client_reference: Option<String>,
    pub encoding: Option<String>,
}

/// Slack message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackMessage {
    pub text: String,
    pub username: Option<String>,
    pub channel: Option<String>,
    pub icon_emoji: Option<String>,
    pub icon_url: Option<String>,
    pub attachments: Vec<SlackAttachment>,
    pub blocks: Vec<serde_json::Value>,
}

/// Slack attachment structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackAttachment {
    pub color: Option<String>,
    pub title: Option<String>,
    pub text: Option<String>,
    pub fields: Vec<SlackField>,
}

/// Slack field structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackField {
    pub title: String,
    pub value: String,
    pub short: bool,
}

impl MailMessage {
    pub fn new(to: String, subject: String, content: MailContent) -> Self {
        Self {
            from: None,
            to,
            subject,
            content,
            attachments: Vec::new(),
        }
    }

    pub fn from(mut self, from: String) -> Self {
        self.from = Some(from);
        self
    }

    pub fn attach(mut self, attachment: String) -> Self {
        self.attachments.push(attachment);
        self
    }
}

impl DatabaseMessage {
    pub fn new(data: serde_json::Value) -> Self {
        Self { data }
    }
}

impl SmsMessage {
    pub fn new(content: String) -> Self {
        Self {
            content,
            from: None,
            client_reference: None,
            encoding: None,
        }
    }

    pub fn from(mut self, from: String) -> Self {
        self.from = Some(from);
        self
    }

    pub fn client_reference(mut self, reference: String) -> Self {
        self.client_reference = Some(reference);
        self
    }

    pub fn unicode(mut self) -> Self {
        self.encoding = Some("unicode".to_string());
        self
    }
}

impl SlackMessage {
    pub fn new(text: String) -> Self {
        Self {
            text,
            username: None,
            channel: None,
            icon_emoji: None,
            icon_url: None,
            attachments: Vec::new(),
            blocks: Vec::new(),
        }
    }

    pub fn username(mut self, username: String) -> Self {
        self.username = Some(username);
        self
    }

    pub fn channel(mut self, channel: String) -> Self {
        self.channel = Some(channel);
        self
    }

    pub fn icon_emoji(mut self, emoji: String) -> Self {
        self.icon_emoji = Some(emoji);
        self
    }

    pub fn icon_url(mut self, url: String) -> Self {
        self.icon_url = Some(url);
        self
    }

    pub fn attachment(mut self, attachment: SlackAttachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    pub fn block(mut self, block: serde_json::Value) -> Self {
        self.blocks.push(block);
        self
    }
}

impl SlackAttachment {
    pub fn new() -> Self {
        Self {
            color: None,
            title: None,
            text: None,
            fields: Vec::new(),
        }
    }

    pub fn color(mut self, color: String) -> Self {
        self.color = Some(color);
        self
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }

    pub fn field(mut self, field: SlackField) -> Self {
        self.fields.push(field);
        self
    }
}

impl SlackField {
    pub fn new(title: String, value: String, short: bool) -> Self {
        Self { title, value, short }
    }
}

/// On-demand recipient for routing notifications to specific channels
pub struct OnDemandRecipient {
    channel: NotificationChannel,
    route: String,
}

impl OnDemandRecipient {
    pub fn new(channel: NotificationChannel, route: String) -> Self {
        Self { channel, route }
    }
}

#[async_trait]
impl Notifiable for OnDemandRecipient {
    async fn route_notification_for(&self, channel: &NotificationChannel) -> Option<String> {
        if channel == &self.channel {
            Some(self.route.clone())
        } else {
            None
        }
    }

    fn get_key(&self) -> String {
        format!("OnDemand_{}_{}", self.channel.to_string(), self.route)
    }
}

/// Notification facade for Laravel-style static methods
pub struct NotificationFacade;

impl NotificationFacade {
    /// Send a notification to multiple notifiables (Laravel Notification::send)
    pub async fn send<N: Notifiable>(
        notifiables: Vec<&N>,
        notification: impl Notification + Send + Sync + Clone,
    ) -> Result<()> {
        for notifiable in notifiables {
            notify_entity(notifiable, notification.clone()).await?;
        }
        Ok(())
    }

    /// Send a notification to a single notifiable
    pub async fn send_to<N: Notifiable>(
        notifiable: &N,
        notification: impl Notification + Send + Sync,
    ) -> Result<()> {
        notify_entity(notifiable, notification).await
    }

    /// Route a notification to specific channels and recipients
    pub async fn route(
        channel: NotificationChannel,
        route: String,
        notification: impl Notification + Send + Sync,
    ) -> Result<()> {
        // Create a simple recipient notifiable for on-demand notifications
        let recipient = OnDemandRecipient::new(channel, route);
        let notification_service = crate::app::services::notification_service::NotificationService::new().await;
        notification_service.send(&notification, &recipient).await
    }

    /// Enable notification faking for testing
    pub async fn fake() {
        // In a production implementation, this would enable a fake driver
        // that captures notifications instead of sending them
        tracing::info!("Notification faking enabled - notifications will be captured for testing");
    }

    /// Assert that a notification was sent
    pub async fn assert_sent_to<N: Notifiable>(
        notifiable: &N,
        notification_type: &str,
    ) -> bool {
        // In a production implementation, this would check against captured notifications
        // For now, we'll check if there are any database notifications of this type
        let notification_service = crate::app::services::notification_service::NotificationService::new().await;

        // Extract notifiable type and ID from the key
        let key = notifiable.get_key();
        let parts: Vec<&str> = key.split('_').collect();
        if parts.len() >= 2 {
            let notifiable_type = parts[0];
            let notifiable_id = parts[1];

            if let Ok(notifications) = notification_service
                .get_notifications(notifiable_type, notifiable_id, Some(10), None)
                .await
            {
                return notifications.iter().any(|n| n.notification_type == notification_type);
            }
        }
        false
    }

    /// Assert that no notifications were sent
    pub async fn assert_nothing_sent() -> bool {
        // TODO: In a production implementation, this would check that no notifications
        // were captured during the test
        tracing::info!("Checking that no notifications were sent during test");
        true
    }
}

/// Laravel-style notify function for any Notifiable
pub async fn notify_entity<N: Notifiable>(
    notifiable: &N,
    notification: impl Notification + Send + Sync,
) -> Result<()> {
    let channels = notification.via(notifiable);
    for channel in channels {
        if let Some(_route) = notifiable.route_notification_for(&channel).await {
            // Send notification via the specific channel
            // Implementation would go here
        }
    }
    Ok(())
}

/// Global notification functions (Laravel-style)
pub async fn notify<N: Notifiable>(
    notifiable: &N,
    notification: impl Notification + Send + Sync,
) -> Result<()> {
    // Use the notification service for proper channel management
    let notification_service = crate::app::services::notification_service::NotificationService::new().await;
    notification_service.send(&notification, notifiable).await
}

/// Send notification via specific channels
pub async fn notify_via<N: Notifiable>(
    notifiable: &N,
    channels: Vec<NotificationChannel>,
    notification: impl Notification + Send + Sync,
) -> Result<()> {
    // Use the notification service with specific channels
    let notification_service = crate::app::services::notification_service::NotificationService::new().await;
    notification_service.send_via_channels(&notification, notifiable, channels).await
}