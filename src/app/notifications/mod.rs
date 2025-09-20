pub mod notification;
pub mod channels;
pub mod notifiable;
pub mod welcome_notification;
pub mod invoice_paid_notification;
pub mod order_shipped_notification;

// Re-export main traits and types for easier imports
pub use notification::{
    Notification, Notifiable, NotificationChannel,
    MailMessage, MailContent, DatabaseMessage, BroadcastMessage,
    SmsMessage, SlackMessage, SlackAttachment, SlackField,
    ShouldQueue, ShouldQueueAfterCommit, Queueable, HasLocalePreference,
    NotificationFacade, notify, notify_via
};

// Re-export notification channels
pub use channels::{
    mail_channel::MailChannel,
    database_channel::DatabaseChannel,
    broadcast_channel::BroadcastChannel,
    sms_channel::SmsChannel,
    slack_channel::SlackChannel,
    web_push_channel::WebPushChannel,
};