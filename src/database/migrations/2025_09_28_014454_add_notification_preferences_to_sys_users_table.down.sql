-- Remove notification preference columns
ALTER TABLE sys_users DROP COLUMN IF EXISTS email_notifications;
ALTER TABLE sys_users DROP COLUMN IF EXISTS push_notifications;
ALTER TABLE sys_users DROP COLUMN IF EXISTS sms_notifications;
ALTER TABLE sys_users DROP COLUMN IF EXISTS slack_notifications;
ALTER TABLE sys_users DROP COLUMN IF EXISTS marketing_emails;
ALTER TABLE sys_users DROP COLUMN IF EXISTS security_alerts;
ALTER TABLE sys_users DROP COLUMN IF EXISTS system_updates;
ALTER TABLE sys_users DROP COLUMN IF EXISTS mention_notifications;
ALTER TABLE sys_users DROP COLUMN IF EXISTS comment_notifications;
ALTER TABLE sys_users DROP COLUMN IF EXISTS follow_notifications;