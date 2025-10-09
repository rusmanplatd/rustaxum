-- Remove notification preference columns
ALTER TABLE sys_users DROP COLUMN email_notifications;
ALTER TABLE sys_users DROP COLUMN push_notifications;
ALTER TABLE sys_users DROP COLUMN sms_notifications;
ALTER TABLE sys_users DROP COLUMN slack_notifications;
ALTER TABLE sys_users DROP COLUMN marketing_emails;
ALTER TABLE sys_users DROP COLUMN security_alerts;
ALTER TABLE sys_users DROP COLUMN system_updates;
ALTER TABLE sys_users DROP COLUMN mention_notifications;
ALTER TABLE sys_users DROP COLUMN comment_notifications;
ALTER TABLE sys_users DROP COLUMN follow_notifications;