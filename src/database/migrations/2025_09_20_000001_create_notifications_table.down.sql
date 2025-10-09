-- Drop the notifications table and related objects
DROP TRIGGER IF EXISTS update_notifications_updated_at ON notifications;
DROP INDEX IF EXISTS idx_notifications_failed;
DROP INDEX IF EXISTS idx_notifications_unread;
DROP INDEX IF EXISTS idx_notifications_priority;
DROP INDEX IF EXISTS idx_notifications_created_at;
DROP INDEX IF EXISTS idx_notifications_scheduled_at;
DROP INDEX IF EXISTS idx_notifications_failed_at;
DROP INDEX IF EXISTS idx_notifications_sent_at;
DROP INDEX IF EXISTS idx_notifications_read_at;
DROP INDEX IF EXISTS idx_notifications_notifiable;
DROP INDEX IF EXISTS idx_notifications_type;
DROP TABLE notifications;