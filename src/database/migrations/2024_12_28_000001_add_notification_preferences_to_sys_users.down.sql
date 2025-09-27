-- Rollback: Remove notification preference columns from sys_users table

-- Drop indexes first
DROP INDEX IF EXISTS idx_sys_users_email_notifications;
DROP INDEX IF EXISTS idx_sys_users_marketing_emails;
DROP INDEX IF EXISTS idx_sys_users_security_alerts;

-- Remove notification preference columns
ALTER TABLE sys_users
DROP COLUMN IF EXISTS email_notifications,
DROP COLUMN IF EXISTS database_notifications,
DROP COLUMN IF EXISTS broadcast_notifications,
DROP COLUMN IF EXISTS web_push_notifications,
DROP COLUMN IF EXISTS sms_notifications,
DROP COLUMN IF EXISTS slack_notifications,
DROP COLUMN IF EXISTS marketing_emails,
DROP COLUMN IF EXISTS security_alerts,
DROP COLUMN IF EXISTS order_updates,
DROP COLUMN IF EXISTS newsletter,
DROP COLUMN IF EXISTS promotional_emails,
DROP COLUMN IF EXISTS account_notifications;