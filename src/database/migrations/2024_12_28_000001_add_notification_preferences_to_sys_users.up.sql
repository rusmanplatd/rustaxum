-- Migration: Add notification preference columns to sys_users table
-- Laravel-style individual boolean columns for notification preferences

ALTER TABLE sys_users
ADD COLUMN email_notifications BOOLEAN DEFAULT true,
ADD COLUMN database_notifications BOOLEAN DEFAULT true,
ADD COLUMN broadcast_notifications BOOLEAN DEFAULT true,
ADD COLUMN web_push_notifications BOOLEAN DEFAULT true,
ADD COLUMN sms_notifications BOOLEAN DEFAULT true,
ADD COLUMN slack_notifications BOOLEAN DEFAULT false,
ADD COLUMN marketing_emails BOOLEAN DEFAULT false,
ADD COLUMN security_alerts BOOLEAN DEFAULT true,
ADD COLUMN order_updates BOOLEAN DEFAULT true,
ADD COLUMN newsletter BOOLEAN DEFAULT false,
ADD COLUMN promotional_emails BOOLEAN DEFAULT false,
ADD COLUMN account_notifications BOOLEAN DEFAULT true;

-- Create indexes for commonly queried preferences
CREATE INDEX idx_sys_users_email_notifications ON sys_users (email_notifications);
CREATE INDEX idx_sys_users_marketing_emails ON sys_users (marketing_emails);
CREATE INDEX idx_sys_users_security_alerts ON sys_users (security_alerts);

-- Add comments for documentation
COMMENT ON COLUMN sys_users.email_notifications IS 'Enable email notifications';
COMMENT ON COLUMN sys_users.database_notifications IS 'Enable database notifications';
COMMENT ON COLUMN sys_users.broadcast_notifications IS 'Enable real-time broadcast notifications';
COMMENT ON COLUMN sys_users.web_push_notifications IS 'Enable web push notifications';
COMMENT ON COLUMN sys_users.sms_notifications IS 'Enable SMS notifications';
COMMENT ON COLUMN sys_users.slack_notifications IS 'Enable Slack notifications';
COMMENT ON COLUMN sys_users.marketing_emails IS 'Enable marketing emails';
COMMENT ON COLUMN sys_users.security_alerts IS 'Enable security alert notifications';
COMMENT ON COLUMN sys_users.order_updates IS 'Enable order update notifications';
COMMENT ON COLUMN sys_users.newsletter IS 'Enable newsletter subscription';
COMMENT ON COLUMN sys_users.promotional_emails IS 'Enable promotional emails';
COMMENT ON COLUMN sys_users.account_notifications IS 'Enable account-related notifications';